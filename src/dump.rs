use core::panic;

use crate::register::{
    calc_relative_disp, effective_address, Register, Register16Bit, SegmentRegister,
};

use super::operation::Operation;

fn dump_op_info(op: &Operation) {
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
    print!("{}", effective_address(op.rm, op.mod_rm, op.disp, op.w));
}

fn dump_immediate(op: &Operation) {
    match op.w {
        0 => print!("{:02x}", op.data),
        1 => match op.s {
            0 => print!("{:04x}", op.data),
            1 => print!("{:x}", op.data as i16),
            _ => panic!("Invalid s"),
        },
        _ => panic!("Invalid w"),
    }
}

fn dump_segment_register(seg_reg: u8) {
    print!("{}", SegmentRegister::from_u8(seg_reg));
}

fn dump_relative_disp(op: &Operation, is_2byte_disp: bool) {
    let offset = op.get_next_operation_pos();
    print!("{:04x}", calc_relative_disp(offset, op.disp, is_2byte_disp));
}

fn dump_port(port: u8) {
    print!("{:02x}", port);
}

// --- Dump Operation ---

pub fn move1(op: &Operation) {
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

pub fn move2(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_ea(op);
    dump_comma();
    dump_immediate(op);
}

pub fn move3(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_reg(op.reg, op.w);
    dump_comma();
    dump_immediate(&op);
}

pub fn push1(op: &Operation) {
    // Register/Memory
    dump_op_info(op);
    dump_space();
    dump_ea(op);
}

pub fn push2(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_reg(op.reg, 1);
}

pub fn push3(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_segment_register(op.reg);
}

pub fn pop1(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_ea(op);
}

pub fn pop2(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_reg(op.reg, 1);
}

pub fn pop3(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_segment_register(op.reg);
}

pub fn in1(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_reg(0b000, op.w);
    dump_comma();
    dump_port(op.port);
}

pub fn in2(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_reg(0b000, op.w);
    dump_comma();
    print!("{dx_reg}", dx_reg = Register16Bit::DX);
}

pub fn lea(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_reg(op.reg, 1);
    dump_comma();
    dump_ea(op);
}

pub fn add1(op: &Operation) {
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

pub fn add2(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_ea(op);
    dump_comma();
    dump_immediate(op);
}

pub fn inc1(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_ea(op);
}

pub fn inc2(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_reg(op.reg, op.w);
}

pub fn sub1(op: &Operation) {
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

pub fn sub2(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_ea(&op);
    dump_comma();
    dump_immediate(&op);
}

pub fn sub3(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_segment_register(op.reg);
    dump_comma();
    dump_immediate(op);
}

pub fn ssb1(op: &Operation) {
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

pub fn neg(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_ea(op);
}

pub fn cmp2(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_ea(op);
    dump_comma();
    dump_immediate(op);
}

pub fn cbw(op: &Operation) {
    dump_op_info(op);
}

pub fn cwd(op: &Operation) {
    dump_op_info(op);
}

pub fn or1(op: &Operation) {
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

pub fn xor1(op: &Operation) {
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

pub fn call1(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_relative_disp(op, true);
}

pub fn jmp1(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_relative_disp(op, true);
}

pub fn jmp2(op: &Operation) {
    dump_op_info(op);
    dump_space();
    print!("short");
    dump_space();
    dump_relative_disp(op, false);
}

pub fn ret1(op: &Operation) {
    dump_op_info(op);
}

pub fn jump(op: &Operation) {
    dump_op_info(op);
    dump_space();
    dump_relative_disp(op, false);
}

pub fn int1(op: &Operation) {
    dump_op_info(op);
    dump_space();
    print!("{:02x}", op.int_type);
}

pub fn int2(op: &Operation) {
    dump_op_info(op);
    dump_space();
    print!("3");
}

pub fn none(op: &Operation) {
    dump_op_info(op);
}
