use std::path::PathBuf;

use filesystem::{Disk, DiskFile, Permission, SectorSize};

fn main() {
    let disk = DiskFile::new(
        PathBuf::from("disk.img"),
        1024,
        SectorSize::AllOf(&[512]),
        Permission::ReadWrite,
    )
    .unwrap();

    disk.write_sector(0, &[1; 512]).unwrap();

    let mut buf = [0; 512];
    disk.read_sector(0, &mut buf).unwrap();

    println!("{buf:?}")
}
