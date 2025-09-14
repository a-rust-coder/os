use crate::{
    Disk, DiskErr,
    filesystems::fat::bpb::{BiosParameterBlockCommon, ExtendedBpb12_16, FatType},
};
use alloc::{boxed::Box, vec};

pub struct Fat12Raw {
    bpb: BiosParameterBlockCommon,
    extended_bpb: ExtendedBpb12_16,
}

impl Fat12Raw {
    pub fn read_from_disk(disk: &dyn Disk) -> Result<Option<Self>, DiskErr> {
        let sector_size = match disk.disk_infos()?.sector_size.minimal_ge(512) {
            None => return Err(DiskErr::UnsupportedDiskSectorSize),
            Some(sector_size) => sector_size,
        };

        let mut sector = vec![0; sector_size];
        disk.read_sector(0, &mut sector)?;

        let mut bpb = [0; 36];
        bpb.copy_from_slice(&sector[..36]);
        let mut extended_bpb = [0; 476];
        extended_bpb.copy_from_slice(&sector[36..512]);

        let bpb = BiosParameterBlockCommon::from(bpb);
        let extended_bpb = ExtendedBpb12_16::from(extended_bpb);

        if !(bpb.is_valid()
            && extended_bpb.is_valid()
            && bpb.detect_fat_type() == Some(FatType::Fat12))
        {
            return Ok(None);
        }

        Ok(Some(Self { bpb, extended_bpb }))
    }

    pub fn write_to_disk(&self, disk: &dyn Disk) -> Result<(), DiskErr> {
        let sector_size = match disk.disk_infos()?.sector_size.minimal_ge(512) {
            None => return Err(DiskErr::UnsupportedDiskSectorSize),
            Some(sector_size) => sector_size,
        };

        let mut sector = vec![0; sector_size];

        let bpb: [u8; 36] = self.bpb.into();
        let extended_bpb: [u8; 476] = self.extended_bpb.into();

        sector[..36].copy_from_slice(&bpb);
        sector[36..512].copy_from_slice(&extended_bpb);

        disk.write_sector(0, &sector)
    }
}

pub struct Fat12 {
    raw: Fat12Raw,
    disk: Box<dyn Disk>,
    sector_size: usize,
}

impl Fat12 {
    pub fn read_from_disk(disk: Box<dyn Disk>) -> Result<Option<Self>, DiskErr> {
        let raw = match Fat12Raw::read_from_disk(&*disk)? {
            None => return Ok(None),
            Some(v) => v,
        };

        let sector_size = match disk.disk_infos()?.sector_size.minimal_ge(512) {
            None => return Err(DiskErr::UnsupportedDiskSectorSize),
            Some(v) => v,
        };

        Ok(Some(Self {
            raw,
            disk,
            sector_size,
        }))
    }

    pub fn new(
        disk: Box<dyn Disk>,
        number_of_fats: u8,
        hidden_sectors: usize,
        root_directory_entries: usize,
    ) -> Result<Self, DiskErr> {
        let disk_infos = disk.disk_infos()?;

        let sector_size = match disk_infos.sector_size.minimal_ge(512) {
            None => return Err(DiskErr::UnsupportedDiskSectorSize),
            Some(v) => v,
        };

        let total_sectors = disk_infos.disk_size / sector_size;
        let (total_sectors_16, total_sectors_32) = {
            if total_sectors <= u16::MAX as usize {
                (total_sectors as u16, 0)
            } else if total_sectors <= u32::MAX as usize {
                (0, total_sectors as u32)
            } else {
                return Err(DiskErr::InvalidDiskSize);
            }
        };

        let root_dir_sectors = (root_directory_entries * 32 + sector_size - 1) / sector_size;

        let mut sectors_per_cluster = total_sectors / 4085;
        sectors_per_cluster = match sectors_per_cluster {
            ..2 => 1,
            2 => 2,
            3..5 => 4,
            5..9 => 8,
            9..17 => 16,
            17..33 => 32,
            33..65 => 64,
            65..129 => 128,
            129.. => return Err(DiskErr::InvalidDiskSize),
        };

        let mut fat_size_16 =
            (((total_sectors - root_dir_sectors - 1) / sectors_per_cluster) * 12 + 8 * 512 - 1)
                / (12 * sector_size);
        let mut reserved_sectors_count = total_sectors
            - ((total_sectors - 1 - fat_size_16 * number_of_fats as usize) / sectors_per_cluster)
                * sectors_per_cluster;

        if reserved_sectors_count > total_sectors / 20 {
            sectors_per_cluster = sectors_per_cluster * 2;
            if sectors_per_cluster > 128 {
                sectors_per_cluster = 128
            }

            fat_size_16 =
                (((total_sectors - root_dir_sectors - 1) / sectors_per_cluster) * 12 + 8 * 512 - 1)
                    / (8 * sector_size);
            reserved_sectors_count = total_sectors
                - ((total_sectors - 1 - fat_size_16 * number_of_fats as usize)
                    / sectors_per_cluster)
                    * sectors_per_cluster;
        }

        let bpb = BiosParameterBlockCommon {
            jmp_boot: [0; 3],
            oem_name: [0; 8],
            bytes_per_sector: sector_size as u16,
            sectors_per_cluster: sectors_per_cluster as u8,
            reserved_sectors_count: reserved_sectors_count as u16,
            number_of_fats,
            root_directory_entries: root_directory_entries as u16,
            total_sectors_16,
            media: 0xF8,
            fat_size_16: fat_size_16 as u16,
            sectors_per_track: 0,
            number_of_heads: 0,
            hidden_sectors: hidden_sectors as u32,
            total_sectors_32,
        };

        let extended_bpb = ExtendedBpb12_16 {
            drive_number: 0x80,
            reserved: 0,
            boot_signature: 0x29,
            volume_serial_number: 0,
            volume_label: *b"NO NAME    ",
            file_system_type: *b"FAT12   ",
            boot_code: [0; 448],
            signature: 0xAA55,
        };

        let slf = Self {
            raw: Fat12Raw { bpb, extended_bpb },
            disk,
            sector_size,
        };
        slf.write()?;

        let sector = vec![0; sector_size];
        for f in 0..number_of_fats {
            for s in 0..fat_size_16 {
                let sector_index = reserved_sectors_count + s * f as usize;
                slf.disk.write_sector(sector_index, &sector)?;
            }
        }

        for s in 0..root_dir_sectors {
            let sector_index = reserved_sectors_count + s + fat_size_16 * number_of_fats as usize;
            slf.disk.write_sector(sector_index, &sector)?;
        }

        for s in 1..reserved_sectors_count {
            slf.disk.write_sector(s, &sector)?;
        }

        Ok(slf)
    }

    pub fn write(&self) -> Result<(), DiskErr> {
        self.raw.write_to_disk(&*self.disk)
    }
}
