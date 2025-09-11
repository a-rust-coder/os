use std::{fs::remove_file, path::PathBuf};

use partfs::{
    Disk, DiskFile, Permissions, SectorSize,
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

    drop(mbr);

    let disk = DiskFile::from_file(
        "target/disk.img".into(),
        SectorSize::AllOf(&[512]),
        Permissions::read_write(),
    )
    .unwrap();

    let mbr = GenericMbr::read_from_disk(Box::new(disk), None).unwrap().unwrap();

    println!("{:?}", mbr.partition_infos(0).unwrap());
    println!("{:?}", mbr.partition_infos(1).unwrap());
}
