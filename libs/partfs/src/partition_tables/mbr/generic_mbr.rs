use crate::{
    Disk, DiskErr, Permissions,
    partition_tables::mbr::{MbrEntry, PartitionInfos, PartitionType, RawMbr},
    wrappers::{DiskWrapper, SubDisk},
};
use alloc::sync::Arc;

/// This struct allows to choose the sector size. It's always better explicitly specify the sector
/// size to use, if not specified, will try to use the smallest possible >= 512. For a physical
/// drive, it's always better to use the physical (or emulated) sector size (often 512B). Note that
/// MBR is designed to work with 512B sector size.
#[derive(Clone)]
pub struct GenericMbr {
    raw: RawMbr,
    disk: Arc<DiskWrapper>,
    sector_size: usize,
}

impl GenericMbr {
    /// This function creates a new MBR structure in memory (without writing it to the disk)
    pub fn new<T: Disk + 'static>(disk: T, sector_size: Option<usize>) -> Result<Self, DiskErr> {
        let sector_size = match sector_size {
            None => match disk.disk_infos()?.sector_size.minimal_ge(512) {
                None => return Err(DiskErr::UnsupportedDiskSectorSize),
                Some(v) => v,
            },
            Some(v) => v,
        };

        Ok(Self {
            raw: RawMbr {
                bootstrap: [0; 446],
                partitions: [MbrEntry::empty(); 4],
                signature: 0xAA55,
            },
            disk: DiskWrapper::new(disk),
            sector_size,
        })
    }

    /// Reads a MBR from the given disk.
    pub fn read_from_disk<T: Disk + 'static>(
        disk: T,
        sector_size: Option<usize>,
    ) -> Result<Option<Self>, DiskErr> {
        let sector_size = match sector_size {
            None => match disk.disk_infos()?.sector_size.minimal_ge(512) {
                None => return Err(DiskErr::UnsupportedDiskSectorSize),
                Some(v) => v,
            },
            Some(v) => v,
        };

        let raw = RawMbr::read_from_disk(&disk)?;
        if raw.signature == 0xAA55 {
            Ok(Some(Self {
                raw,
                disk: DiskWrapper::new(disk),
                sector_size,
            }))
        } else {
            Ok(None)
        }
    }

    /// Writes the MBR structure to the disk
    pub fn write(&self) -> Result<(), DiskErr> {
        self.raw.write_to_disk(&*self.disk)
    }

    /// Returnes the partition size (if the partition exists) in sectors
    pub fn partition_size(&self, partition_index: usize) -> Option<usize> {
        match self.raw.partitions.get(partition_index) {
            None => None,
            Some(v) => Some(v.sectors as usize),
        }
    }

    /// Returns the lba of the first sector of the partition (if it exists)
    pub fn partition_start(&self, partition_index: usize) -> Option<usize> {
        match self.raw.partitions.get(partition_index) {
            None => None,
            Some(v) => Some(v.lba_first as usize),
        }
    }

    pub fn partition_type(&self, partition_index: usize) -> Option<PartitionType> {
        match self.raw.partitions.get(partition_index) {
            None => None,
            Some(v) => Some(v.partition_type),
        }
    }

    pub fn partition_infos(&self, partition_index: usize) -> Option<PartitionInfos> {
        match self.raw.partitions.get(partition_index) {
            None => None,
            Some(v) => Some(PartitionInfos {
                lba_start: v.lba_first as usize,
                size: v.sectors as usize,
                sector_size: self.sector_size,
                partition_type: v.partition_type,
            }),
        }
    }

    pub const fn sector_size(&self) -> usize {
        self.sector_size
    }

    /// `start` and `size` are in sector (using self.sector_size)
    pub fn create_partition(
        &mut self,
        partition_index: usize,
        start: usize,
        size: usize,
        partition_type: PartitionType,
    ) -> Result<(), DiskErr> {
        if partition_index >= 4 {
            return Err(DiskErr::InvalidPartitionIndex);
        }

        let end = start + size;
        for p in self.raw.partitions {
            if ((p.lba_first as usize) <= start && start < (p.lba_first + p.sectors) as usize)
                || ((p.lba_first as usize) < end && end <= (p.lba_first + p.sectors) as usize)
            {
                return Err(DiskErr::SpaceAlreadyInUse);
            }
        }

        if start == 0 {
            return Err(DiskErr::SpaceAlreadyInUse);
        }

        let disk_size = self.disk.disk_infos()?.disk_size;
        if end > disk_size / self.sector_size {
            return Err(DiskErr::InvalidSectorIndex {
                found: end,
                max: disk_size / self.sector_size,
            });
        }

        let entry = MbrEntry {
            chs_last: [0; 3],
            chs_first: [0; 3],
            lba_first: start as u32,
            sectors: size as u32,
            status: 0x80,
            partition_type,
        };

        self.raw.partitions[partition_index] = entry;

        Ok(())
    }

    pub fn get_partition(
        &self,
        partition_index: usize,
        permissions: Permissions,
    ) -> Result<SubDisk, DiskErr> {
        match self.raw.partitions.get(partition_index) {
            None => Err(DiskErr::InvalidPartitionIndex),
            Some(partition) => self.disk.subdisk(
                (partition.lba_first as usize) * self.sector_size,
                ((partition.lba_first + partition.sectors) as usize) * self.sector_size,
                permissions,
            ),
        }
    }

    pub fn set_boot_code(&mut self, boot_code: [u8; 446]) {
        self.raw.bootstrap = boot_code
    }
}
