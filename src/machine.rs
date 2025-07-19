use crate::{
    disassembler,
    message::Message,
    metadata::{self},
    operation::{OperandType, Operation, OperationType},
    register::{self, Register, Register16Bit, RegisterType},
};

pub struct Machine {
    program_counter: usize,
    stop: bool,
    text: Vec<u8>,
    data: Vec<u8>,
    memory: Vec<u8>,
    register: Register,
    metadata: metadata::Metadata,
}

impl Machine {
    pub fn new<R: std::io::Read>(mut reader: R, metadata: metadata::Metadata) -> Self {
        let mut text = vec![0; metadata.text as usize];
        reader
            .read_exact(&mut text)
            .expect("Failed to read text segment");

        let mut data: Vec<u8> = vec![0; metadata.data as usize];
        reader
            .read_exact(&mut data)
            .expect("Failed to read data segment");

        Machine {
            program_counter: 0,
            stop: false,
            text,
            data,
            memory: vec![0; metadata.total as usize],
            register: Register::new(),
            metadata,
        }
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
                    0b000 => {
                        self.register.get(RegisterType::Word(Register16Bit::BX))
                            + self.register.get(RegisterType::Word(Register16Bit::SI))
                    }
                    0b001 => {
                        self.register.get(RegisterType::Word(Register16Bit::BX))
                            + self.register.get(RegisterType::Word(Register16Bit::DI))
                    }
                    0b010 => {
                        self.register.get(RegisterType::Word(Register16Bit::BP))
                            + self.register.get(RegisterType::Word(Register16Bit::SI))
                    }
                    0b011 => {
                        self.register.get(RegisterType::Word(Register16Bit::BP))
                            + self.register.get(RegisterType::Word(Register16Bit::DI))
                    }
                    0b100 => self.register.get(RegisterType::Word(Register16Bit::SI)),
                    0b101 => self.register.get(RegisterType::Word(Register16Bit::DI)),
                    0b110 => self.register.get(RegisterType::Word(Register16Bit::BP)),
                    0b111 => self.register.get(RegisterType::Word(Register16Bit::BX)),
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
            OperandType::Imm => op.data,
            OperandType::EA => {
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
            _ => unreachable!(),
        }
    }

    fn write_operand(&mut self, op: &Operation, operand: OperandType, value: u16) {
        match operand {
            OperandType::Reg => self.register.set(op.get_register(), value),
            OperandType::Imm => panic!("Immediate value cannot be used as a destination"),
            OperandType::EA => {
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
        let result = left - right;
        self.write_operand(op, op.first, result);
    }

    fn int(&mut self, _: &Operation) {
        let msg = Message::load(
            &self.data,
            self.register.get(RegisterType::Word(Register16Bit::BX)) as usize,
        );
        match msg.message_type {
            1 => {
                // Exit
                let status = msg.load_detail1(&self.data).detail[0];
                self.exit(status);
            }
            4 => {
                // Write
                let detail = msg.load_detail1(&self.data);
                let fd = detail.m1i1();
                let addr = detail.m1p1();
                let len = detail.m1i2();
                self.write(fd, addr, len);
            }
            _ => {
                println!("Unhandled interrupt type: {}", msg.message_type);
            }
        }
    }

    pub fn run(&mut self) {
        let mut disassembler =
            disassembler::Disassembler::new(self.text.clone(), &self.metadata, true);
        self.program_counter = 0;
        loop {
            if self.stop {
                break;
            }
            let op = match disassembler.next() {
                Some(op) => op,
                None => break,
            };
            match op.operation_type {
                OperationType::Mov => self.mov(op),
                OperationType::Add => self.add(op),
                OperationType::Sub => self.sub(op),
                OperationType::Int => self.int(op),
                _ => {
                    println!("Unknown operation: {:?}", op);
                }
            }
            self.program_counter += 1;
        }
    }

    pub fn exit(&mut self, status: u16) {
        println!("<exit({})>", status);
        self.stop = true;
    }

    pub fn write(&self, fd: u16, addr: u16, len: u16) {
        if addr as usize >= self.data.len() {
            panic!("Memory access out of bounds at address {}", addr);
        }
        let begin = addr as usize;
        let end = begin + len as usize;
        let str = String::from_utf8(self.data[begin..end].to_vec())
            .unwrap_or_else(|_| panic!("Failed to convert memory to string at address {}", addr));
        print!(
            "<write(fd={}, addr=0x{:04x}, len={})>{}",
            fd, addr, len, str
        );
    }
}
