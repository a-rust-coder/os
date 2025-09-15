pub mod bpb;
pub mod fat12;
pub mod fat16;
pub mod fat32;

#[repr(C, packed)]
pub struct DirEntry {
    name: [u8; 11],
    attributes: u8,
    reserved: u8,
    creation_time_tenth: u8,
    creation_time: u16,
    creation_date: u16,
    last_access_date: u16,
    first_cluster_high: u16,
    write_time: u16,
    write_date: u16,
    first_cluster_low: u16,
    file_size: u32,
}

impl Into<[u8; 32]> for DirEntry {
    fn into(self) -> [u8; 32] {
        let mut out = [0; 32];

        out[..11].copy_from_slice(&self.name);
        out[11] = self.attributes;
        out[12] = self.reserved;
        out[13] = self.creation_time_tenth;
        out[14..16].copy_from_slice(&self.creation_time.to_le_bytes());
        out[16..18].copy_from_slice(&self.creation_date.to_le_bytes());
        out[18..20].copy_from_slice(&self.last_access_date.to_le_bytes());
        out[20..22].copy_from_slice(&self.first_cluster_high.to_le_bytes());
        out[22..24].copy_from_slice(&self.write_time.to_le_bytes());
        out[24..26].copy_from_slice(&self.write_date.to_le_bytes());
        out[26..28].copy_from_slice(&self.first_cluster_low.to_le_bytes());
        out[28..].copy_from_slice(&self.file_size.to_le_bytes());

        out
    }
}

impl From<[u8; 32]> for DirEntry {
    fn from(value: [u8; 32]) -> Self {
        Self {
            name: {
                let mut name = [0; 11];
                name.copy_from_slice(&value[..11]);
                name
            },
            attributes: value[11],
            reserved: value[12],
            creation_time_tenth: value[13],
            creation_time: u16::from_le_bytes([value[14], value[15]]),
            creation_date: u16::from_le_bytes([value[16], value[17]]),
            last_access_date: u16::from_le_bytes([value[18], value[19]]),
            first_cluster_high: u16::from_le_bytes([value[20], value[21]]),
            write_time: u16::from_le_bytes([value[22], value[23]]),
            write_date: u16::from_le_bytes([value[24], value[25]]),
            first_cluster_low: u16::from_le_bytes([value[26], value[27]]),
            file_size: u32::from_le_bytes([value[28], value[29], value[30], value[32]]),
        }
    }
}

impl DirEntry {
    pub fn cluster_value(&self) -> u32 {
        (self.first_cluster_high as u32) | ((self.first_cluster_low as u32) << 16)
    }
}
