pub enum Register {
    AX,
    CX,
    DX,
    BX,
    SP,
    BP,
    SI,
    DI,
}

impl Register {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0b000 => Some(Register::AX),
            0b001 => Some(Register::CX),
            0b010 => Some(Register::DX),
            0b011 => Some(Register::BX),
            0b100 => Some(Register::SP),
            0b101 => Some(Register::BP),
            0b110 => Some(Register::SI),
            0b111 => Some(Register::DI),
            _ => None,
        }
    }
}
