use alloc::{boxed::Box, string::String, vec::Vec};

use crate::{Disk, DiskErr};

pub mod fat12;

pub trait FileSystem {
    fn get_file(&self, path: String) -> Result<Option<File>, DiskErr>;

    fn create_file(&self, path: String, file: File) -> Result<(), DiskErr>;

    fn get_dir(&self, path: String) -> Result<Option<Directory>, DiskErr>;

    fn create_dir(&self, path: String, dir: Directory) -> Result<(), DiskErr>;

    fn remove_file(&self, path: String) -> Result<(), DiskErr>;

    fn remove_dir(&self, path: String) -> Result<(), DiskErr>;

    fn move_file(&self, from: String, to: String) -> Result<(), DiskErr>;
    
    fn move_dir(&self, from: String, to: String) -> Result<(), DiskErr>;
}

pub struct File {
    pub name: String,
    pub size: usize,
    pub content: Box<dyn Disk>,
    pub attributes: Attributes,
}

pub struct Attributes {
    pub read: Option<bool>,
    pub write: Option<bool>,
    pub hidden: Option<bool>,
    pub system: Option<bool>,
}

pub struct Directory {
    pub name: String,
    pub attributes: Attributes,
    pub entries: Vec<DirectoryEntry>,
}

pub struct DirectoryEntry {
    pub name: String,
    pub size: usize,
    pub attributes: Attributes,
}
