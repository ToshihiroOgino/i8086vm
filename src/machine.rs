use crate::{
    disassembler,
    dump::Dump,
    flag::Flag,
    message::Message,
    metadata::{self},
    operation::{OperandType, Operation, OperationType},
    register::{self, Register, Register16Bit, RegisterType},
};

pub struct Machine {
    stop: bool,
    memory: Vec<u8>,
    register: Register,
    metadata: metadata::Metadata,
    flag: Flag,
    dump: Dump,
}

impl Machine {
    pub fn new<R: std::io::Read>(
        mut reader: R,
        metadata: metadata::Metadata,
        args: &[String],
        envs: &[String],
    ) -> Self {
        let mut text = vec![0; metadata.text_size];
        reader
            .read_exact(&mut text)
            .expect("Failed to read text segment");

        let mut data: Vec<u8> = vec![0; metadata.data_size];
        reader
            .read_exact(&mut data)
            .expect("Failed to read data segment");

        let mut memory = vec![0; metadata.total];
        memory[0..metadata.text_size].copy_from_slice(&text);
        memory[metadata.text_size..metadata.text_size + metadata.data_size].copy_from_slice(&data);

        let args_frame = Self::create_args_frame(args, envs, metadata.total);
        let frame_base = metadata.total - args_frame.len();
        memory[frame_base..metadata.total].copy_from_slice(&args_frame);

        let mut register = Register::new();
        register.sp = frame_base as u16;

        Machine {
            stop: false,
            memory,
            register,
            metadata,
            flag: Flag::new(),
            dump: Dump::new(true),
        }
    }

    fn create_args_frame(args: &[String], envs: &[String], total_memory: usize) -> Vec<u8> {
        // argc(u16) + args_address(u16) * n + 0(u16) + env_address(u16) * m
        let mut frame_size = 2 + args.len() * 2 + 2 + envs.len() * 2;

        let mut args_offset = Vec::new();
        let mut args_seg = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            let buf = arg.as_bytes();
            args_offset.push(frame_size + i * 2);
            args_seg.extend_from_slice(buf);
            args_seg.push(0);
            frame_size += buf.len() + 1;
        }

        let mut env_offset = Vec::new();
        let mut env_seg: Vec<u8> = Vec::new();
        for (i, env) in envs.iter().enumerate() {
            env_offset.push(frame_size + i * 2);
            env_seg.extend_from_slice(env.as_bytes());
            env_seg.push(0);
            frame_size += env.as_bytes().len() + 1;
        }

        let frame_base = total_memory - frame_size;
        let mut frame = Vec::new();

        // argc
        frame.extend_from_slice(&(args.len() as u16).to_le_bytes());
        // args address
        for offset in args_offset {
            let addr = (frame_base + offset) as u16;
            frame.extend_from_slice(&addr.to_le_bytes());
        }
        // 0u16
        frame.extend_from_slice(&0u16.to_le_bytes());

        // env address
        for offset in env_offset {
            let addr = (frame_base + offset) as u16;
            frame.extend_from_slice(&addr.to_le_bytes());
        }
        // 0u16
        frame.extend_from_slice(&0u16.to_le_bytes());

        // args string
        frame.extend_from_slice(&args_seg);
        // env string
        frame.extend_from_slice(&env_seg);

        // padding
        // frame.extend_from_slice(&0u16.to_le_bytes());
        frame.push(0);

        frame
    }

    fn calc_effective_address(&self, op: &Operation) -> usize {
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
                    0b000 => self.register.bx + self.register.si,
                    0b001 => self.register.bx + self.register.di,
                    0b010 => self.register.bp + self.register.si,
                    0b011 => self.register.bp + self.register.di,
                    0b100 => self.register.si,
                    0b101 => self.register.di,
                    0b110 => self.register.bp,
                    0b111 => self.register.bx,
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
                if addr >= self.memory.len() {
                    panic!("Memory access out of bounds at address {}", addr);
                }
                if op.w == 0 {
                    self.memory[addr] as u16
                } else {
                    if addr + 1 >= self.memory.len() {
                        panic!("Memory access out of bounds at address {}", addr + 1);
                    }
                    u16::from_le_bytes([self.memory[addr], self.memory[addr + 1]])
                }
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

    fn get_text_segment(&self) -> &[u8] {
        &self.memory[0..self.metadata.text_size]
    }
    fn get_data_segment(&self) -> &[u8] {
        &self.memory[self.metadata.text_size..self.metadata.text_size + self.metadata.data_size]
    }

    fn mov(&mut self, op: &Operation) {
        let val = self.read_operand(op, op.second);
        self.write_operand(op, op.first, val);
    }

    fn add(&mut self, op: &Operation) {
        let left = self.read_operand(op, op.first);
        let right = self.read_operand(op, op.second);
        let result = left + right;
        self.write_operand(op, op.first, result);
    }

    fn sub(&mut self, op: &Operation) {
        let left = self.read_operand(op, op.first);
        let right = self.read_operand(op, op.second);
        let res_i32: i32 = left as i32 - right as i32;
        let left = match (op.s << 1 | op.w) {
            0b00 => {
                // 1 byte
                let res_u8 = left as u8 - right as u8;
                self.flag.zero = res_u8 == 0;
                self.flag.sign = false;
                self.flag.overflow = res_u8 as i32 != res_i32;
                self.flag.carry = left < right;
                res_u8 as u16
            }
            0b01 => {
                // 2 bytes
                let res_i16 = left as i16 - right as i16;
                self.flag.zero = res_i16 == 0;
                self.flag.sign = res_i16 < 0;
                self.flag.overflow = res_i16 as i32 != res_i32;
                self.flag.carry = left < right;
                res_i16 as u16
            }
            0b11 => {
                // signed
                let res_i16 = left as i16 - right as u8 as i16;
                self.flag.zero = res_i16 == 0;
                self.flag.sign = res_i16 < 0;
                self.flag.overflow = res_i16 as i32 != res_i32;
                self.flag.carry = left < right as u8 as u16;
                res_i16 as u16
            }
            _ => unreachable!("Invalid operand width"),
        };
        let result = left - right;
        self.write_operand(op, op.first, result);
    }

    fn int(&mut self, _: &Operation) {
        let msg = Message::load(self.get_data_segment(), self.register.bx as usize);
        match msg.message_type {
            1 => {
                // Exit
                let status = msg.load_detail1(self.get_data_segment()).detail[0];
                self.exit(status);
            }
            4 => {
                // Write
                let detail = msg.load_detail1(self.get_data_segment());
                let fd = detail.m1i1();
                let addr = detail.m1p1();
                let len = detail.m1i2();
                self.write(fd, addr as usize, len);
            }
            _ => {
                println!("Unhandled interrupt type: {}", msg.message_type);
            }
        }
    }

    fn push(&mut self, op: &Operation) {
        let value = self.read_operand(op, op.first).to_le_bytes();
        self.stack_push(&value);
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
        let return_addr = self.register.ip + op.raws.len() as u16;
        self.stack_push(&return_addr.to_le_bytes());
        self.register.ip = addr as u16;
    }

    fn jmp(&mut self, op: &Operation) {
        let addr = self.calc_effective_address(op);
        if addr >= self.memory.len() {
            panic!("Memory access out of bounds at address {}", addr);
        }
        self.register.ip = addr as u16;
    }

    pub fn run(&mut self) {
        let mut disassembler =
            disassembler::Disassembler::new(self.get_text_segment().to_vec(), &self.metadata, true);
        self.dump.labels();
        loop {
            if self.stop {
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
                OperationType::Int => self.int(&op),
                OperationType::Push => self.push(&op),
                OperationType::Pop => self.pop(&op),
                OperationType::Call => self.call(&op),
                OperationType::Jmp => self.jmp(&op),
                _ => {
                    println!("Unknown operation: {:?}", op.operation_type);
                    self.stop = true;
                }
            }
        }
    }

    pub fn exit(&mut self, status: u16) {
        println!("<exit({})>", status);
        self.stop = true;
    }

    pub fn write(&self, fd: u16, addr: usize, len: u16) {
        if addr >= self.metadata.data_size {
            panic!("Memory access out of bounds at address {}", addr);
        }
        let data = self.get_data_segment();
        let begin = addr;
        let end = begin + len as usize;
        let str = String::from_utf8(data[begin..end].to_vec())
            .unwrap_or_else(|_| panic!("Failed to convert memory to string at address {}", addr));
        print!(
            "<write(fd={}, addr=0x{:04x}, len={})>{}",
            fd, addr, len, str
        );
    }

    fn stack_push(&mut self, value: &[u8]) {
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
}
