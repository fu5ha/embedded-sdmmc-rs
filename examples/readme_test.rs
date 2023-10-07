//! This is the code from the README.md file.
//!
//! We add enough stuff to make it compile, but it won't run because our fake
//! SPI doesn't do any replies.

struct FakeSpi();

impl embedded_hal::blocking::spi::Transfer<u8> for FakeSpi {
    type Error = core::convert::Infallible;
    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        Ok(words)
    }
}

impl embedded_hal::blocking::spi::Write<u8> for FakeSpi {
    type Error = core::convert::Infallible;
    fn write(&mut self, _words: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }
}

struct FakeCs();

impl embedded_hal::digital::v2::OutputPin for FakeCs {
    type Error = core::convert::Infallible;
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

struct FakeDelayer();

impl embedded_hal::blocking::delay::DelayUs<u8> for FakeDelayer {
    fn delay_us(&mut self, us: u8) {
        std::thread::sleep(std::time::Duration::from_micros(u64::from(us)));
    }
}

struct FakeTimesource();

impl embedded_sdmmc::TimeSource for FakeTimesource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        embedded_sdmmc::Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

#[derive(Debug, Clone)]
enum Error {
    Filesystem(embedded_sdmmc::Error<embedded_sdmmc::SdCardError>),
    Disk(embedded_sdmmc::SdCardError),
}

impl From<embedded_sdmmc::Error<embedded_sdmmc::SdCardError>> for Error {
    fn from(value: embedded_sdmmc::Error<embedded_sdmmc::SdCardError>) -> Error {
        Error::Filesystem(value)
    }
}

impl From<embedded_sdmmc::SdCardError> for Error {
    fn from(value: embedded_sdmmc::SdCardError) -> Error {
        Error::Disk(value)
    }
}

fn main() -> Result<(), Error> {
    let sdmmc_spi = FakeSpi();
    let sdmmc_cs = FakeCs();
    let delay = FakeDelayer();
    let time_source = FakeTimesource();
    // Build an SD Card interface out of an SPI device, a chip-select pin and the delay object
    let sdcard = embedded_sdmmc::SdCard::new(sdmmc_spi, sdmmc_cs, delay);
    // Get the card size (this also triggers card initialisation because it's not been done yet)
    println!("Card size is {} bytes", sdcard.num_bytes()?);
    // Now let's look for volumes (also known as partitions) on our block device.
    // To do this we need a Volume Manager. It will take ownership of the block device.
    let mut volume_mgr = embedded_sdmmc::VolumeManager::new(sdcard, time_source);
    // Try and access Volume 0 (i.e. the first partition).
    // The volume object holds information about the filesystem on that volume.
    let mut volume0 = volume_mgr.open_volume(embedded_sdmmc::VolumeIdx(0))?;
    println!("Volume 0: {:?}", volume0);
    // Open the root directory (mutably borrows from the volume).
    let mut root_dir = volume0.open_root_dir()?;
    // Open a file called "MY_FILE.TXT" in the root directory
    // This mutably borrows the directory.
    let mut my_file = root_dir.open_file_in_dir("MY_FILE.TXT", embedded_sdmmc::Mode::ReadOnly)?;
    // Print the contents of the file
    while !my_file.is_eof() {
        let mut buffer = [0u8; 32];
        let num_read = my_file.read(&mut buffer)?;
        for b in &buffer[0..num_read] {
            print!("{}", *b as char);
        }
    }
    Ok(())
}

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
