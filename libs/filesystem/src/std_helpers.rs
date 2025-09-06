use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

use mutex::Mutex;

use crate::{Disk, DiskErr, DiskInfos, Permission, SectorSize};

#[derive(Debug)]
pub struct DiskFile {
    sector_size: SectorSize,
    /// In bytes
    size: usize,
    perm: Permission,
    file: Mutex<File>,
}

impl Disk for DiskFile {
    fn read_sector(&self, sector: usize, buf: &mut [u8]) -> Result<(), DiskErr> {
        if self.perm == Permission::WriteOnly {
            return Err(DiskErr::InvalidPermission {
                disk_permission: self.perm,
            });
        }

        let sector_size = buf.len();

        if !self.sector_size.is_supported(sector_size, self.size) {
            return Err(DiskErr::InvalidSectorSize {
                found: sector_size,
                supported: self.sector_size,
            });
        }

        let offset = sector_size * sector;

        if offset + sector_size > self.size {
            return Err(DiskErr::InvalidSectorIndex {
                found: sector,
                max: self.size / sector_size,
            });
        }

        if let Err(_) = self.file.lock().seek(SeekFrom::Start(offset as u64)) {
            return Err(DiskErr::IOErr);
        }

        if let Err(_) = self.file.lock().read_exact(buf) {
            return Err(DiskErr::IOErr);
        }

        Ok(())
    }

    fn write_sector(&self, sector: usize, buf: &[u8]) -> Result<(), DiskErr> {
        if self.perm == Permission::ReadOnly {
            return Err(DiskErr::InvalidPermission {
                disk_permission: self.perm,
            });
        }

        let sector_size = buf.len();

        if !self.sector_size.is_supported(sector_size, self.size) {
            return Err(DiskErr::InvalidSectorSize {
                found: sector_size,
                supported: self.sector_size,
            });
        }

        let offset = sector_size * sector;

        if offset + sector_size > self.size {
            return Err(DiskErr::InvalidSectorIndex {
                found: sector,
                max: self.size / sector_size,
            });
        }

        if let Err(_) = self.file.lock().seek(SeekFrom::Start(offset as u64)) {
            return Err(DiskErr::IOErr);
        }

        if let Err(_) = self.file.lock().write_all(buf) {
            return Err(DiskErr::IOErr);
        }

        Ok(())
    }

    fn disk_infos(&self) -> Result<DiskInfos, DiskErr> {
        Ok(DiskInfos {
            sector_sizes: self.sector_size,
            disk_size: self.size,
            permission: self.perm,
        })
    }
}

impl DiskFile {
    pub fn new(
        path: PathBuf,
        size: usize,
        sector_conf: SectorSize,
        permission: Permission,
    ) -> io::Result<Self> {
        let file = File::create_new(path)?;
        file.set_len(size as u64)?;

        io::Result::Ok(Self {
            file: Mutex::new(file),
            sector_size: sector_conf,
            size,
            perm: permission,
        })
    }

    pub fn from_file(
        file: PathBuf,
        sector_conf: SectorSize,
        permission: Permission,
    ) -> io::Result<Self> {
        let file = match permission {
            Permission::ReadOnly => File::options()
                .create_new(false)
                .read(true)
                .write(false)
                .open(file)?,
            Permission::WriteOnly => File::options()
                .create_new(false)
                .read(false)
                .write(true)
                .open(file)?,
            Permission::ReadWrite => File::options()
                .create_new(false)
                .read(true)
                .write(true)
                .open(file)?,
        };

        let size = file.metadata()?.len() as usize;

        Ok(Self {
            sector_size: sector_conf,
            size,
            perm: permission,
            file: Mutex::new(file),
        })
    }
}
