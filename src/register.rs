use core::panic;
use std::fmt::Display;

#[derive(Debug, Default)]
pub struct Register {
    reg_16: [u16; 8],
    reg_8: [u8; 8],
    reg_seg: [u16; 4],
}

#[allow(unused)]
impl Register {
    pub fn new() -> Self {
        Register::default()
    }

    pub fn get(&self, reg: RegisterType) -> u16 {
        match reg {
            RegisterType::Word(r) => self.reg_16[r as usize],
            RegisterType::Byte(r) => self.reg_8[r as usize] as u16,
            RegisterType::Segment(seg) => self.reg_seg[seg as usize],
        }
    }

    pub fn set(&mut self, reg: RegisterType, value: u16) {
        match reg {
            RegisterType::Word(r) => self.reg_16[r as usize] = value,
            RegisterType::Byte(r) => {
                if value > u8::MAX as u16 {
                    panic!("Value exceeds 8-bit register limit");
                }
                self.reg_8[r as usize] = value as u8;
            }
            RegisterType::Segment(seg) => self.reg_seg[seg as usize] = value,
        }
    }

    pub fn get_ax(&self) -> u16 {
        self.get(RegisterType::Word(Register16Bit::AX))
    }
    pub fn set_ax(&mut self, value: u16) {
        self.set(RegisterType::Word(Register16Bit::AX), value);
    }
    pub fn get_cx(&self) -> u16 {
        self.get(RegisterType::Word(Register16Bit::CX))
    }
    pub fn set_cx(&mut self, value: u16) {
        self.set(RegisterType::Word(Register16Bit::CX), value);
    }
    pub fn get_dx(&self) -> u16 {
        self.get(RegisterType::Word(Register16Bit::DX))
    }
    pub fn set_dx(&mut self, value: u16) {
        self.set(RegisterType::Word(Register16Bit::DX), value);
    }
    pub fn get_bx(&self) -> u16 {
        self.get(RegisterType::Word(Register16Bit::BX))
    }
    pub fn set_bx(&mut self, value: u16) {
        self.set(RegisterType::Word(Register16Bit::BX), value);
    }
    pub fn get_sp(&self) -> u16 {
        self.get(RegisterType::Word(Register16Bit::SP))
    }
    pub fn set_sp(&mut self, value: u16) {
        self.set(RegisterType::Word(Register16Bit::SP), value);
    }
    pub fn get_bp(&self) -> u16 {
        self.get(RegisterType::Word(Register16Bit::BP))
    }
    pub fn set_bp(&mut self, value: u16) {
        self.set(RegisterType::Word(Register16Bit::BP), value);
    }
    pub fn get_si(&self) -> u16 {
        self.get(RegisterType::Word(Register16Bit::SI))
    }
    pub fn set_si(&mut self, value: u16) {
        self.set(RegisterType::Word(Register16Bit::SI), value);
    }
    pub fn get_di(&self) -> u16 {
        self.get(RegisterType::Word(Register16Bit::DI))
    }
    pub fn set_di(&mut self, value: u16) {
        self.set(RegisterType::Word(Register16Bit::DI), value);
    }
    pub fn get_al(&self) -> u8 {
        self.get(RegisterType::Byte(Register8Bit::AL)) as u8
    }
    pub fn set_al(&mut self, value: u8) {
        self.set(RegisterType::Byte(Register8Bit::AL), value as u16);
    }
    pub fn get_cl(&self) -> u8 {
        self.get(RegisterType::Byte(Register8Bit::CL)) as u8
    }
    pub fn set_cl(&mut self, value: u8) {
        self.set(RegisterType::Byte(Register8Bit::CL), value as u16);
    }
    pub fn get_dl(&self) -> u8 {
        self.get(RegisterType::Byte(Register8Bit::DL)) as u8
    }
    pub fn set_dl(&mut self, value: u8) {
        self.set(RegisterType::Byte(Register8Bit::DL), value as u16);
    }
    pub fn get_bl(&self) -> u8 {
        self.get(RegisterType::Byte(Register8Bit::BL)) as u8
    }
    pub fn set_bl(&mut self, value: u8) {
        self.set(RegisterType::Byte(Register8Bit::BL), value as u16);
    }
    pub fn get_ah(&self) -> u8 {
        self.get(RegisterType::Byte(Register8Bit::AH)) as u8
    }
    pub fn set_ah(&mut self, value: u8) {
        self.set(RegisterType::Byte(Register8Bit::AH), value as u16);
    }
    pub fn get_ch(&self) -> u8 {
        self.get(RegisterType::Byte(Register8Bit::CH)) as u8
    }
    pub fn set_ch(&mut self, value: u8) {
        self.set(RegisterType::Byte(Register8Bit::CH), value as u16);
    }
    pub fn get_dh(&self) -> u8 {
        self.get(RegisterType::Byte(Register8Bit::DH)) as u8
    }
    pub fn set_dh(&mut self, value: u8) {
        self.set(RegisterType::Byte(Register8Bit::DH), value as u16);
    }
    pub fn get_bh(&self) -> u8 {
        self.get(RegisterType::Byte(Register8Bit::BH)) as u8
    }
    pub fn set_bh(&mut self, value: u8) {
        self.set(RegisterType::Byte(Register8Bit::BH), value as u16);
    }
    pub fn get_es(&self) -> u16 {
        self.get(RegisterType::Segment(SegmentRegister::ES))
    }
    pub fn set_es(&mut self, value: u16) {
        self.set(RegisterType::Segment(SegmentRegister::ES), value);
    }
    pub fn get_cs(&self) -> u16 {
        self.get(RegisterType::Segment(SegmentRegister::CS))
    }
    pub fn set_cs(&mut self, value: u16) {
        self.set(RegisterType::Segment(SegmentRegister::CS), value);
    }
    pub fn get_ss(&self) -> u16 {
        self.get(RegisterType::Segment(SegmentRegister::SS))
    }
    pub fn set_ss(&mut self, value: u16) {
        self.set(RegisterType::Segment(SegmentRegister::SS), value);
    }
    pub fn get_ds(&self) -> u16 {
        self.get(RegisterType::Segment(SegmentRegister::DS))
    }
    pub fn set_ds(&mut self, value: u16) {
        self.set(RegisterType::Segment(SegmentRegister::DS), value);
    }
}

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
