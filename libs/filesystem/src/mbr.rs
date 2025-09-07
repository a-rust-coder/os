use crate::{Disk, DiskErr};
use alloc::{boxed::Box, vec};

/// This struct is identity mapped to the disk. It's not intented for direct use. It follows the
/// canonical MBR format.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawMbr {
    bootstrap: [u8; 446],
    partitions: [MbrEntry; 4],
    signature: u16,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MbrEntry {
    status: u8,
    chs_first: [u8; 3],
    partition_type: u8,
    chs_last: [u8; 3],
    lba_first: u32,
    sectors: u32,
}

impl RawMbr {
    pub fn write_to_disk(&self, disk: &dyn Disk) -> Result<(), DiskErr> {
        if let Some(sector_size) = disk.disk_infos()?.sector_size.minimal_ge(512) {
            let mut sector = vec![0; sector_size];
            sector[..512].copy_from_slice(&self.to_bytes());
            disk.write_sector(0, &sector)
        } else {
            Err(DiskErr::UnsupportedDiskSectorSize)
        }
    }

    pub fn read_from_disk(&self, disk: &dyn Disk) -> Result<Self, DiskErr> {
        if let Some(sector_size) = disk.disk_infos()?.sector_size.minimal_ge(512) {
            let mut sector = vec![0; sector_size];
            disk.read_sector(0, &mut sector)?;
            Ok(Self::from_bytes(&sector).unwrap())
        } else {
            Err(DiskErr::UnsupportedDiskSectorSize)
        }
    }

    pub fn to_bytes(&self) -> [u8; 512] {
        let mut buf = [0u8; 512];

        buf[..446].copy_from_slice(&self.bootstrap);

        for (i, part) in self.partitions.iter().enumerate() {
            let offset = 446 + i * 16;
            part.write_to(&mut buf[offset..offset + 16]);
        }

        buf[510] = (self.signature & 0x00FF) as u8;
        buf[511] = (self.signature >> 8) as u8;

        buf
    }

    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() < 512 {
            return None;
        }

        let mut bootstrap = [0u8; 446];

        bootstrap.copy_from_slice(&buf[..446]);

        let mut partitions = [MbrEntry::empty(); 4];

        for i in 0..4 {
            let offset = 446 + i * 16;
            partitions[i] = MbrEntry::read_from(&buf[offset..offset + 16]);
        }

        let signature = u16::from_le_bytes([buf[510], buf[511]]);

        Some(Self {
            bootstrap,
            partitions,
            signature,
        })
    }
}

impl MbrEntry {
    fn empty() -> Self {
        Self {
            status: 0,
            chs_first: [0; 3],
            partition_type: 0,
            chs_last: [0; 3],
            lba_first: 0,
            sectors: 0,
        }
    }

    fn write_to(&self, buf: &mut [u8]) {
        buf[0] = self.status;
        buf[1..4].copy_from_slice(&self.chs_first);
        buf[4] = self.partition_type;
        buf[5..8].copy_from_slice(&self.chs_last);
        buf[8..12].copy_from_slice(&self.lba_first.to_le_bytes());
        buf[12..16].copy_from_slice(&self.sectors.to_le_bytes());
    }

    fn read_from(buf: &[u8]) -> Self {
        Self {
            status: buf[0],
            chs_first: [buf[1], buf[2], buf[3]],
            partition_type: buf[4],
            chs_last: [buf[5], buf[6], buf[7]],
            lba_first: u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]),
            sectors: u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]),
        }
    }
}
