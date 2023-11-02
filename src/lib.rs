//! # embedded-sdmmc
//!
//! > An SD/MMC Library written in Embedded Rust
//!
//! This crate is intended to allow you to read/write files on a FAT formatted
//! SD card on your Rust Embedded device, as easily as using the `SdFat` Arduino
//! library. It is written in pure-Rust, is `#![no_std]` and does not use
//! `alloc` or `collections` to keep the memory footprint low. In the first
//! instance it is designed for readability and simplicity over performance.
//!
//! ## Using the crate
//!
//! You will need something that implements the `BlockDevice` trait, which can
//! read and write the 512-byte blocks (or sectors) from your card. If you were
//! to implement this over USB Mass Storage, there's no reason this crate
//! couldn't work with a USB Thumb Drive, but we only supply a `BlockDevice`
//! suitable for reading SD and SDHC cards over SPI.
//!
//! ```rust,no_run
//! # struct DummySpi;
//! # struct DummyCsPin;
//! # struct DummyUart;
//! # struct DummyTimeSource;
//! # struct DummyDelayer;
//! # use embedded_hal::spi::Operation;
//! # use embedded_hal::spi::ErrorType;
//! # impl ErrorType for DummySpi {
//! #     type Error = ();
//! # }
//! # impl embedded_hal::spi::SpiDevice<u8> for DummySpi {
//! #
//! # fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
//! #         Ok(())
//! #     }
//! #
//! # fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
//! #         Ok(())
//! #     }
//! #
//! # fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
//! #         Ok(())
//! #     }
//! #
//! # fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
//! #         Ok(())
//! #     }
//! #
//! # fn transfer_in_place(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
//! #         Ok(())
//! #     }
//! # }
//! # impl embedded_hal::digital::ErrorType for DummyCsPin {
//! #     type Error = ();
//! # }
//! # impl embedded_hal::digital::OutputPin for DummyCsPin {
//! #   fn set_low(&mut self) -> Result<(), ()> { Ok(()) }
//! #   fn set_high(&mut self) -> Result<(), ()> { Ok(()) }
//! # }
//! # impl embedded_sdmmc::TimeSource for DummyTimeSource {
//! #   fn get_timestamp(&self) -> embedded_sdmmc::Timestamp { embedded_sdmmc::Timestamp::from_fat(0, 0) }
//! # }
//! # impl embedded_hal::delay::DelayUs for DummyDelayer {
//! #   fn delay_us(&mut self, us: u8) {}
//! # }
//! # impl std::fmt::Write for DummyUart {
//! #     fn write_str(&mut self, s: &str) -> std::fmt::Result { Ok(()) }
//! # }
//! # use std::fmt::Write;
//! # use embedded_sdmmc::VolumeManager;
//! # fn main() -> Result<(), embedded_sdmmc::Error<embedded_sdmmc::SdCardError>> {
//! # let mut sdmmc_spi = DummySpi;
//! # let mut sdmmc_cs = DummyCsPin;
//! # let time_source = DummyTimeSource;
//! # let delayer = DummyDelayer;
//! # let sdcard = embedded_sdmmc::SdCard::new(sdmmc_spi, sdmmc_cs, delayer);
//! # println!("Card size is {} bytes", sdcard.num_bytes()?);
//! # let mut volume_mgr = embedded_sdmmc::VolumeManager::new(sdcard, time_source);
//! # let mut volume0 = volume_mgr.open_volume(embedded_sdmmc::VolumeIdx(0))?;
//! # println!("Volume 0: {:?}", volume0);
//! # let mut root_dir = volume0.open_root_dir()?;
//! # let mut my_file = root_dir.open_file_in_dir("MY_FILE.TXT", embedded_sdmmc::Mode::ReadOnly)?;
//! # while !my_file.is_eof() {
//! #     let mut buffer = [0u8; 32];
//! #     let num_read = my_file.read(&mut buffer)?;
//! #     for b in &buffer[0..num_read] {
//! #         print!("{}", *b as char);
//! #     }
//! # }
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! * `log`: Enabled by default. Generates log messages using the `log` crate.
//! * `defmt-log`: By turning off the default features and enabling the
//! `defmt-log` feature you can configure this crate to log messages over defmt
//! instead.
//!
//! You cannot enable both the `log` feature and the `defmt-log` feature.

#![cfg_attr(not(test), no_std)]
#![deny(missing_docs)]

// ****************************************************************************
//
// Imports
//
// ****************************************************************************

#[cfg(test)]
use hex_literal::hex;

#[macro_use]
mod structure;

pub mod blockdevice;
pub mod fat;
pub mod filesystem;
pub mod sdcard;

use filesystem::SearchId;

#[doc(inline)]
pub use crate::blockdevice::{Block, BlockCount, BlockDevice, BlockIdx};

#[doc(inline)]
pub use crate::fat::FatVolume;

#[doc(inline)]
pub use crate::filesystem::{
    Attributes, ClusterId, DirEntry, Directory, File, FilenameError, Mode, RawDirectory, RawFile,
    ShortFileName, TimeSource, Timestamp, MAX_FILE_SIZE,
};

use filesystem::DirectoryInfo;

#[doc(inline)]
pub use crate::sdcard::Error as SdCardError;

#[doc(inline)]
pub use crate::sdcard::SdCard;

mod volume_mgr;
#[doc(inline)]
pub use volume_mgr::VolumeManager;

#[cfg(all(feature = "defmt-log", feature = "log"))]
compile_error!("Cannot enable both log and defmt-log");

#[cfg(feature = "log")]
use log::{debug, trace, warn};

#[cfg(feature = "defmt-log")]
use defmt::{debug, trace, warn};

#[cfg(all(not(feature = "defmt-log"), not(feature = "log")))]
#[macro_export]
/// Like log::debug! but does nothing at all
macro_rules! debug {
    ($($arg:tt)+) => {};
}

#[cfg(all(not(feature = "defmt-log"), not(feature = "log")))]
#[macro_export]
/// Like log::trace! but does nothing at all
macro_rules! trace {
    ($($arg:tt)+) => {};
}

#[cfg(all(not(feature = "defmt-log"), not(feature = "log")))]
#[macro_export]
/// Like log::warn! but does nothing at all
macro_rules! warn {
    ($($arg:tt)+) => {};
}

// ****************************************************************************
//
// Public Types
//
// ****************************************************************************

/// Represents all the ways the functions in this crate can fail.
#[cfg_attr(feature = "defmt-log", derive(defmt::Format))]
#[derive(Debug, Clone)]
pub enum Error<E>
where
    E: core::fmt::Debug,
{
    /// The underlying block device threw an error.
    DeviceError(E),
    /// The filesystem is badly formatted (or this code is buggy).
    FormatError(&'static str),
    /// The given `VolumeIdx` was bad,
    NoSuchVolume,
    /// The given filename was bad
    FilenameError(FilenameError),
    /// Out of memory opening volumes
    TooManyOpenVolumes,
    /// Out of memory opening directories
    TooManyOpenDirs,
    /// Out of memory opening files
    TooManyOpenFiles,
    /// Bad handle given
    BadHandle,
    /// That file doesn't exist
    FileNotFound,
    /// You can't open a file twice or delete an open file
    FileAlreadyOpen,
    /// You can't open a directory twice
    DirAlreadyOpen,
    /// You can't open a directory as a file
    OpenedDirAsFile,
    /// You can't open a file as a directory
    OpenedFileAsDir,
    /// You can't delete a directory as a file
    DeleteDirAsFile,
    /// You can't close a volume with open files or directories
    VolumeStillInUse,
    /// You can't open a volume twice
    VolumeAlreadyOpen,
    /// We can't do that yet
    Unsupported,
    /// Tried to read beyond end of file
    EndOfFile,
    /// Found a bad cluster
    BadCluster,
    /// Error while converting types
    ConversionError,
    /// The device does not have enough space for the operation
    NotEnoughSpace,
    /// Cluster was not properly allocated by the library
    AllocationError,
    /// Jumped to free space during FAT traversing
    UnterminatedFatChain,
    /// Tried to open Read-Only file with write mode
    ReadOnly,
    /// Tried to create an existing file
    FileAlreadyExists,
    /// Bad block size - only 512 byte blocks supported
    BadBlockSize(u16),
    /// Bad offset given when seeking
    InvalidOffset,
    /// Disk is full
    DiskFull,
}

impl<E> From<E> for Error<E>
where
    E: core::fmt::Debug,
{
    fn from(value: E) -> Error<E> {
        Error::DeviceError(value)
    }
}

/// Represents a partition with a filesystem within it.
#[cfg_attr(feature = "defmt-log", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RawVolume(SearchId);

impl RawVolume {
    /// Convert a raw volume into a droppable [`Volume`]
    pub fn to_volume<
        D,
        T,
        const MAX_DIRS: usize,
        const MAX_FILES: usize,
        const MAX_VOLUMES: usize,
    >(
        self,
        volume_mgr: &mut VolumeManager<D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
    ) -> Volume<D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>
    where
        D: crate::BlockDevice,
        T: crate::TimeSource,
    {
        Volume::new(self, volume_mgr)
    }
}

/// Represents an open volume on disk.
///
/// If you drop a value of this type, it closes the volume automatically. However,
/// it holds a mutable reference to its parent `VolumeManager`, which restricts
/// which operations you can perform.
pub struct Volume<'a, D, T, const MAX_DIRS: usize, const MAX_FILES: usize, const MAX_VOLUMES: usize>
where
    D: crate::BlockDevice,
    T: crate::TimeSource,
{
    raw_volume: RawVolume,
    volume_mgr: &'a mut VolumeManager<D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
}

impl<'a, D, T, const MAX_DIRS: usize, const MAX_FILES: usize, const MAX_VOLUMES: usize>
    Volume<'a, D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>
where
    D: crate::BlockDevice,
    T: crate::TimeSource,
{
    /// Create a new `Volume` from a `RawVolume`
    pub fn new(
        raw_volume: RawVolume,
        volume_mgr: &'a mut VolumeManager<D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
    ) -> Volume<'a, D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES> {
        Volume {
            raw_volume,
            volume_mgr,
        }
    }

    /// Open the volume's root directory.
    ///
    /// You can then read the directory entries with `iterate_dir`, or you can
    /// use `open_file_in_dir`.
    pub fn open_root_dir(
        &mut self,
    ) -> Result<crate::Directory<D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>, Error<D::Error>> {
        let d = self.volume_mgr.open_root_dir(self.raw_volume)?;
        Ok(d.to_directory(self.volume_mgr))
    }

    /// Convert back to a raw volume
    pub fn to_raw_volume(self) -> RawVolume {
        let v = self.raw_volume;
        core::mem::forget(self);
        v
    }
}

impl<'a, D, T, const MAX_DIRS: usize, const MAX_FILES: usize, const MAX_VOLUMES: usize> Drop
    for Volume<'a, D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>
where
    D: crate::BlockDevice,
    T: crate::TimeSource,
{
    fn drop(&mut self) {
        self.volume_mgr
            .close_volume(self.raw_volume)
            .expect("Failed to close volume");
    }
}

impl<'a, D, T, const MAX_DIRS: usize, const MAX_FILES: usize, const MAX_VOLUMES: usize>
    core::fmt::Debug for Volume<'a, D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>
where
    D: crate::BlockDevice,
    T: crate::TimeSource,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Volume({})", self.raw_volume.0 .0)
    }
}

#[cfg(feature = "defmt-log")]
impl<'a, D, T, const MAX_DIRS: usize, const MAX_FILES: usize, const MAX_VOLUMES: usize>
    defmt::Format for Volume<'a, D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>
where
    D: crate::BlockDevice,
    T: crate::TimeSource,
{
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "Volume({})", self.raw_volume.0 .0)
    }
}

/// Internal information about a Volume
#[cfg_attr(feature = "defmt-log", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct VolumeInfo {
    /// Search ID for this volume.
    volume_id: RawVolume,
    /// TODO: some kind of index
    idx: VolumeIdx,
    /// What kind of volume this is
    volume_type: VolumeType,
}

/// This enum holds the data for the various different types of filesystems we
/// support.
#[cfg_attr(feature = "defmt-log", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub enum VolumeType {
    /// FAT16/FAT32 formatted volumes.
    Fat(FatVolume),
}

/// A `VolumeIdx` is a number which identifies a volume (or partition) on a
/// disk.
///
/// `VolumeIdx(0)` is the first primary partition on an MBR partitioned disk.
#[cfg_attr(feature = "defmt-log", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct VolumeIdx(pub usize);

/// Marker for a FAT32 partition. Sometimes also use for FAT16 formatted
/// partitions.
const PARTITION_ID_FAT32_LBA: u8 = 0x0C;
/// Marker for a FAT16 partition with LBA. Seen on a Raspberry Pi SD card.
const PARTITION_ID_FAT16_LBA: u8 = 0x0E;
/// Marker for a FAT16 partition. Seen on a card formatted with the official
/// SD-Card formatter.
const PARTITION_ID_FAT16: u8 = 0x06;
/// Marker for a FAT32 partition. What Macosx disk utility (and also SD-Card formatter?)
/// use.
const PARTITION_ID_FAT32_CHS_LBA: u8 = 0x0B;

// ****************************************************************************
//
// Unit Tests
//
// ****************************************************************************

// None

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
