use crate::{
    Disk,
    filesystems::fat::bpb::{BiosParameterBlockCommon, ExtendedBpb12_16},
};
use alloc::boxed::Box;

pub struct Fat12Raw {
    bpb: BiosParameterBlockCommon,
    extended_bpb: ExtendedBpb12_16,
    // TODO:
}

pub struct Fat12 {
    raw: Fat12Raw,
    disk: Box<dyn Disk>,
}
