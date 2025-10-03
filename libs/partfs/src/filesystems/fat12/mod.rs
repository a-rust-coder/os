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
}
