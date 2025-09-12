use crate::{
    Disk,
    filesystems::fat::bpb::{BiosParameterBlockCommon, ExtendedBpb32},
};
use alloc::boxed::Box;

pub struct Fat32Raw {
    bpb: BiosParameterBlockCommon,
    extended_bpb: ExtendedBpb32,
    // TODO:
}

pub struct Fat32 {
    raw: Fat32Raw,
    disk: Box<dyn Disk>,
}
