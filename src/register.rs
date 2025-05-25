use core::panic;
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

pub enum Register16Bit {
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

pub enum Register8Bit {
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

fn get_some_disp(disp: Option<u16>) -> u16 {
    match disp {
        Some(d) => d,
        None => panic!("Invalid displacement"),
    }
}

pub fn effective_address(rm: u8, mod_rm: u8, disp: Option<u16>, w: u8) -> String {
    let base = match rm {
        0b000 => "BX+SI",
        0b001 => "BX+DI",
        0b010 => "BP+SI",
        0b011 => "BP+DI",
        0b100 => "SI",
        0b101 => "DI",
        0b110 => "BP",
        0b111 => "BX",
        _ => panic!("Invalid effective address"),
    };
    match mod_rm {
        0b00 => {
            if rm == 0b110 {
                format!("[{disp:04x}]", disp = get_some_disp(disp))
            } else {
                format!("[{base}]")
            }
        }
        0b01 => {
            let disp_signed = get_some_disp(disp) as i8;
            if disp_signed >= 0 {
                format!("[{base}+{disp_signed:x}]")
            } else {
                format!("[{base}-{disp_signed:x}]", disp_signed = disp_signed.abs())
            }
        }
        0b10 => {
            let disp_signed = get_some_disp(disp) as i16;
            if disp_signed >= 0 {
                format!("[{base}+{disp:x}]", disp = disp_signed)
            } else {
                format!("[{base}-{disp:x}]", disp = disp_signed.abs())
            }
        }
        0b11 => format!("{reg}", reg = Register::new(rm, w)),
        _ => panic!("Invalid mod"),
    }
    .to_string()
}

pub fn calc_relative_disp(offset: usize, disp: Option<u16>, is_2byte_disp: bool) -> u16 {
    if offset > u16::MAX as usize {
        panic!("Offset is overflowing");
    }
    let offset = offset as u16;
    let signed_disp = match disp {
        Some(d) => {
            if is_2byte_disp {
                d as i16
            } else {
                (d as i8).into()
            }
        }
        None => panic!("Invalid displacement"),
    };
    if signed_disp >= 0 {
        offset + signed_disp as u16
    } else {
        offset - signed_disp.abs() as u16
    }
}

pub enum SegmentRegister {
    ES,
    CS,
    SS,
    DS,
}

impl SegmentRegister {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0b00 => SegmentRegister::ES,
            0b01 => SegmentRegister::CS,
            0b10 => SegmentRegister::SS,
            0b11 => SegmentRegister::DS,
            _ => panic!("Invalid segment register value"),
        }
    }
}

impl Display for SegmentRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            SegmentRegister::ES => "ES",
            SegmentRegister::CS => "CS",
            SegmentRegister::SS => "SS",
            SegmentRegister::DS => "DS",
        };
        write!(f, "{}", name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(0b000, 0b11, None, 0, "AL" ; "REG 1")]
    #[test_case(0b001, 0b11, None, 1, "CX" ; "REG 2")]
    #[test_case(0b100, 0b00, None, 0, "SI"; "No disp 1")]
    #[test_case(0b000, 0b00, None, 0, "[BX+SI]"; "No disp 2")]
    #[test_case(0b110, 0b01, Some(0xee), 0, "[BP-12]" ; "Sign-extended disp")]
    #[test_case(0b110, 0b10, Some(0x0f), 0, "[BP+f]" ; "r/m + Disp")]
    #[test_case(0b110, 0b00, Some(0x0f), 0, "[000f]" ; "only Disp")]
    #[test_case(0b110, 0b01, Some(0x04), 0, "[BP+4]" ; "mod=0b01")]
    fn test_effective_address(rm: u8, mod_rm: u8, disp: Option<u16>, w: u8, expected: &str) {
        let result = effective_address(rm, mod_rm, disp, w);
        assert_eq!(result, expected);
    }
}
