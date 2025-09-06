use std::{fs::remove_file, path::PathBuf};

use filesystem::{Disk, DiskFile, Permission, SectorSize, wrappers::DiskWrapper};

fn main() {
    let _ = remove_file("disk.img");

    let disk = DiskFile::new(
        PathBuf::from("disk.img"),
        2048,
        SectorSize::AllOf(&[512, 1024]),
        Permission {
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
            Permission {
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
            Permission {
                read: true,
                write: true,
            },
        )
        .unwrap();

}
