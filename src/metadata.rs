#[allow(unused)]
#[derive(Debug)]
pub struct Metadata {
    pub magic: [u8; 2],
    pub flags: u8,
    pub cpu: u8,
    pub hdr_len: u8,
    pub unused: u8,
    pub version: u16,
    pub text_size: usize,
    pub data_size: usize,
    pub bss_size: usize,
    pub entry: usize,
    pub total: usize,
    pub syms: usize,
}

impl Metadata {
    fn from_bytes(data: [u8; 32]) -> Self {
        Metadata {
            magic: [data[0], data[1]],
            flags: data[2],
            cpu: data[3],
            hdr_len: data[4],
            unused: data[5],
            version: u16::from_le_bytes([data[6], data[7]]),
            text_size: u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize,
            data_size: u32::from_le_bytes([data[12], data[13], data[14], data[15]]) as usize,
            bss_size: u32::from_le_bytes([data[16], data[17], data[18], data[19]]) as usize,
            entry: u32::from_le_bytes([data[20], data[21], data[22], data[23]]) as usize,
            total: u32::from_le_bytes([data[24], data[25], data[26], data[27]]) as usize,
            syms: u32::from_le_bytes([data[28], data[29], data[30], data[31]]) as usize,
        }
    }

    pub fn load_from_stream<R: std::io::Read>(mut stream: R) -> std::io::Result<Self> {
        let mut buffer = [0u8; 32];
        stream.read_exact(&mut buffer)?;
        Ok(Metadata::from_bytes(buffer))
    }
}
