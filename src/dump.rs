use core::panic;

use crate::register::{effective_address, Register};

use super::operation::Operation;

fn dump_common(op: &Operation) {
    let mut bytes = String::new();
    for byte in &op.raws {
        bytes.push_str(&format!("{:02x}", byte));
    }
    // Insert Tab
    for _ in bytes.len()..14 {
        bytes.push(' ');
    }
    print!(
        "{pos:04x}: {bytes}{op_type}",
        pos = op.pos,
        op_type = op.operation_type
    );
}

fn dump_comma() {
    print!(", ");
}

fn dump_space() {
    print!(" ");
}

fn dump_reg(reg: u8, w: u8) {
    let reg = Register::new(reg, w);
    print!("{reg}");
}

fn dump_ea(op: &Operation) {
    match op.mod_rm {
        0b00 => {
            if op.rm == 0b110 {
                print!("[{:04x}]", op.disp)
            } else {
                let ea = effective_address(op.rm);
                print!("[{ea}]")
            }
        }
        0b01 | 0b10 => {
            let ea = effective_address(op.rm);
            print!("[{ea}+{disp}]", disp = op.disp)
        }
        0b11 => {
            let reg = Register::new(op.rm, op.w);
            print!("{reg}")
        }
        _ => panic!("Invalid mod"),
    };
}

fn dump_immediate(op: &Operation) {
    match op.w {
        0 => print!("{:02x}", op.data),
        1 => match op.s {
            0 => print!("{:04x}", op.data),
            1 => print!("{:04x}", op.data as i16),
            _ => panic!("Invalid s"),
        },
        _ => panic!("Invalid w"),
    }
}

pub fn move1(op: &Operation) {
    dump_common(op);
    dump_space();
    match op.d {
        0 => {
            dump_ea(op);
            dump_comma();
            dump_reg(op.reg, op.w);
        }
        1 => {
            dump_comma();
            dump_ea(op);
            dump_reg(op.reg, op.w);
        }
        _ => panic!("Invalid d"),
    }
}

pub fn move2(op: &Operation) {
    dump_common(op);
    dump_space();
    // TODO
    panic!("Not implemented yet");
}

pub fn move3(op: &Operation) {
    dump_common(op);
    dump_space();
    dump_reg(op.reg, op.w);
    dump_comma();
    dump_immediate(&op);
}

pub fn add1(op: &Operation) {
    dump_common(op);
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

pub fn sub2(op: &Operation) {
    dump_common(op);
    dump_space();
    dump_ea(&op);
    dump_comma();
    dump_immediate(&op);
}

pub fn int1(op: &Operation) {
    dump_common(op);
    dump_space();
    print!("{:02x}", op.int_type);
}

pub fn int2(op: &Operation) {
    dump_common(op);
    dump_space();
    print!("3");
}

pub fn none(op: &Operation) {
    dump_common(op);
    dump_space();
}
