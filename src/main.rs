use std::fs::remove_file;

use partfs::{filesystems::fat12::Fat12, DiskFile, Permissions, SectorSize};

fn main() {
    let _ = remove_file("target/disk.img");
    let disk = DiskFile::new(
        "target/disk.img".into(),
        1024 * 1024 * 4 + 14,
        SectorSize::AllOf(vec![512]),
        Permissions::read_write(),
    )
    .unwrap();

    let fat12 = Fat12::new(disk, 512, 2, 0, None, None).unwrap().unwrap();

    println!("{:?}", fat12.bios_parameter_block());
}
