use std::fmt::Display;

pub enum Register {
    Word(Register16Bit),
    Byte(Register8Bit),
}

impl Register {
    pub fn new(reg: u8, w: u8) -> Self {
        match w {
            0 => Register::Byte(Register8Bit::from_u8(reg)),
            1 => Register::Word(Register16Bit::from_u8(reg)),
            _ => panic!("Invalid register size"),
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Register::Word(reg) => write!(f, "{}", reg),
            Register::Byte(reg) => write!(f, "{}", reg),
        }
    }
}

enum Register16Bit {
    AX,
    CX,
    DX,
    BX,
    SP,
    BP,
    SI,
    DI,
}

impl Register16Bit {
    fn from_u8(value: u8) -> Self {
        match value {
            0b000 => Register16Bit::AX,
            0b001 => Register16Bit::CX,
            0b010 => Register16Bit::DX,
            0b011 => Register16Bit::BX,
            0b100 => Register16Bit::SP,
            0b101 => Register16Bit::BP,
            0b110 => Register16Bit::SI,
            0b111 => Register16Bit::DI,
            _ => panic!("Invalid register value"),
        }
    }
}

impl Display for Register16Bit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Register16Bit::AX => "AX",
            Register16Bit::CX => "CX",
            Register16Bit::DX => "DX",
            Register16Bit::BX => "BX",
            Register16Bit::SP => "SP",
            Register16Bit::BP => "BP",
            Register16Bit::SI => "SI",
            Register16Bit::DI => "DI",
        };
        write!(f, "{}", name)
    }
}

enum Register8Bit {
    AL,
    CL,
    DL,
    BL,
    AH,
    CH,
    DH,
    BH,
}

impl Register8Bit {
    fn from_u8(value: u8) -> Self {
        match value {
            0b000 => Register8Bit::AL,
            0b001 => Register8Bit::CL,
            0b010 => Register8Bit::DL,
            0b011 => Register8Bit::BL,
            0b100 => Register8Bit::AH,
            0b101 => Register8Bit::CH,
            0b110 => Register8Bit::DH,
            0b111 => Register8Bit::BH,
            _ => panic!("Invalid register value"),
        }
    }
}

impl Display for Register8Bit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Register8Bit::AL => "AL",
            Register8Bit::CL => "CL",
            Register8Bit::DL => "DL",
            Register8Bit::BL => "BL",
            Register8Bit::AH => "AH",
            Register8Bit::CH => "CH",
            Register8Bit::DH => "DH",
            Register8Bit::BH => "BH",
        };
        write!(f, "{}", name)
    }
}

pub fn effective_address(rm: u8) -> String {
    match rm {
        0b000 => "BX+SI",
        0b001 => "BX+DI",
        0b010 => "BP+SI",
        0b011 => "BP+DI",
        0b100 => "SI",
        0b101 => "DI",
        0b110 => "BX",
        0b111 => "BP",
        _ => panic!("Invalid effective address"),
    }
    .to_string()
}
