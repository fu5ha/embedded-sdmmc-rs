use core::convert::TryFrom;

use crate::blockdevice::BlockIdx;
use crate::fat::{FatType, OnDiskDirEntry};
use crate::filesystem::{Attributes, ClusterId, SearchId, ShortFileName, Timestamp};
use crate::Volume;

/// Represents a directory entry, which tells you about
/// other files and directories.
#[cfg_attr(feature = "defmt-log", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DirEntry {
    /// The name of the file
    pub name: ShortFileName,
    /// When the file was last modified
    pub mtime: Timestamp,
    /// When the file was first created
    pub ctime: Timestamp,
    /// The file attributes (Read Only, Archive, etc)
    pub attributes: Attributes,
    /// The starting cluster of the file. The FAT tells us the following Clusters.
    pub cluster: ClusterId,
    /// The size of the file in bytes.
    pub size: u32,
    /// The disk block of this entry
    pub entry_block: BlockIdx,
    /// The offset on its block (in bytes)
    pub entry_offset: u32,
}

/// Represents an open directory on disk.
///
/// Do NOT drop this object! It doesn't hold a reference to the Volume Manager
/// it was created from and if you drop it, the VolumeManager will think you
/// still have the directory open, and it won't let you open the directory
/// again.
///
/// Instead you must pass it to [`crate::VolumeManager::close_dir`] to close it
/// cleanly.
///
/// If you want your directories to close themselves on drop, create your own
/// `Directory` type that wraps this one and also holds a `VolumeManager`
/// reference. You'll then also need to put your `VolumeManager` in some kind of
/// Mutex or RefCell, and deal with the fact you can't put them both in the same
/// struct any more because one refers to the other. Basically, it's complicated
/// and there's a reason we did it this way.
#[cfg_attr(feature = "defmt-log", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Directory(pub(crate) SearchId);

/// Holds information about an open file on disk
#[cfg_attr(feature = "defmt-log", derive(defmt::Format))]
#[derive(Debug, Clone)]
pub(crate) struct DirectoryInfo {
    /// Unique ID for this directory.
    pub(crate) directory_id: Directory,
    /// The unique ID for the volume this directory is on
    pub(crate) volume_id: Volume,
    /// The starting point of the directory listing.
    pub(crate) cluster: ClusterId,
}

impl DirEntry {
    pub(crate) fn serialize(&self, fat_type: FatType) -> [u8; OnDiskDirEntry::LEN] {
        let mut data = [0u8; OnDiskDirEntry::LEN];
        data[0..11].copy_from_slice(&self.name.contents);
        data[11] = self.attributes.0;
        // 12: Reserved. Must be set to zero
        // 13: CrtTimeTenth, not supported, set to zero
        data[14..18].copy_from_slice(&self.ctime.serialize_to_fat()[..]);
        // 0 + 18: LastAccDate, not supported, set to zero
        let cluster_number = self.cluster.0;
        let cluster_hi = if fat_type == FatType::Fat16 {
            [0u8; 2]
        } else {
            // Safe due to the AND operation
            u16::try_from((cluster_number >> 16) & 0x0000_FFFF)
                .unwrap()
                .to_le_bytes()
        };
        data[20..22].copy_from_slice(&cluster_hi[..]);
        data[22..26].copy_from_slice(&self.mtime.serialize_to_fat()[..]);
        // Safe due to the AND operation
        let cluster_lo = u16::try_from(cluster_number & 0x0000_FFFF)
            .unwrap()
            .to_le_bytes();
        data[26..28].copy_from_slice(&cluster_lo[..]);
        data[28..32].copy_from_slice(&self.size.to_le_bytes()[..]);
        data
    }

    pub(crate) fn new(
        name: ShortFileName,
        attributes: Attributes,
        cluster: ClusterId,
        ctime: Timestamp,
        entry_block: BlockIdx,
        entry_offset: u32,
    ) -> Self {
        Self {
            name,
            mtime: ctime,
            ctime,
            attributes,
            cluster,
            size: 0,
            entry_block,
            entry_offset,
        }
    }
}

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
