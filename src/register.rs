use core::panic;
use std::fmt::Display;

#[derive(Debug, Default)]
pub struct Register {
    // General purpose registers
    pub al: u8,
    pub ah: u8,
    pub cl: u8,
    pub ch: u8,
    pub dl: u8,
    pub dh: u8,
    pub bl: u8,
    pub bh: u8,

    // 16-bit registers
    pub sp: u16,
    pub bp: u16,
    pub si: u16,
    pub di: u16,

    // Segment registers
    pub es: u16,
    pub cs: u16,
    pub ss: u16,
    pub ds: u16,
    // Instruction Pointer
    pub ip: u16,
}

#[allow(unused)]
impl Register {
    pub fn new() -> Self {
        Register::default()
    }

    pub fn get(&self, reg: RegisterType) -> u16 {
        match reg {
            RegisterType::Word(r) => match r {
                Register16Bit::AX => self.get_ax(),
                Register16Bit::CX => self.get_cx(),
                Register16Bit::DX => self.get_dx(),
                Register16Bit::BX => self.get_bx(),
                Register16Bit::SP => self.sp,
                Register16Bit::BP => self.bp,
                Register16Bit::SI => self.si,
                Register16Bit::DI => self.di,
            },
            RegisterType::Byte(r) => match r {
                Register8Bit::AL => self.al,
                Register8Bit::CL => self.cl,
                Register8Bit::DL => self.dl,
                Register8Bit::BL => self.bl,
                Register8Bit::AH => self.ah,
                Register8Bit::CH => self.ch,
                Register8Bit::DH => self.dh,
                Register8Bit::BH => self.bh,
            }
            .try_into()
            .expect("Invalid byte register conversion"),
            RegisterType::Segment(seg) => match seg {
                SegmentRegister::ES => self.es,
                SegmentRegister::CS => self.cs,
                SegmentRegister::SS => self.ss,
                SegmentRegister::DS => self.ds,
            },
        }
    }

    pub fn set(&mut self, reg: RegisterType, value: u16) {
        match reg {
            RegisterType::Word(r) => match r {
                Register16Bit::AX => self.set_ax(value),
                Register16Bit::CX => self.set_cx(value),
                Register16Bit::DX => self.set_dx(value),
                Register16Bit::BX => self.set_bx(value),
                Register16Bit::SP => self.sp = value,
                Register16Bit::BP => self.bp = value,
                Register16Bit::SI => self.si = value,
                Register16Bit::DI => self.di = value,
            },
            RegisterType::Byte(r) => {
                let value = value as u8;
                match r {
                    Register8Bit::AL => self.al = value,
                    Register8Bit::CL => self.cl = value,
                    Register8Bit::DL => self.dl = value,
                    Register8Bit::BL => self.bl = value,
                    Register8Bit::AH => self.ah = value,
                    Register8Bit::CH => self.ch = value,
                    Register8Bit::DH => self.dh = value,
                    Register8Bit::BH => self.bh = value,
                }
            }
            RegisterType::Segment(seg) => match seg {
                SegmentRegister::ES => self.es = value,
                SegmentRegister::CS => self.cs = value,
                SegmentRegister::SS => self.ss = value,
                SegmentRegister::DS => self.ds = value,
            },
        }
    }

    pub fn get_ax(&self) -> u16 {
        u16::from_le_bytes([self.al, self.ah])
    }
    pub fn set_ax(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.al = bytes[0];
        self.ah = bytes[1];
    }

    pub fn get_cx(&self) -> u16 {
        u16::from_le_bytes([self.cl, self.ch])
    }
    pub fn set_cx(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.cl = bytes[0];
        self.ch = bytes[1];
    }

    pub fn get_dx(&self) -> u16 {
        u16::from_le_bytes([self.dl, self.dh])
    }
    pub fn set_dx(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.dl = bytes[0];
        self.dh = bytes[1];
    }

    pub fn get_bx(&self) -> u16 {
        u16::from_le_bytes([self.bl, self.bh])
    }
    pub fn set_bx(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.bl = bytes[0];
        self.bh = bytes[1];
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum RegisterType {
    Word(Register16Bit),
    Byte(Register8Bit),
    Segment(SegmentRegister),
}

impl RegisterType {
    pub fn new(reg: u8, w: u8) -> Self {
        match w {
            0 => RegisterType::Byte(Register8Bit::from_u8(reg)),
            1 => RegisterType::Word(Register16Bit::from_u8(reg)),
            _ => panic!("Invalid register size"),
        }
    }
}

impl Display for RegisterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterType::Word(reg) => write!(f, "{}", reg),
            RegisterType::Byte(reg) => write!(f, "{}", reg),
            RegisterType::Segment(seg) => write!(f, "{}", seg),
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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

pub fn effective_address(rm: u8, mod_rm: u8, disp: u16, w: u8) -> String {
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
                format!("[{disp:04x}]", disp = disp)
            } else {
                format!("[{base}]")
            }
        }
        0b01 => {
            let disp_signed = disp as i8;
            if disp_signed >= 0 {
                format!("[{base}+{disp_signed:x}]")
            } else {
                format!("[{base}-{disp_signed:x}]", disp_signed = disp_signed.abs())
            }
        }
        0b10 => {
            let disp_signed = disp as i16;
            if disp_signed >= 0 {
                format!("[{base}+{disp:x}]", disp = disp_signed)
            } else {
                format!("[{base}-{disp:x}]", disp = disp_signed.abs())
            }
        }
        0b11 => format!("{reg}", reg = RegisterType::new(rm, w)),
        _ => panic!("Invalid mod"),
    }
    .to_string()
}

pub fn calc_relative_disp(offset: usize, disp: u16, is_2byte_disp: bool) -> u16 {
    if offset > u16::MAX as usize {
        panic!("Offset is overflowing");
    }
    let offset = offset as u16;
    let signed_disp = if is_2byte_disp {
        disp as i16
    } else {
        (disp as i8).into()
    };
    if signed_disp >= 0 {
        offset + signed_disp as u16
    } else {
        offset - signed_disp.abs() as u16
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SegmentRegister {
    ES = 0,
    CS = 1,
    SS = 2,
    DS = 3,
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

    #[test_case(0b000, 0b11, 0, 0, "AL" ; "REG 1")]
    #[test_case(0b001, 0b11, 0, 1, "CX" ; "REG 2")]
    #[test_case(0b100, 0b00, 0, 0, "SI"; "No disp 1")]
    #[test_case(0b000, 0b00, 0, 0, "[BX+SI]"; "No disp 2")]
    #[test_case(0b110, 0b01, 0xee, 0, "[BP-12]" ; "Sign-extended disp")]
    #[test_case(0b110, 0b10, 0x0f, 0, "[BP+f]" ; "r/m + Disp")]
    #[test_case(0b110, 0b00, 0x0f, 0, "[000f]" ; "only Disp")]
    #[test_case(0b110, 0b01, 0x04, 0, "[BP+4]" ; "mod=0b01")]
    fn test_effective_address(rm: u8, mod_rm: u8, disp: u16, w: u8, expected: &str) {
        let result = effective_address(rm, mod_rm, disp, w);
        assert_eq!(result, expected);
    }
}
