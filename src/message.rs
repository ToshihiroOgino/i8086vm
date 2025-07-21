#[allow(unused)]
#[derive(Debug)]
pub struct Message {
    offset: usize,
    source: u16,
    pub message_type: u16,
}

#[derive(Debug)]
pub struct Detail1 {
    pub detail: [u16; 6],
}

pub const MESSAGE_SIZE: usize = 2 * 2;

impl Message {
    pub fn load(data: &[u8], offset: usize) -> Self {
        let source = u16::from_le_bytes([data[offset], data[offset + 1]]);
        let message_type = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
        Message {
            offset,
            source,
            message_type,
        }
    }

    pub fn load_detail1(&self, data: &[u8]) -> Detail1 {
        let mut detail = [0u16; 6];
        for i in 0..6 {
            detail[i] = u16::from_le_bytes([
                data[self.offset + MESSAGE_SIZE + i * 2],
                data[self.offset + MESSAGE_SIZE + i * 2 + 1],
            ]);
        }
        Detail1 { detail }
    }
}

#[allow(unused)]
impl Detail1 {
    pub fn m1i1(&self) -> u16 {
        self.detail[0]
    }
    pub fn m1i2(&self) -> u16 {
        self.detail[1]
    }
    pub fn m1i3(&self) -> u16 {
        self.detail[2]
    }
    pub fn m1p1(&self) -> u16 {
        self.detail[3]
    }
    pub fn m1p2(&self) -> u16 {
        self.detail[4]
    }
    pub fn m1p3(&self) -> u16 {
        self.detail[5]
    }
}
