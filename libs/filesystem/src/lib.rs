#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
pub mod std_helpers;

use mutex::Mutex;
#[cfg(feature = "std")]
pub use std_helpers::*;

/// MBR partition table implementation
pub mod mbr;

pub trait Disk {
    /// The buffer size is the sector size
    fn read_sector(&self, sector: usize, buf: &mut [u8]) -> Result<(), DiskErr>;

    /// The buffer size is the sector size
    fn write_sector(&self, sector: usize, buf: &[u8]) -> Result<(), DiskErr>;

    fn disk_infos(&self) -> Result<DiskInfos, DiskErr>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DiskErr {
    /// Will trigger if the size of the buffer doesn't match a supported sector size.
    ///
    /// `found` is the size of the provided buffer
    ///
    /// `available` is a list of the supported sector sizes
    InvalidSectorSize { found: usize, supported: SectorSize },

    /// Will trigger if the sector index is out of the range of the disk.
    ///
    /// `found` is the provided sector index (lba)
    ///
    /// `max` is the last existing sector index **with the size of the given buffer**
    InvalidSectorIndex { found: usize, max: usize },

    /// Will trigger if a write is performed on a read-only disk or if the program tries to read a
    /// write-only disk
    InvalidPermission { disk_permission: Permission },

    /// Will trigger if, for any reason, the disk is not found anymore
    UnreachableDisk,

    IOErr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiskInfos {
    pub sector_sizes: SectorSize,
    /// The disk size in bytes
    pub disk_size: usize,
    /// Specially useful when working with disk images, or without `sudo` privileges
    pub permission: Permission,
}

/// Informs the supported sector sizes. A sector size superior to the disk size is always invalid
/// and should trigger an error (DiskErr::InvalidSectorSize).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectorSize {
    /// All sector sizes are supported
    Any,

    /// All sector sizes in the list are supported
    AllOf(&'static [usize]),
    /// All sector sizes are supported expected the ones in the list
    AnyExpected(&'static [usize]),

    /// All sector sizes in one of the ranges are supported. min <= size < max
    InRanges(&'static [(usize, usize)]),
    /// All sector sizes are supported expected the ones in one of these ranges. min <= size < max
    AnyExpectedRanges(&'static [(usize, usize)]),
}

/// These permissions are only intented for disk usage, **not** for filesystems. They only are a
/// clue about the context in which the program is called, and to avoid accidental writes. IF THE
/// DISK CAN BE WRITTEN, A READ ONLY FILESYSTEM OR PARTITION ARE NOT A GUARANTEE. THIS IS THE
/// `Disk` IMPLEMENTATION RESPONSIBILTY TO CHECK THE PERMISSION, THE CALLER MAY TRY ILLEGAL
/// OPERATIONS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

impl SectorSize {
    pub fn is_supported(&self, sector_size: usize, disk_size: usize) -> bool {
        (match self {
            Self::Any => true,
            Self::AllOf(l) => l.contains(&sector_size),
            Self::AnyExpected(l) => !l.contains(&sector_size),
            Self::InRanges(rs) => rs.iter().any(|r| r.0 <= sector_size && sector_size < r.1),
            Self::AnyExpectedRanges(rs) => {
                !rs.iter().any(|r| r.0 <= sector_size && sector_size < r.1)
            }
        }) && (sector_size <= disk_size)
    }
}
