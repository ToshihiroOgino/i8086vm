pub struct Message {
    offset: usize,
    source: u16,
    pub message_type: u16,
}

pub struct Detail1 {
    pub detail: [u16; 6],
}

const MESSAGE_SIZE: usize = 2 * 2;
const DETAIL1_SIZE: usize = 2 * 6;

impl Message {
    pub fn load(data_memory: &[u8], offset: usize) -> Self {
        if offset + MESSAGE_SIZE > data_memory.len() {
            panic!("Memory access out of bounds at offset {}", offset);
        }
        let source = u16::from_le_bytes([data_memory[offset], data_memory[offset + 1]]);
        let message_type = u16::from_le_bytes([data_memory[offset + 2], data_memory[offset + 3]]);
        Message {
            offset,
            source,
            message_type,
        }
    }

    pub fn load_detail1(&self, data_memory: &[u8]) -> Detail1 {
        if self.offset + MESSAGE_SIZE + DETAIL1_SIZE > data_memory.len() {
            panic!(
                "Memory access out of bounds at offset 0x{:04x}",
                self.offset
            );
        }
        let mut detail = [0u16; 6];
        for i in 0..6 {
            detail[i] = u16::from_le_bytes([
                data_memory[self.offset + MESSAGE_SIZE + i * 2],
                data_memory[self.offset + MESSAGE_SIZE + i * 2 + 1],
            ]);
        }
        Detail1 { detail }
    }
}

#[allow(dead_code)]
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
