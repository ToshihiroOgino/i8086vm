use core::panic;

use crate::{
    flag,
    operation::OperationType,
    register::{
        calc_relative_disp, effective_address, Register, Register16Bit, Register8Bit, RegisterType,
        SegmentRegister,
    },
};

use super::operation::Operation;

fn dump_type(op_type: &OperationType, w: u8) {
    let type_str = match op_type {
        OperationType::Movs
        | OperationType::Cmps
        | OperationType::Scas
        | OperationType::Lods
        | OperationType::Stos => {
            let str = op_type.to_string();
            if w == 0 {
                format!("{str}B")
            } else {
                format!("{str}W")
            }
        }
        _ => format!("{op_type}"),
    };
    print!("{type_str}");
}

fn dump_op_info(op: &Operation) {
    let mut bytes = String::new();
    for byte in &op.raws {
        bytes.push_str(&format!("{:02x}", byte));
    }
    // Insert Tab
    for _ in bytes.len()..14 {
        bytes.push(' ');
    }
    print!("{pos:04x}: {bytes}", pos = op.pos);
    dump_type(&op.operation_type, op.w);
}

fn dump_comma() {
    print!(", ");
}

fn dump_space() {
    print!(" ");
}

fn dump_reg(reg: u8, w: u8) {
    let reg = RegisterType::new(reg, w);
    print!("{reg}");
}

fn dump_ea(op: &Operation) {
    print!("{}", effective_address(op.rm, op.mod_rm, op.disp, op.w));
}

fn dump_immediate(op: &Operation) {
    match op.w {
        0 => print!("{:x}", op.data),
        1 => match op.s {
            0 => print!("{:04x}", op.data),
            1 => {
                let data = op.data as i8;
                if data >= 0 {
                    print!("{:x}", data)
                } else {
                    print!("-{:x}", data.abs())
                }
            }
            _ => panic!("Invalid s"),
        },
        _ => panic!("Invalid w"),
    }
}

fn dump_segment_register(seg_reg: u8) {
    print!("{}", SegmentRegister::from_u8(seg_reg));
}

fn dump_absolute_disp(disp: u16) {
    print!("{:04x}", disp);
}

fn dump_relative_disp(op: &Operation, is_2byte_disp: bool) {
    let offset = op.get_next_operation_pos();
    print!("{:04x}", calc_relative_disp(offset, op.disp, is_2byte_disp));
}

fn dump_port(port: u8) {
    print!("{:02x}", port);
}

fn dump_byte() {
    print!("Byte");
}

fn dump_short() {
    print!("Short");
}

fn dump_count(v: u8) {
    match v {
        0 => print!("1"),
        1 => print!("{}", Register8Bit::CL),
        _ => panic!("Invalid v"),
    }
}

pub struct Dump {
    pub enabled: bool,
}

impl Dump {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    // --- Dump Operation ---
    pub fn name(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }
        dump_op_info(op);
    }

    pub fn mov4(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }
        // Memory to Accumulator
        dump_op_info(op);
        dump_space();
        print!("{acc_reg}", acc_reg = RegisterType::new(0b000, op.w));
        dump_comma();
        dump_ea(op);
    }

    pub fn mov5(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        // Accumulator to Memory
        dump_op_info(op);
        dump_space();
        dump_ea(op);
        dump_comma();
        print!("{acc_reg}", acc_reg = RegisterType::new(0b000, op.w));
    }

    pub fn simple_calc1(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        // Add, Sub, etc...
        // Reg./Memory with Register to Either
        dump_op_info(op);
        dump_space();
        match op.d {
            0 => {
                dump_ea(op);
                dump_comma();
                dump_reg(op.reg, op.w);
            }
            1 => {
                dump_reg(op.reg, op.w);
                dump_comma();
                dump_ea(op);
            }
            _ => panic!("Invalid d"),
        }
    }

    pub fn simple_calc2(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        // Add, Sub, etc...
        // Immediate to Register/Memory
        dump_op_info(op);
        dump_space();
        if op.mod_rm != 0b11 && op.w == 0 {
            dump_byte();
            dump_space();
        }
        dump_ea(op);
        dump_comma();
        dump_immediate(op);
    }

    pub fn simple_calc3(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        // Add, Sub, etc...
        // Immediate to Accumulator
        dump_op_info(op);
        dump_space();
        dump_reg(op.reg, op.w);
        dump_comma();
        dump_immediate(&op);
    }

    pub fn stack1(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        // Register/Memory
        dump_op_info(op);
        dump_space();
        dump_ea(op);
    }

    pub fn stack2(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        // Register
        dump_op_info(op);
        dump_space();
        dump_reg(op.reg, 1);
    }

    pub fn stack3(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        // Segment Register
        dump_op_info(op);
        dump_space();
        dump_segment_register(op.reg);
    }

    pub fn xchg1(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        // Register/Memory with Register
        dump_op_info(op);
        dump_space();
        dump_ea(op);
        dump_comma();
        dump_reg(op.reg, op.w);
    }

    pub fn xchg2(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        // Register with Accumulator
        dump_op_info(op);
        dump_space();
        dump_reg(op.reg, 1);
        dump_comma();
        print!("{acc_reg}", acc_reg = Register16Bit::AX);
    }

    pub fn in1(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        dump_op_info(op);
        dump_space();
        dump_reg(0b000, op.w);
        dump_comma();
        dump_port(op.port);
    }

    pub fn in2(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        dump_op_info(op);
        dump_space();
        dump_reg(0b000, op.w);
        dump_comma();
        print!("{dx_reg}", dx_reg = Register16Bit::DX);
    }

    pub fn lea(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        dump_op_info(op);
        dump_space();
        dump_reg(op.reg, 1);
        dump_comma();
        dump_ea(op);
    }

    pub fn inc_dec1(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        dump_op_info(op);
        dump_space();
        dump_ea(op);
    }

    pub fn inc_dec2(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        dump_op_info(op);
        dump_space();
        dump_reg(op.reg, op.w);
    }

    pub fn complicate_calc(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        dump_op_info(op);
        dump_space();
        dump_ea(op);
    }

    pub fn bit_op1(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        dump_op_info(op);
        dump_space();
        match op.d {
            0 => {
                dump_ea(op);
                dump_comma();
                dump_reg(op.reg, op.w);
            }
            1 => {
                dump_reg(op.reg, op.w);
                dump_comma();
                dump_ea(op);
            }
            _ => panic!("Invalid d"),
        }
    }

    pub fn bit_op3(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        dump_op_info(op);
        dump_space();
        dump_reg(op.reg, op.w);
        dump_comma();
        dump_immediate(op);
    }

    pub fn rep(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        dump_op_info(op);
        dump_space();
        dump_type(&op.rep_operation_type, op.w);
    }

    pub fn shift_rotate(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        dump_op_info(op);
        dump_space();
        dump_ea(op);
        dump_comma();
        dump_count(op.v);
    }

    pub fn test2(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        dump_op_info(op);
        dump_space();
        if op.w == 0 && op.mod_rm != 0b11 {
            dump_byte();
            dump_space();
        }
        dump_ea(op);
        dump_comma();
        dump_immediate(op);
    }

    pub fn call1(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        dump_op_info(op);
        dump_space();
        dump_relative_disp(op, true);
    }

    pub fn jmp1(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        dump_op_info(op);
        dump_space();
        dump_relative_disp(op, true);
    }

    pub fn jmp2(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        dump_op_info(op);
        dump_space();
        dump_short();
        dump_space();
        dump_relative_disp(op, false);
    }

    pub fn ret2(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        dump_op_info(op);
        dump_space();
        dump_absolute_disp(op.disp);
    }

    pub fn jump(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        dump_op_info(op);
        dump_space();
        dump_relative_disp(op, false);
    }

    pub fn loop1(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        dump_op_info(op);
        dump_space();
        dump_relative_disp(op, false);
    }

    pub fn int1(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        dump_op_info(op);
        dump_space();
        print!("{:02x}", op.int_type);
    }

    pub fn int2(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }

        dump_op_info(op);
        dump_space();
        print!("3");
    }

    pub fn none(&self, op: &Operation) {
        if !self.is_enabled() {
            return;
        }
        dump_op_info(op);
    }

    pub fn eol(&self) {
        if !self.is_enabled() {
            return;
        }
        println!();
        // stdout().flush().unwrap();
    }

    pub fn labels(&self) {
        if !self.is_enabled() {
            return;
        }
        println!(" AX   BX   CX   DX   SP   BP   SI   DI  FLAGS IP");
    }

    pub fn state(&self, reg: &Register, flags: &flag::Flag) {
        if !self.is_enabled() {
            return;
        }
        print!(
            "{ax:04x} {bx:04x} {cx:04x} {dx:04x} {sp:04x} {bp:04x} {si:04x} {di:04x} {flags} ",
            ax = reg.get_ax(),
            bx = reg.get_bx(),
            cx = reg.get_cx(),
            dx = reg.get_dx(),
            sp = reg.sp,
            bp = reg.bp,
            si = reg.si,
            di = reg.di,
            flags = flags,
        );
    }

    pub fn address_value(&self, addr: usize, value: u16) {
        if !self.is_enabled() {
            return;
        }
        print!(" ;[{addr:04x}]{value:04x}", addr = addr, value = value);
    }

    pub fn address_value_change(&self, addr: usize, prev_value: u16, new_value: u16) {
        if !self.is_enabled() {
            return;
        }
        print!(
            " ;[{addr:04x}]{prev_value:04x}->{new_value:04x}",
            addr = addr,
            prev_value = prev_value,
            new_value = new_value
        );
    }

    pub fn write(&self, fd: u16, addr: usize, len: u16) {
        if !self.is_enabled() {
            return;
        }
        print!("\n<write(fd={}, addr=0x{:04x}, len={})>", fd, addr, len);
    }

    pub fn exit(&self, status: u16) {
        if !self.is_enabled() {
            return;
        }
        print!("\n<exit({})>\n", status);
    }

    pub fn ioctl(&self, fd: u16, req: u16, addr: u16) {
        if !self.is_enabled() {
            return;
        }
        print!("\n<ioctl(fd={}, req=0x{:04x}, addr=0x{:04x})>", fd, req, addr);
    }

    pub fn brk(&self, addr: u16, ok: bool) {
        if !self.is_enabled() {
            return;
        }
        print!("\n<brk(0x{:04x}) => ", addr);
        if ok {
            print!("0>");
        } else {
            print!("ENOMEM>");
        }
    }
}
