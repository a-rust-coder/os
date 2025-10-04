use crate::{Disk, DiskErr, filesystems::fat12::bpb::BiosParameterBlock, wrappers::DiskWrapper};
use alloc::{sync::Arc, vec};

pub mod bpb;

pub struct Fat12 {
    bpb: BiosParameterBlock,
    disk: Arc<DiskWrapper>,
    sector_size: usize,
}

impl Fat12 {
    pub fn read_from_disk<T: Disk + 'static>(
        disk: T,
        sector_size: Option<usize>,
    ) -> Result<Option<Self>, DiskErr> {
        let disk = DiskWrapper::new(disk);
        let sector_size = sector_size.unwrap_or_else(|| {
            let infos = match disk.disk_infos() {
                Ok(v) => v,
                Err(_) => return 0,
            };
            for i in [512, 1024, 2048, 4096] {
                if infos.sector_size.is_supported(i, infos.disk_size) {
                    return i;
                }
            }
            0
        });

        if sector_size < 512 || sector_size.count_ones() != 1 {
            return Err(DiskErr::UnsupportedDiskSectorSize);
        }

        let mut first_sector = vec![0; sector_size];
        disk.read_sector(0, &mut first_sector)?;

        let mut bs = [0; 512];
        bs.copy_from_slice(&first_sector[..512]);

        let bpb = BiosParameterBlock::from_bytes(bs);

        if !bpb.is_valid() || bpb.bytes_per_sector() != sector_size {
            return Ok(None);
        }

        Ok(Some(Self {
            bpb,
            disk,
            sector_size,
        }))
    }

    pub fn new<T: Disk + 'static>(
        disk: T,
        root_dir_entries: usize,
        number_of_fats: usize,
        hidden_sectors: usize,
        sector_size: Option<usize>,
        sectors_per_cluster: Option<usize>,
    ) -> Result<Option<Self>, DiskErr> {
        let disk = DiskWrapper::new(disk);
        let disk_infos = disk.disk_infos()?;

        let sector_size = sector_size.unwrap_or_else(|| {
            for i in [512, 1024, 2048, 4096] {
                if disk_infos.sector_size.is_supported(i, disk_infos.disk_size) {
                    return i;
                }
            }
            0
        });

        if sector_size < 512
            || sector_size.count_ones() != 1
            || sector_size > 0xFFFF
            || (root_dir_entries * 32) % sector_size != 0
            || number_of_fats > 0xFF
            || root_dir_entries > 0xFFFF
        {
            return Ok(None);
        }

        let root_dir_sectors = (root_dir_entries * 32) / sector_size;
        let total_sectors = disk_infos.disk_size / sector_size;

        let sectors_per_cluster = match sectors_per_cluster {
            Some(v) => v,
            None => {
                // TODO: optimize the `sectors_per_cluster` choice to get the most possible sectors
                // with fewer reserved sectors
                ((total_sectors - root_dir_sectors - 1 + 4084) / 4085).next_power_of_two()
            }
        };

        if sectors_per_cluster.count_ones() != 1
            || sectors_per_cluster > 0xFF
            || total_sectors > 0xFFFFFFFF
        {
            return Ok(None);
        }

        let mut count_of_clusters = (total_sectors - root_dir_sectors - 1) / sectors_per_cluster;
        let fat_size = (count_of_clusters + count_of_clusters / 2 + sector_size - 1) / sector_size;
        count_of_clusters = (total_sectors - root_dir_sectors - fat_size * number_of_fats - 1)
            / sectors_per_cluster;
        let reserved_sectors = total_sectors
            - count_of_clusters * sectors_per_cluster
            - fat_size * number_of_fats
            - root_dir_sectors;

        let (total_sectors_16, total_sectors_32) = if total_sectors < 0x10000 {
            (total_sectors as u16, 0)
        } else {
            (0, total_sectors as u32)
        };

        let bpb = BiosParameterBlock {
            jmp_boot: [0xEB, 0xFE, 0x90],
            oem_name: [0; 8],
            bytes_per_sector: sector_size as u16,
            sectors_per_cluster: sectors_per_cluster as u8,
            reserved_sectors_count: reserved_sectors as u16,
            number_of_fats: number_of_fats as u8,
            root_entries_count: root_dir_entries as u16,
            total_sectors_16,
            media: 0xF8,
            fat_size: fat_size as u16,
            sectors_per_track: 0,
            number_of_heads: 0,
            total_sectors_32,
            hidden_sectors: hidden_sectors as u32,
            drive_number: 0x80,
            _reserved0: 0,
            boot_signature: 0x29,
            volume_id: 0,
            volume_label: *b"NO NAME    ",
            fs_type: *b"FAT12   ",
            boot_code: [0; 448],
            signature: 0xAA55,
        };

        if !bpb.is_valid() {
            return Ok(None);
        }

        let bytes = bpb.to_bytes();
        let mut sector = vec![0; sector_size];

        for i in 0..(reserved_sectors + number_of_fats * fat_size + root_dir_sectors) {
            disk.write_sector(i, &sector)?;
        }

        sector[..512].copy_from_slice(&bytes);
        disk.write_sector(0, &sector)?;

        Ok(Some(Self {
            bpb,
            disk,
            sector_size,
        }))
    }

    pub fn bios_parameter_block(&self) -> BiosParameterBlock {
        self.bpb
    }
}
