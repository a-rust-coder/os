use std::{fs::remove_file, path::PathBuf};

use filesystem::{Disk, DiskFile, Permissions, SectorSize, wrappers::DiskWrapper};

fn main() {
    let _ = remove_file("target/disk.img");

    let disk = DiskFile::new(
        PathBuf::from("target/disk.img"),
        2048,
        SectorSize::AllOf(&[512, 1024]),
        Permissions {
            read: true,
            write: true,
        },
    )
    .unwrap();

    disk.write_sector(0, &[1; 1024]).unwrap();

    let disk = DiskWrapper::new(Box::new(disk));

    disk.write_sector(0, &[2; 512]).unwrap();

    let subdisk = disk
        .subdisk(
            512,
            2048,
            Permissions {
                read: true,
                write: true,
            },
        )
        .unwrap();
    drop(subdisk);

    let subdisk2 = disk
        .subdisk(
            1024,
            2048,
            Permissions {
                read: true,
                write: true,
            },
        )
        .unwrap();

}
