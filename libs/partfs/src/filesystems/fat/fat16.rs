use crate::{
    Disk,
    filesystems::fat::bpb::{BiosParameterBlockCommon, ExtendedBpb12_16},
};
use alloc::boxed::Box;

pub struct Fat16Raw {
    bpb: BiosParameterBlockCommon,
    extended_bpb: ExtendedBpb12_16,
    // TODO:
}

pub struct Fat16 {
    raw: Fat16Raw,
    disk: Box<dyn Disk>,
}
