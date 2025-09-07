#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

/// Procides an implementation of the `Disk` trait for `std::fs`
#[cfg(feature = "std")]
pub mod std_helpers;

#[cfg(feature = "std")]
pub use std_helpers::*;

/// MBR partition table implementation
pub mod mbr;

/// Procides disk wrappers to allow subdisk creation. `SubDisk`s are useful when working with
/// partitions or filesystems for example.
pub mod wrappers;

/// The main trait representing any disk. Provides only minimal methods: read, write, and infos.
/// The disks work with sectors, but there are various possible sector sizes. To allow more
/// flexibility, this library is not directly dependent of the sector size of the disk, even if it
/// can be the case in some partition tables, partition types, or filesystems. It also allows a
/// same disk to support multiple sector sizes (typically when working with disk image).
pub trait Disk {
    /// The size of the buffer is implicitly the sector size (in bytes). `sector` is the LBA of the
    /// sector. It's the implementation responsibility to check the sector and the disk sizes, the
    /// caller may produce invalid requests.
    fn read_sector(&self, sector: usize, buf: &mut [u8]) -> Result<(), DiskErr>;

    /// The size of the buffer is implicitly the sector size (in bytes). `sector` is the LBA of the
    /// sector. It's the implementation responsibility to check the sector and the disk sizes, the
    /// caller may produce invalid requests.
    fn write_sector(&self, sector: usize, buf: &[u8]) -> Result<(), DiskErr>;

    fn disk_infos(&self) -> Result<DiskInfos, DiskErr>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DiskErr {
    /// Will trigger if the size of the buffer isn't supported.
    ///
    /// `found` is the size of the provided buffer (`buf.len()`)
    ///
    /// `supported` is the supported sector size(s)
    ///
    /// `start % sector_size` should be zero (used with subdisks)
    InvalidSectorSize {
        found: usize,
        supported: SectorSize,
        start: usize,
    },

    /// Will trigger if the sector index is out of the range of the disk.
    ///
    /// `found` is the provided sector index (lba)
    ///
    /// `max` is the last existing sector index **with the size of the given buffer**
    InvalidSectorIndex { found: usize, max: usize },

    /// Will trigger if a write is performed on a read-only disk or if the program tries to read a
    /// write-only disk
    InvalidPermission { disk_permissions: Permissions },

    /// Will trigger if, for any reason, the disk is not found anymore.
    UnreachableDisk,

    /// Will trigger when attempting to create a subdisk out of the range of the original disk
    /// size
    InvalidDiskSize,

    /// Will trigger if a read/write/subdisk creation is requested when the disk is already in
    /// use/on a space already borrowed
    Busy,

    /// Will trigger for all the errors coming from IO processes
    IOErr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiskInfos {
    pub sector_size: SectorSize,
    /// The disk size in bytes
    pub disk_size: usize,
    /// Specially useful when working with disk images, or without `sudo` privileges
    pub permissions: Permissions,
}

/// Informs the supported sector sizes. A sector size superior to the disk size is always invalid
/// and should trigger an error `DiskErr::InvalidSectorSize`.
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
/// DISK CAN BE WRITTEN, A READ ONLY FILESYSTEM OR PARTITION IS NOT A GUARANTEE. THIS IS THE
/// `Disk` IMPLEMENTATION RESPONSIBILTY TO CHECK THE PERMISSIONS, THE CALLER MAY TRY ILLEGAL
/// OPERATIONS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permissions {
    pub read: bool,
    pub write: bool,
}

impl Permissions {
    pub const fn read_only() -> Self {
        Self {
            read: true,
            write: false,
        }
    }

    pub const fn write_only() -> Self {
        Self {
            read: false,
            write: true,
        }
    }

    pub const fn read_write() -> Self {
        Self {
            read: true,
            write: true,
        }
    }
}

impl SectorSize {
    /// Checks if a given sector size is supported.
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
