use crate::{Disk, DiskErr, DiskInfos, Permission, SectorSize};
use alloc::{
    boxed::Box,
    sync::{Arc, Weak},
    vec::Vec,
};
use mutex::Mutex;

pub struct DiskWrapper {
    disk: Box<dyn Disk>,
    r_borrows: Mutex<Vec<(usize, usize)>>,
    w_borrows: Mutex<Vec<(usize, usize)>>,
    weak_self: Mutex<Weak<Self>>,
}

impl DiskWrapper {
    pub fn new(disk: Box<dyn Disk>) -> Arc<Self> {
        let slf = Arc::new(Self {
            disk,
            r_borrows: Mutex::new(Vec::new()),
            w_borrows: Mutex::new(Vec::new()),
            weak_self: Mutex::new(Weak::new()),
        });
        let weak = Arc::downgrade(&slf);
        *slf.weak_self.lock() = weak;
        slf
    }

    pub fn is_r_borrowed(&self, start: usize, end: usize) -> bool {
        for i in &*self.r_borrows.lock() {
            if (i.0 <= start && start < i.1) || (i.0 < end && end <= i.1) {
                return true;
            }
        }
        false
    }

    pub fn is_w_borrowed(&self, start: usize, end: usize) -> bool {
        for i in &*self.w_borrows.lock() {
            if (i.0 <= start && start < i.1) || (i.0 < end && end <= i.1) {
                return true;
            }
        }
        false
    }

    pub fn subdisk(
        &self,
        start: usize,
        end: usize,
        permission: Permission,
    ) -> Result<SubDisk, DiskErr> {
        if self.is_w_borrowed(start, end) || (self.is_r_borrowed(start, end) && permission.write) {
            return Err(DiskErr::Busy);
        }

        if end > self.disk.disk_infos()?.disk_size {
            return Err(DiskErr::InvalidDiskSize);
        }

        if permission.read {
            self.r_borrows.lock().push((start, end));
        }

        if permission.write {
            self.w_borrows.lock().push((start, end));
        }

        let sector_size = self.disk_infos()?.sector_size;
        let parent = self.weak_self.lock().clone();

        Ok(SubDisk {
            parent,
            start,
            end,
            sector_size,
            permission,
        })
    }
}

impl Disk for DiskWrapper {
    fn read_sector(&self, sector: usize, buf: &mut [u8]) -> Result<(), DiskErr> {
        let start = sector * buf.len();
        let end = start + buf.len();

        if self.is_w_borrowed(start, end) {
            return Err(DiskErr::Busy);
        }

        self.disk.read_sector(sector, buf)
    }

    fn write_sector(&self, sector: usize, buf: &[u8]) -> Result<(), DiskErr> {
        let start = sector * buf.len();
        let end = start + buf.len();

        if self.is_w_borrowed(start, end) || self.is_r_borrowed(start, end) {
            return Err(DiskErr::Busy);
        }

        self.disk.write_sector(sector, buf)
    }

    fn disk_infos(&self) -> Result<crate::DiskInfos, DiskErr> {
        self.disk.disk_infos()
    }
}

pub struct SubDisk {
    parent: Weak<DiskWrapper>,
    start: usize,
    end: usize,
    sector_size: SectorSize,
    permission: Permission,
}

impl Disk for SubDisk {
    fn read_sector(&self, sector: usize, buf: &mut [u8]) -> Result<(), DiskErr> {
        let parent = match self.parent.upgrade() {
            Some(v) => v,
            None => return Err(DiskErr::UnreachableDisk),
        };

        if !self.permission.read {
            return Err(DiskErr::InvalidPermission {
                disk_permission: self.permission,
            });
        }

        let sector_size = buf.len();

        if !self
            .sector_size
            .is_supported(sector_size, self.end - self.start)
            || self.start % sector_size != 0
        {
            return Err(DiskErr::InvalidSectorSize {
                found: sector_size,
                supported: self.sector_size,
                start: self.start,
            });
        }

        let offset = self.start + sector_size * sector;

        if offset >= self.end {
            return Err(DiskErr::InvalidSectorIndex {
                found: sector,
                max: (self.end - self.start) / sector_size,
            });
        }

        let sector = offset / sector_size;

        parent.disk.read_sector(sector, buf)
    }

    fn write_sector(&self, sector: usize, buf: &[u8]) -> Result<(), DiskErr> {
        let parent = match self.parent.upgrade() {
            Some(v) => v,
            None => return Err(DiskErr::UnreachableDisk),
        };

        if !self.permission.write {
            return Err(DiskErr::InvalidPermission {
                disk_permission: self.permission,
            });
        }

        let sector_size = buf.len();

        if !self
            .sector_size
            .is_supported(sector_size, self.end - self.start)
            || self.start % sector_size != 0
        {
            return Err(DiskErr::InvalidSectorSize {
                found: sector_size,
                supported: self.sector_size,
                start: self.start,
            });
        }

        let offset = self.start + sector_size * sector;

        if offset >= self.end {
            return Err(DiskErr::InvalidSectorIndex {
                found: sector,
                max: (self.end - self.start) / sector_size,
            });
        }

        let sector = offset / sector_size;

        parent.disk.write_sector(sector, buf)
    }

    fn disk_infos(&self) -> Result<DiskInfos, DiskErr> {
        Ok(DiskInfos {
            sector_size: self.sector_size,
            disk_size: self.end - self.start,
            permission: self.permission,
        })
    }
}

impl Drop for SubDisk {
    fn drop(&mut self) {
        if let Some(parent) = self.parent.upgrade() {
            if self.permission.read {
                let mut r_borrows = parent.r_borrows.lock();
                let idx = r_borrows
                    .iter()
                    .position(|&x| x == (self.start, self.end))
                    .unwrap();
                r_borrows.swap_remove(idx);
            }
            if self.permission.write {
                let mut w_borrows = parent.w_borrows.lock();
                let idx = w_borrows
                    .iter()
                    .position(|&x| x == (self.start, self.end))
                    .unwrap();
                w_borrows.swap_remove(idx);
            }
        }
    }
}
