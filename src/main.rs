use std::{fs::remove_file, path::PathBuf};

use partfs::{
    DiskFile, Permissions, SectorSize,
    partition_tables::mbr::{generic_mbr::GenericMbr, partition_types},
};

fn main() {
    let _ = remove_file("target/disk.img");

    let disk = DiskFile::new(
        PathBuf::from("target/disk.img"),
        1024 * 1024,
        SectorSize::AllOf(&[512]),
        Permissions {
            read: true,
            write: true,
        },
    )
    .unwrap();

    let mut mbr = GenericMbr::new(Box::new(disk), None).unwrap();

    mbr.create_partition(0, 1, 1024, partition_types::EMPTY)
        .unwrap();
    mbr.create_partition(1, 1025, 1022, partition_types::EXFAT)
        .unwrap();

    mbr.write().unwrap();
}
