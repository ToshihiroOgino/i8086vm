use crate::{
    disassembler,
    dump::Dump,
    flag::Flag,
    message::{Message, MESSAGE_SIZE},
    metadata::{self},
    operation::{OperandType, Operation, OperationType},
    register::{self, Register, RegisterType},
};

pub struct Machine {
    stop: bool,
    memory: Vec<u8>,
    register: Register,
    metadata: metadata::Metadata,
    flag: Flag,
    dump: Dump,
    text: Vec<u8>,

    // For debugging
    stop_count: u16,
}

fn read_16(memory: &[u8], addr: usize) -> u16 {
    if addr + 1 >= memory.len() {
        panic!("Memory access out of bounds at address {}", addr + 1);
    }
    u16::from_le_bytes([memory[addr], memory[addr + 1]])
}

fn write_16(memory: &mut [u8], addr: usize, value: u16) {
    if addr + 1 >= memory.len() {
        panic!("Memory access out of bounds at address {}", addr + 1);
    }
    memory[addr..addr + 2].copy_from_slice(&value.to_le_bytes());
}

fn carry_lsh_u8(value: u8, count: u8) -> bool {
    if count == 0 {
        return false;
    }
    let res = (value << (count - 1)) >> 7;
    res & 1 == 1
}

fn carry_lsh_u16(value: u16, count: u16) -> bool {
    if count == 0 {
        return false;
    }
    let res = (value << (count - 1)) >> 15;
    res & 1 == 1
}

fn carry_rsh_u16(value: u16, count: u16) -> bool {
    if count == 0 {
        return false;
    }
    let res = (value >> (count - 1)) & 1;
    res == 1
}

impl Machine {
    pub fn new(executable: &Vec<u8>, args: &[String], envs: &[String], debug: bool) -> Self {
        let metadata = metadata::Metadata::from_bytes(executable);

        let text_begin = metadata.hdr_len as usize;
        let text = executable[text_begin..text_begin + metadata.text_size].to_vec();

        let mut memory = vec![0; metadata.total];
        let data_begin = metadata.hdr_len as usize + metadata.text_size;
        memory[0..metadata.data_size]
            .copy_from_slice(&executable[data_begin..data_begin + metadata.data_size]);

        let args_frame = Self::create_args_frame(args, envs, metadata.total);
        let frame_base = metadata.total - args_frame.len();
        memory[frame_base..metadata.total].copy_from_slice(&args_frame);

        // dump args
        // for i in 0xffcc..metadata.total {
        //     if (i == 0xffcc) || (i % 16 == 0) {
        //         print!("\n{:04x}: ", i);
        //     }
        //     print!("{:02x} ", memory[i]);
        // }
        // println!();

        let mut register = Register::new();
        register.sp = frame_base as u16;

        Machine {
            stop: false,
            memory,
            register,
            metadata,
            flag: Flag::new(),
            dump: Dump::new(debug),
            text,

            stop_count: 0,
        }
    }

    fn create_args_frame(args: &[String], envs: &[String], total_memory: usize) -> Vec<u8> {
        let mut args_offset = Vec::new();
        let mut args_seg = Vec::new();
        for arg in args {
            let buf = arg.as_bytes();
            args_offset.push(args_seg.len());
            args_seg.extend_from_slice(buf);
            if !arg.ends_with("\0") {
                args_seg.push(0);
            }
        }

        let mut env_offset = Vec::new();
        let mut env_seg: Vec<u8> = Vec::new();
        for env in envs {
            env_offset.push(env_seg.len());
            env_seg.extend_from_slice(env.as_bytes());
            if !env.ends_with("\0") {
                env_seg.push(0);
            }
        }

        let header_size = 2 + // argc
            args_offset.len() * 2 + // args address
            2 + // 0u16
            env_offset.len() * 2 + // env address
            2; // 0u16

        let mut frame_size = header_size
            + args_seg.len() // args string
            + env_seg.len(); // env string

        let last_0_required = frame_size % 2 != 0;

        if last_0_required {
            frame_size += 1; // align to even size
        }

        let frame_base = total_memory - frame_size;
        let args_begin = frame_base + header_size;
        let env_begin = args_begin + args_seg.len() + 2;

        // argc
        let mut frame = (args.len() as u16).to_le_bytes().to_vec();

        // args address
        for offset in args_offset {
            let addr = (args_begin + offset) as u16;
            frame.extend_from_slice(&addr.to_le_bytes());
        }
        // padding
        frame.extend_from_slice(&0u16.to_le_bytes());

        // env address
        for offset in env_offset {
            let addr = (env_begin + offset) as u16;
            frame.extend_from_slice(&addr.to_le_bytes());
        }
        // padding
        frame.extend_from_slice(&0u16.to_le_bytes());

        // args string
        frame.extend_from_slice(&args_seg);
        // env string
        frame.extend_from_slice(&env_seg);

        if last_0_required {
            frame.push(0);
        }

        frame
    }

    fn calc_effective_address(&self, op: &Operation) -> usize {
        if op.mod_rm == 0b11 {
            let addr = self.register.get(RegisterType::new(op.rm, op.w)) as usize;
            return addr;
        }

        match op.raws[0] {
            0b1110_1000 | 0b1110_1001 => {
                let offset = op.get_next_operation_pos();
                register::calc_relative_disp(offset, op.disp, true)
            }
            0b0111_0000..=0b0111_1111 | 0b1110_0000..=0b1110_0010 | 0b1110_1011 => {
                let offset = op.get_next_operation_pos();
                register::calc_relative_disp(offset, op.disp, false)
            }
            _ => {
                let base = match op.rm {
                    0b000 => self.register.get_bx() + self.register.si,
                    0b001 => self.register.get_bx() + self.register.di,
                    0b010 => self.register.bp + self.register.si,
                    0b011 => self.register.bp + self.register.di,
                    0b100 => self.register.si,
                    0b101 => self.register.di,
                    0b110 => self.register.bp,
                    0b111 => self.register.get_bx(),
                    _ => panic!("Invalid effective address"),
                };
                match op.mod_rm {
                    0b00 => {
                        if op.rm == 0b110 {
                            op.disp
                        } else {
                            base
                        }
                    }
                    0b01 => {
                        let disp_signed = op.disp as i8;
                        if disp_signed >= 0 {
                            base + disp_signed as u16
                        } else {
                            base - disp_signed.abs() as u16
                        }
                    }
                    0b10 => {
                        let disp_signed = op.disp as i16;
                        if disp_signed >= 0 {
                            base + disp_signed as u16
                        } else {
                            base - disp_signed.abs() as u16
                        }
                    }
                    0b11 => self.register.get(RegisterType::new(op.rm, op.w)),
                    _ => unreachable!(),
                }
            }
        }
        .try_into()
        .unwrap_or_else(|_| panic!("Effective address calculation overflowed"))
    }

    fn read_operand(&self, op: &Operation, operand: OperandType) -> u16 {
        match operand {
            OperandType::Reg => self.register.get(op.get_register()),
            OperandType::EA => {
                if op.mod_rm == 0b11 {
                    return self.register.get(RegisterType::new(op.rm, op.w));
                }

                let addr = self.calc_effective_address(op);
                if addr + 1 >= self.memory.len() {
                    panic!("Memory access out of bounds at address {}", addr + 1);
                }
                let value = u16::from_le_bytes([self.memory[addr], self.memory[addr + 1]]);
                self.dump.address_value(addr, value);
                value
            }
            OperandType::SegReg => match op.reg {
                0b00 => self.register.es,
                0b01 => self.register.cs,
                0b10 => self.register.ss,
                0b11 => self.register.ds,
                _ => panic!("Invalid segment register"),
            },
            OperandType::Imm => op.data,
            _ => unreachable!("Invalid operand type: {:?}", operand),
        }
    }

    fn write_operand(&mut self, op: &Operation, operand: OperandType, value: u16) {
        match operand {
            OperandType::Reg => self.register.set(op.get_register(), value),
            OperandType::Imm => panic!("Immediate value cannot be used as a destination"),
            OperandType::EA => {
                if op.mod_rm == 0b11 {
                    self.register.set(RegisterType::new(op.rm, op.w), value);
                    return;
                }

                let addr = self.calc_effective_address(op);
                let prev = u16::from_le_bytes([
                    self.memory[addr],
                    if op.w == 0 { 0 } else { self.memory[addr + 1] },
                ]);
                self.dump.address_value_change(addr, prev, value);
                if addr >= self.memory.len() {
                    panic!("Memory access out of bounds at address {}", addr);
                }
                if op.w == 0 {
                    self.memory[addr] = value as u8;
                } else {
                    if addr + 1 >= self.memory.len() {
                        panic!("Memory access out of bounds at address {}", addr + 1);
                    }
                    self.memory[addr..addr + 2].copy_from_slice(&value.to_le_bytes());
                }
            }
            _ => unreachable!(),
        }
    }

    fn get_data_segment(&self) -> &[u8] {
        let begin = 0;
        let end = begin + self.metadata.total;
        &self.memory[begin..end]
    }

    fn mov(&mut self, op: &Operation) {
        let val = self.read_operand(op, op.second);
        self.write_operand(op, op.first, val);
    }

    fn add(&mut self, op: &Operation) {
        let left = self.read_operand(op, op.first);
        let right = self.read_operand(op, op.second);
        let result = match op.s << 1 | op.w {
            0b00 => {
                let res = left + right;
                let res_u8 = left as u8 + right as u8;
                self.flag
                    .set_cosz(res >= 0x100, res >= 0x100, false, res == 0);
                res_u8 as u16
            }
            0b01 => {
                let res = left as u16 as i32 + right as u16 as i32;
                let res_u16 = left + right;
                self.flag.set_cosz(
                    res > 0x10000,
                    res != res_u16 as i32,
                    (res_u16 as i16) < 0,
                    res_u16 == 0,
                );
                res_u16 as u16
            }
            0b11 => {
                let res = left as i16 as i32 + right as i16 as i32;
                let res_i16 = left as i16 + right as i16;
                self.flag.set_cosz(
                    res > 0x10000,
                    res != res_i16 as i32,
                    res_i16 < 0,
                    res_i16 == 0,
                );
                res_i16 as u16
            }
            _ => unreachable!("Invalid operand width"),
        };
        self.write_operand(op, op.first, result);
    }

    fn sub(&mut self, op: &Operation) {
        let left = self.read_operand(op, op.first);
        let right = self.read_operand(op, op.second);
        let result: u16 = match op.s << 1 | op.w {
            0b00 => {
                let res_u8 = left as u8 - right as u8;
                let res = res_u8 as i32;
                self.flag.zero = res_u8 == 0;
                self.flag.sign = false;
                self.flag.overflow = res as i32 != res;
                self.flag.carry = left < right;
                res as u16
            }
            0b01 => {
                let res_i16: i16 = left as i16 - right as i16;
                let res = res_i16 as i32;
                self.flag.zero = res == 0;
                self.flag.sign = res < 0;
                self.flag.overflow = res != res_i16 as i32;
                self.flag.carry = left < right;
                res_i16 as u16
            }
            0b11 => {
                let res_i16: i16 = left as i16 - right as i16;
                let res = (left as i16 - (right as u8 as i16)) as i32;
                self.flag.zero = res_i16 == 0;
                self.flag.sign = res_i16 < 0;
                self.flag.overflow = res != res_i16 as i32;
                self.flag.carry = left < right as u8 as u16;
                res_i16 as u16
            }
            _ => unreachable!("Invalid operand width"),
        };
        self.write_operand(op, op.first, result);
    }

    fn div(&mut self, op: &Operation) {
        let left = self.read_operand(op, op.second);
        let numerator = self.register.get_ax() as u32 | ((self.register.get_dx() as u32) << 16);
        match op.w {
            0 => {
                let quot_i32 = numerator as i32 / left as i32;
                let quot_u8 = (quot_i32 & 0xff) as u8;
                let rem = numerator % left as u32;
                self.register.al = quot_u8;
                self.register.ah = (rem & 0xff) as u8;
            }
            1 => {
                let quot_i32 = numerator as i32 / left as i32;
                let quot_u16 = (quot_i32 & 0xffff) as u16;
                let rem = numerator % left as u32;
                self.register.set_ax(quot_u16);
                self.register.set_dx((rem & 0xffff) as u16);
            }
            _ => unreachable!("Invalid w"),
        }
    }

    fn neg(&mut self, op: &Operation) {
        let value = self.read_operand(op, op.first);
        if op.w == 1 {
            let val_i32 = value as i32;
            let res_i16 = -val_i32 as i16;
            self.flag
                .set_cosz(value != 0, false, res_i16 < 0, res_i16 == 0);
            self.write_operand(op, op.first, res_i16 as u16);
        } else {
            let val_u8 = value as u8;
            let res_i8 = -(val_u8 as i8);
            self.flag
                .set_cosz(value != 0, false, res_i8 < 0, res_i8 == 0);
            self.write_operand(op, op.first, res_i8 as u16);
        }
    }

    fn inc(&mut self, op: &Operation) {
        let value = self.read_operand(op, op.first) as i16;
        let result = value as i32 + 1;

        if op.w == 0 {
            let res_u8 = (result & 0xff) as u8;
            self.flag
                .set_cosz(self.flag.carry, res_u8 as i32 != result, false, res_u8 == 0);
            self.write_operand(op, op.first, res_u8 as u16);
        } else {
            let res_i16 = (result & 0xffff) as i16;
            self.flag.set_cosz(
                self.flag.carry,
                result != res_i16 as i32,
                res_i16 < 0,
                res_i16 == 0,
            );
            self.write_operand(op, op.first, res_i16 as u16);
        }
    }

    fn dec(&mut self, op: &Operation) {
        let value = self.read_operand(op, op.first);
        let res = (value as i16 - 1) as i32;

        if op.w == 0 {
            let res_u8 = (res & 0xff) as u8;
            self.flag
                .set_cosz(self.flag.carry, res_u8 as i32 != res, false, res_u8 == 0);
            self.write_operand(op, op.first, res_u8 as u16);
        } else {
            let res_i16 = res as i16;
            self.flag.set_cosz(
                self.flag.carry,
                res != res_i16 as i32,
                res < 0,
                res_i16 == 0,
            );
            self.write_operand(op, op.first, res_i16 as u16);
        }
    }

    fn int(&mut self) {
        let bx = self.register.get_bx() as usize;
        let msg = Message::load(self.get_data_segment(), bx);
        match msg.message_type {
            1 => {
                // exit
                let detail = msg.load_detail1(self.get_data_segment());
                self.exit(detail.m1i1());
            }
            4 => {
                // write
                let detail = msg.load_detail1(self.get_data_segment());
                let fd = detail.m1i1();
                let addr = detail.m1p1();
                let len = detail.m1i2();
                self.write(fd, addr as usize, len);

                // Write errno and return value to memory
                // res
                write_16(&mut self.memory, bx, 0);
                // errno
                write_16(&mut self.memory, bx + 2, len);

                self.register.set_ax(0);
            }
            17 => {
                // brk
                let addr = read_16(self.get_data_segment(), bx + MESSAGE_SIZE + 6);
                let res = self.brk(addr);
                self.register.set_ax(0);
                write_16(&mut self.memory, bx, 0);
                if res {
                    write_16(&mut self.memory, bx + 2, 0);
                } else {
                    write_16(&mut self.memory, bx + 2, 12); // EINVAL
                }
            }
            // 19 => {
            //     // lseek
            //     let fd = read_16(self.get_data_segment(), bx + MESSAGE_SIZE);
            //     let offset = read_16(self.get_data_segment(), bx + MESSAGE_SIZE + 6);
            //     let whence = read_16(self.get_data_segment(), bx + MESSAGE_SIZE + 2);
            // }
            54 => {
                // ioctl
                let fd = read_16(self.get_data_segment(), bx + MESSAGE_SIZE);
                let req = read_16(self.get_data_segment(), bx + MESSAGE_SIZE + 4);
                let addr = read_16(self.get_data_segment(), bx + MESSAGE_SIZE + 14);
                self.ioctl(fd, req, addr);
                self.register.set_ax(0);
                write_16(&mut self.memory, bx, 0);
                let errno = -22 as i16;
                write_16(&mut self.memory, bx + 2, errno as u16);
            }
            _ => {
                panic!("\nUnhandled interrupt type: {}", msg.message_type);
            }
        }
    }

    fn push(&mut self, op: &Operation) {
        let value = self.read_operand(op, op.first);
        self.stack_push_u16(value);
    }

    fn pop(&mut self, op: &Operation) {
        let res = self.stack_pop_u16();
        self.write_operand(op, op.first, res);
    }

    fn call(&mut self, op: &Operation) {
        let addr = self.calc_effective_address(op);
        if addr >= self.memory.len() {
            panic!("Memory access out of bounds at address {}", addr);
        }
        let return_addr = self.register.ip as u16;
        self.stack_push_u16(return_addr);
        self.register.ip = addr as u16;
    }

    fn lea(&mut self, op: &Operation) {
        let addr = self.calc_effective_address(op);
        let val = match op.w {
            0 => self.memory[addr] as u16,
            1 => {
                if addr + 1 >= self.memory.len() {
                    panic!("Memory access out of bounds at address {}", addr + 1);
                }
                u16::from_le_bytes([self.memory[addr], self.memory[addr + 1]])
            }
            _ => unreachable!("Invalid operand width"),
        };
        self.dump.address_value(addr, val);
        self.register.set(op.get_register(), addr as u16);
    }

    fn ret(&mut self, op: &Operation) {
        let return_addr = self.stack_pop_u16();
        if return_addr as usize >= self.memory.len() {
            panic!("Memory access out of bounds at address {}", return_addr);
        }
        match op.first {
            OperandType::Imm => {
                // Within Segment Adding Immediate to Sp
                let imm = op.disp;
                self.register.sp = self.register.sp.wrapping_add(imm);
                self.register.ip = return_addr;
            }
            OperandType::None => {
                // Within Segment
                self.register.ip = return_addr;
            }
            _ => unreachable!("Invalid operand type for ret: {:?}", op.first),
        }
    }

    fn and(&mut self, op: &Operation) {
        let left = self.read_operand(op, op.first);
        let right = self.read_operand(op, op.second);
        let result = match op.w {
            0 => (left as u8 & right as u8) as u16,
            1 => left & right,
            _ => unreachable!("Invalid operand width"),
        };
        self.flag
            .set_cosz(false, false, (result as i16) < 0, result == 0);
        self.write_operand(op, op.first, result);
    }

    fn or(&mut self, op: &Operation) {
        let left = self.read_operand(op, op.first);
        let right = self.read_operand(op, op.second);
        let result = match op.w {
            0 => (left as u8 | right as u8) as u16,
            1 => left | right,
            _ => unreachable!("Invalid operand width"),
        };
        self.flag.set_cosz(false, false, false, result == 0);
        self.write_operand(op, op.first, result);
    }

    fn xor(&mut self, op: &Operation) {
        let left = self.read_operand(op, op.first);
        let right = self.read_operand(op, op.second);
        let result = match op.w {
            0 => (left as u8 ^ right as u8) as u16,
            1 => left ^ right,
            _ => unreachable!("Invalid operand width"),
        };
        self.flag.set_cosz(false, false, false, result == 0);
        self.write_operand(op, op.first, result);
    }

    fn cmp(&mut self, op: &Operation) {
        let left = self.read_operand(op, op.first);
        let right = self.read_operand(op, op.second);
        match op.s << 1 | op.w {
            0b00 => {
                let res = (left & 0xff) as i32 - (right & 0xff) as i32;
                self.flag
                    .set_cosz(left < right, res > 0xff, res < 0, res == 0);
            }
            0b01 => {
                let res_u16 = (left as i16 - right as i16) as u16;
                let res = left as i32 - right as i32;
                self.flag
                    .set_cosz(left < right, res > 0xffff, res < 0, res_u16 == 0);
            }
            0b11 => {
                if op.second == OperandType::Imm {
                    let res_i16 = left as i16 - right as i8 as i16;
                    let res = left as i16 as i32 - right as i8 as i32;
                    self.flag
                        .set_cosz(left < right, res > 0xffff, res < 0, res_i16 == 0);
                } else {
                    let res_i16 = left as i16 - right as i16;
                    let res = left as i16 as i32 - right as i16 as i32;
                    self.flag
                        .set_cosz(left < right, res > 0xffff, res < 0, res_i16 == 0);
                }
            }
            _ => unreachable!("Invalid operand width"),
        };
    }

    fn shl(&mut self, op: &Operation) {
        let left = self.read_operand(op, op.first);
        match op.v << 1 | op.w {
            0b00 => {
                let left = left as u8;
                let res = (left << 1) as i32;
                let res_i8 = res as i8;
                let carry = carry_lsh_u8(left, 1);
                let overflow = ((res_i8 >> 7) & 1 == 1) != carry;
                self.flag.set_cosz(carry, overflow, res_i8 < 0, res_i8 == 0);
                self.write_operand(op, op.first, res_i8 as u16);
            }
            0b01 => {
                let res = (left << 1) as i32;
                let res_i16 = res as i16;
                let carry = carry_lsh_u16(left, 1 as u16);
                let overflow = ((res_i16 >> 15) & 1 == 1) != carry;
                self.flag
                    .set_cosz(carry, overflow, res_i16 < 0, res_i16 == 0);
                self.write_operand(op, op.first, res_i16 as u16);
            }
            0b10 => {
                let cl = self.register.cl;
                if cl == 0 {
                    // Do nothing if CL is 0
                }
                let left = left as u8;
                let num_shift = cl & 0x1f;
                let res = (left << num_shift) as i32;
                let res_i8 = res as i8;
                let carry = carry_lsh_u8(left, 1);
                let overflow = ((res_i8 >> 7) & 1 == 1) != carry;
                self.flag.set_cosz(carry, overflow, res_i8 < 0, res_i8 == 0);
                self.write_operand(op, op.first, res_i8 as u16);
            }
            0b11 => {
                let cl = self.register.cl;
                if cl == 0 {
                    // Do nothing if CL is 0
                }
                let num_shift = cl & 0x1f;
                let res = (left << num_shift) as i32;
                let res_i16 = res as i16;
                let carry = carry_lsh_u16(left, num_shift as u16);
                let overflow = ((res_i16 >> 15) & 1 == 1) != carry;
                self.flag
                    .set_cosz(carry, overflow, res_i16 < 0, res_i16 == 0);
                self.write_operand(op, op.first, res_i16 as u16);
            }
            _ => unreachable!("Invalid operation"),
        };
    }

    fn sar(&mut self, op: &Operation) {
        let left = self.read_operand(op, op.first);
        match op.v << 1 | op.w {
            0b00 => {
                let left = left as u8;
                let res = (left as i8 >> 1) as i32;
                let res_i8 = res as i8;
                let carry = carry_rsh_u16(left as u16, 1);
                self.flag.set_cosz(carry, false, res_i8 < 0, res_i8 == 0);
                self.write_operand(op, op.first, res_i8 as u16);
            }
            0b01 => {
                let res = (left as i16 >> 1) as i32;
                let res_i16 = res as i16;
                let carry = carry_rsh_u16(left, 1);
                self.flag.set_cosz(carry, false, res_i16 < 0, res_i16 == 0);
                self.write_operand(op, op.first, res_i16 as u16);
            }
            0b10 => {
                let cl = self.register.cl;
                if cl == 0 {
                    // Do nothing if CL is 0
                    return;
                }
                let num_shift = cl & 0x1f;
                let res = (left as i16 >> num_shift) as i32;
                let res_i16 = res as i16;
                let carry = carry_rsh_u16(left, num_shift as u16);
                self.flag.set_cosz(carry, false, res_i16 < 0, res_i16 == 0);
                self.write_operand(op, op.first, res_i16 as u16);
            }
            0b11 => {
                let cl = self.register.cl;
                if cl == 0 {
                    // Do nothing if CL is 0
                    return;
                }
                let num_shift = cl & 0x1f;
                let res = (left >> num_shift) as i32;
                let res_i16 = res as i16;
                let carry = carry_rsh_u16(left, num_shift as u16);
                self.flag.set_cosz(carry, false, res_i16 < 0, res_i16 == 0);
                self.write_operand(op, op.first, res_i16 as u16);
            }
            _ => unreachable!("Invalid operation"),
        };
    }

    fn jmp(&mut self, op: &Operation) {
        let addr = self.calc_effective_address(op);
        if addr >= self.memory.len() {
            panic!("Memory access out of bounds at address {}", addr);
        }
        self.register.ip = addr as u16;
    }

    fn je(&mut self, op: &Operation) {
        if self.flag.zero {
            let addr = self.calc_effective_address(op);
            if addr >= self.memory.len() {
                panic!("Memory access out of bounds at address {}", addr);
            }
            self.register.ip = addr as u16;
        }
    }

    fn jnl(&mut self, op: &Operation) {
        if !self.flag.sign {
            let addr = self.calc_effective_address(op);
            if addr >= self.memory.len() {
                panic!("Memory access out of bounds at address {}", addr);
            }
            self.register.ip = addr as u16;
        }
    }

    fn jnb(&mut self, op: &Operation) {
        if !self.flag.carry {
            let addr = self.calc_effective_address(op);
            if addr >= self.memory.len() {
                panic!("Memory access out of bounds at address {}", addr);
            }
            self.register.ip = addr as u16;
        }
    }

    fn jne(&mut self, op: &Operation) {
        if !self.flag.zero {
            let addr = self.calc_effective_address(op);
            if addr >= self.memory.len() {
                panic!("Memory access out of bounds at address {}", addr);
            }
            self.register.ip = addr as u16;
        }
    }

    fn jl(&mut self, op: &Operation) {
        if self.flag.sign {
            let addr = self.calc_effective_address(op);
            if addr >= self.memory.len() {
                panic!("Memory access out of bounds at address {}", addr);
            }
            self.register.ip = addr as u16;
        }
    }

    fn jb(&mut self, op: &Operation) {
        if self.flag.carry {
            let addr = self.calc_effective_address(op);
            if addr >= self.memory.len() {
                panic!("Memory access out of bounds at address {}", addr);
            }
            self.register.ip = addr as u16;
        }
    }

    fn jbe(&mut self, op: &Operation) {
        if self.flag.carry || self.flag.zero {
            let addr = self.calc_effective_address(op);
            if addr >= self.memory.len() {
                panic!("Memory access out of bounds at address {}", addr);
            }
            self.register.ip = addr as u16;
        }
    }

    fn jle(&mut self, op: &Operation) {
        if self.flag.sign || self.flag.zero {
            let addr = self.calc_effective_address(op);
            if addr >= self.memory.len() {
                panic!("Memory access out of bounds at address {}", addr);
            }
            self.register.ip = addr as u16;
        }
    }

    fn jnbe(&mut self, op: &Operation) {
        if !self.flag.carry && !self.flag.zero {
            let addr = self.calc_effective_address(op);
            if addr >= self.memory.len() {
                panic!("Memory access out of bounds at address {}", addr);
            }
            self.register.ip = addr as u16;
        }
    }

    fn jnle(&mut self, op: &Operation) {
        if !self.flag.sign && !self.flag.zero {
            let addr = self.calc_effective_address(op);
            if addr >= self.memory.len() {
                panic!("Memory access out of bounds at address {}", addr);
            }
            self.register.ip = addr as u16;
        }
    }

    fn test(&mut self, op: &Operation) {
        let left = self.read_operand(op, op.first);
        let right = self.read_operand(op, op.second);
        let result = match op.w {
            0 => (left as u8 & right as u8) as u16,
            1 => left & right,
            _ => unreachable!("Invalid operand width"),
        };
        self.flag
            .set_cosz(false, false, (result as i16) < 0, result == 0);
    }

    fn cbw(&mut self) {
        let al = self.register.al as i8;
        let ah = if al < 0 { 0xff } else { 0x00 };
        self.register.ah = ah as u8;
    }

    fn cwd(&mut self) {
        let ax = self.register.get_ax() as i16;
        let dx = if ax < 0 { 0xffff } else { 0x0000 };
        self.register.set_dx(dx);
    }

    fn xchg(&mut self, op: &Operation) {
        let left = self.read_operand(op, op.first);

        if op.second == OperandType::None {
            // XCHG with AX
            let right = self.register.get_ax();
            self.register.set_ax(left);
            self.write_operand(op, op.first, right);
            return;
        } else {
            let right = self.read_operand(op, op.second);
            dbg!(left, right);
            self.write_operand(op, op.first, right);
            self.write_operand(op, op.second, left);
        }
    }

    pub fn run(&mut self) {
        let mut disassembler =
            disassembler::Disassembler::new(self.text.clone(), &self.metadata, self.dump.enabled);
        self.dump.labels();

        let mut debug_count = 0;

        loop {
            if self.stop || self.register.ip > self.metadata.text_size as u16 {
                break;
            }
            self.dump.state(&self.register, &self.flag);
            let op = match disassembler.next(self.register.ip) {
                Some(op) => op,
                None => break,
            };
            self.register.ip = op.get_next_operation_pos() as u16;
            match op.operation_type {
                OperationType::Mov => self.mov(&op),
                OperationType::Add => self.add(&op),
                OperationType::Sub => self.sub(&op),
                OperationType::Int => self.int(),
                OperationType::Push => self.push(&op),
                OperationType::Pop => self.pop(&op),
                OperationType::Call => self.call(&op),
                OperationType::Jmp => self.jmp(&op),
                OperationType::Lea => self.lea(&op),
                OperationType::Ret => self.ret(&op),
                OperationType::Or => self.or(&op),
                OperationType::JeJz => self.je(&op),
                OperationType::Cmp => self.cmp(&op),
                OperationType::JnlJge => self.jnl(&op),
                OperationType::Xor => self.xor(&op),
                OperationType::JnbJae => self.jnb(&op),
                OperationType::Test => self.test(&op),
                OperationType::JneJnz => self.jne(&op),
                OperationType::Dec => self.dec(&op),
                OperationType::JlJnge => self.jl(&op),
                OperationType::Cbw => self.cbw(),
                OperationType::Inc => self.inc(&op),
                OperationType::And => self.and(&op),
                OperationType::JbJnae => self.jb(&op),
                OperationType::JleJng => self.jle(&op),
                OperationType::JnbeJa => self.jnbe(&op),
                OperationType::ShlSal => self.shl(&op),
                OperationType::Sar => self.sar(&op),
                OperationType::JnleJg => self.jnle(&op),
                OperationType::Cwd => self.cwd(),
                OperationType::Div => self.div(&op),
                OperationType::Xchg => self.xchg(&op),
                OperationType::Neg => self.neg(&op),
                OperationType::JbeJna => self.jbe(&op),
                OperationType::Undefined => {
                    panic!("\nUndefined operation: {:?}", op.operation_type);
                }
                _ => {
                    println!("\nUnknown operation: {:?}", op.operation_type);
                    self.stop = true;
                }
            }
            self.dump.eol();

            if self.stop_count > 0 {
                debug_count += 1;
                if debug_count >= self.stop_count {
                    self.stop = true;
                    println!("\nStopping execution after {} operations", debug_count);
                }
            }
        }
    }

    fn exit(&mut self, status: u16) {
        self.dump.exit(status);
        self.stop = true;
    }

    fn write(&self, fd: u16, addr: usize, len: u16) {
        if addr >= self.metadata.total {
            panic!("Memory access out of bounds at address {}", addr);
        }
        let data = self.get_data_segment();
        let begin = addr;
        let end = begin + len as usize;
        let str = String::from_utf8(data[begin..end].to_vec())
            .unwrap_or_else(|_| panic!("Failed to convert memory to string at address {}", addr));
        self.dump.write(fd, addr, len);
        print!("{}", str);
    }

    fn ioctl(&self, fd: u16, req: u16, addr: u16) {
        self.dump.ioctl(fd, req, addr);
    }

    fn brk(&mut self, addr: u16) -> bool {
        let ok = !(addr < self.metadata.data_size as u16
            || addr >= ((self.register.sp & !0x3ff) - 0x400));
        self.dump.brk(addr, ok);
        ok
    }

    fn stack_push_u16(&mut self, value: u16) {
        let value = value.to_le_bytes();
        let sp = self.register.sp as usize;
        let sp_new = sp
            .checked_sub(value.len())
            .unwrap_or_else(|| panic!("Stack overflow: SP is too low to push value"));
        self.memory[sp_new..sp].copy_from_slice(&value);
        self.register.sp = sp_new as u16;
    }
    fn stack_pop_u16(&mut self) -> u16 {
        let sp = self.register.sp as usize;
        let sp_new = sp
            .checked_add(2)
            .unwrap_or_else(|| panic!("Stack overflow: SP is too high to pop value"));
        let value = u16::from_le_bytes(
            self.memory[sp..sp_new]
                .try_into()
                .expect("Failed to read value from stack"),
        );
        self.register.sp = sp_new as u16;
        value
    }

    // fn stack_push_u8(&mut self, value: u8) {
    //     let sp = self.register.sp as usize;
    //     let sp_new = sp
    //         .checked_sub(1)
    //         .unwrap_or_else(|| panic!("Stack overflow: SP is too low to push value"));
    //     self.memory[sp_new] = value;
    //     self.register.sp = sp_new as u16;
    // }
    // fn stack_pop_u8(&mut self) -> u8 {
    //     let sp = self.register.sp as usize;
    //     let sp_new = sp
    //         .checked_add(1)
    //         .unwrap_or_else(|| panic!("Stack overflow: SP is too high to pop value"));
    //     let value = self.memory[sp];
    //     self.register.sp = sp_new as u16;
    //     value
    // }
}
