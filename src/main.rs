use partfs::{
    Disk, DiskFile, Permissions, SectorSize,
    filesystems::fat::{
        bpb::{BiosParameterBlockCommon, ExtendedBpb12_16, ExtendedBpb32, FatType},
        fat12::Fat12,
    },
    partition_tables::mbr::{generic_mbr::GenericMbr, partition_types},
};
use std::{fs::remove_file, path::PathBuf, u16, vec};

fn main() {
    let create = false;

    if create {
        let _ = remove_file("target/disk.img");

        let disk = DiskFile::new(
            PathBuf::from("target/disk.img"),
            1024 * 1024 * 1024,
            SectorSize::AllOf(vec![512]),
            Permissions::read_write(),
        )
        .unwrap();

        let mut mbr = GenericMbr::new(Box::new(disk), None).unwrap();

        mbr.create_partition(0, 1, 1024 * 1024 / 512, partition_types::FAT12_PRIMARY)
            .unwrap();
        mbr.create_partition(
            1,
            (1024 * 1024 / 512) + 1,
            100 * 1024 * 1024 / 512,
            partition_types::FAT16_PRIMARY,
        )
        .unwrap();
        mbr.create_partition(
            2,
            (1024 * 1024 / 512) + (100 * 1024 * 1024 / 512) + 1,
            800 * 1024 * 1024 / 512,
            partition_types::FAT32_LBA,
        )
        .unwrap();

        mbr.write().unwrap();

        return;
    }

    let disk = DiskFile::from_file(
        PathBuf::from("target/disk.img"),
        SectorSize::AllOf(vec![512]),
        Permissions {
            read: true,
            write: true,
        },
    )
    .unwrap();

    let mbr = GenericMbr::read_from_disk(Box::new(disk), None)
        .unwrap()
        .unwrap();

    let part1 = mbr.get_partition(0, Permissions::read_write()).unwrap();
    let part2 = mbr.get_partition(1, Permissions::read_only()).unwrap();
    let part3 = mbr.get_partition(2, Permissions::read_only()).unwrap();

    let fat12 = Fat12::read_from_disk(Box::new(part1)).unwrap().unwrap();

    let mut i = 0;
    while let Ok(v) = fat12.get_fat_entry(i)
        && i < 10
    {
        i += 1;
        println!("{}", v)
    }

    let mut i = 0;
    while let Ok(v) = fat12.get_root_dir_entry(i)
        && i < 3
    {
        i += 1;
        println!("{:?}", v);

        let cluster = v.cluster_value();

        if cluster > 0 && cluster < u16::MAX as usize {
            let (size, file) = fat12.get_file(cluster, Permissions::read_write()).unwrap();
            let mut sector = [0; 512];

            for i in 0..(size + 511) / 512 {
                file.read_sector(i, &mut sector).unwrap();
                println!("{}", unsafe {
                    String::from_utf8_unchecked(sector.to_vec())
                });
            }
        }
    }

    println!("{:?}", fat12.find_free_clusters(10).unwrap());

    drop(fat12);

    let part1 = mbr.get_partition(0, Permissions::read_only()).unwrap();

    for (i, part) in [part1, part2, part3].iter().enumerate() {
        println!("\nPartition number {}\n==================\n", i);

        let mut bpb = [0; 512];
        part.read_sector(0, &mut bpb).unwrap();

        let mut bpb_common = [0; 36];
        bpb_common.copy_from_slice(&bpb[0..36]);
        let bpb_common = BiosParameterBlockCommon::from(bpb_common);

        let mut ext_bpb = [0; 476];
        ext_bpb.copy_from_slice(&bpb[36..]);

        println!(
            "Bios Parameter Block Common is valid: {}",
            bpb_common.is_valid()
        );

        println!("{:?}", bpb_common);

        let fat_type = bpb_common.detect_fat_type().unwrap();
        println!("Detected FAT type: {:?}", fat_type);

        match fat_type {
            FatType::Fat12 => {
                let ext_bpb = ExtendedBpb12_16::from(ext_bpb);
                println!("FAT12 Extended BPB is valid: {}", ext_bpb.is_valid());
            }
            FatType::Fat16 => {
                let ext_bpb = ExtendedBpb12_16::from(ext_bpb);
                println!("FAT16 Extended BPB is valid: {}", ext_bpb.is_valid());
            }
            FatType::Fat32 => {
                let ext_bpb = ExtendedBpb32::from(ext_bpb);
                println!("FAT32 Extended BPB is valid: {}", ext_bpb.is_valid());
            }
        }
    }
}
