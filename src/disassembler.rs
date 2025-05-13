use core::panic;
use std::{
    fs::File,
    io::{BufReader, Read},
};

use crate::{
    metadata::Metadata,
    operation::{Operation, OperationType},
};

fn read_bytes(reader: &mut BufReader<File>, bytes: usize) -> Vec<u8> {
    let mut buffer = vec![0; bytes];
    reader
        .read_exact(&mut buffer)
        .expect("Failed to read bytes");
    buffer
}

fn decode_instruction(data: &[u8]) {
    let mut current_pos = 0;
    while (current_pos < data.len()) {
        match data[current_pos] {
            // MOV
            0x88 | 0x89 | 0x8A | 0x8B => {}
            // INT
            0xCD => {}
            // Unknown
            _ | 0x00 => {}
        };
    }
}

pub struct Disassembler {
    metadata: Metadata,
    text: Vec<u8>,
    text_pos: usize,
}

impl Disassembler {
    pub fn new(target_path: &str) -> Self {
        let file = File::open(target_path).expect("Failed to open file");
        let mut reader = BufReader::new(file);

        let header = read_bytes(&mut reader, 32);
        let metadata =
            Metadata::from_bytes(header.try_into().expect("Failed to read metadata section"));
        dbg!(&metadata);

        let text = read_bytes(&mut reader, metadata.text as usize);
        Disassembler {
            metadata,
            text,
            text_pos: 0,
        }
    }

    fn next_byte(&mut self) -> u8 {
        let byte = self.text[self.text_pos];
        self.text_pos += 1;
        byte
    }

    fn next_operation(&mut self) -> Operation {
        let mut op = Operation::new();
        let instruction = self.next_byte();
        op.opcode = instruction;
        match instruction {
            // --- Data Transfer ---
            // Mov
            0b1000_1000..=0b1000_1011 => {
                // Register/Memory to/from Register
                op.operation_type = OperationType::Mov;
                let mod_reg_rm = self.next_byte();
                op.set_mod_reg_rm(mod_reg_rm);
                op.d = (instruction >> 1) & 1;
                op.w = instruction & 1;
            }
            0b1100_0110 | 0b1100_0111 => {
                // Immediate to Register/Memory
                op.operation_type = OperationType::Mov;
                let mod_reg_rm = self.next_byte();
                op.set_mod_reg_rm(mod_reg_rm);
                op.w = instruction & 1;
                if op.w == 1 {
                    // 16-bit immediate
                    op.data = u16::from_le_bytes([self.next_byte(), self.next_byte()]);
                } else {
                    // 8-bit immediate
                    op.data = self.next_byte() as u16;
                }
            }
            0b1011_0000..=0b1011_1111 => {
                // Immediate to Register
                op.operation_type = OperationType::Mov;
                op.w = instruction >> 3 & 1;
                op.reg = instruction & 0b111;
                if op.w == 1 {
                    // 16-bit immediate
                    op.data = u16::from_le_bytes([self.next_byte(), self.next_byte()]);
                } else {
                    // 8-bit immediate
                    op.data = self.next_byte() as u16;
                }
            }
            0b1010_0000 | 0b1010_0001 => {
                // Memory to Accumulator
                // Accumulator to Memory
                op.operation_type = OperationType::Mov;
                op.w = instruction & 1;
                op.addr_low = self.next_byte();
                op.addr_high = self.next_byte();
            }
            0b1000_1110 | 0b1000_1100 => {
                // Register to Segment Register
                // Segment Register to Register
                op.operation_type = OperationType::Mov;
                let mod_reg_rm = self.next_byte();
                op.set_mod_reg_rm(mod_reg_rm);
                if op.reg & 0b100 != 0 {
                    panic!("Invalid operation. reg must be 0b0xx in this operation");
                }
            }
            // Push
            0b1111_1111 => {
                // Register/Memory
                op.operation_type = OperationType::Push;
                op.set_mod_reg_rm(self.next_byte());
                if op.reg != 0b110 {
                    panic!("Invalid operation. reg must be 0b110 in this operation");
                }
            }
            0b101_0000..=0b101_0111 => {
                // Register
                op.operation_type = OperationType::Push;
                op.reg = instruction & 0b111;
            }
            0b0000_0110 | 0b0000_1110 | 0b0001_0110 | 0b0001_1110 => {
                // Segment Register
                op.operation_type = OperationType::Push;
                op.reg = instruction >> 3 & 0b111;
            }
            // Pop
            0b1000_1111 => {
                // Register/Memory
                op.operation_type = OperationType::Pop;
                op.set_mod_reg_rm(self.next_byte());
                if op.reg != 0b110 {
                    panic!("Invalid operation. reg must be 0b110 in this operation");
                }
            }
            0b0101_100..=0b0101_1111 => {
                // Register
                op.operation_type = OperationType::Pop;
                op.reg = instruction & 0b111;
            }
            0b0000_0111 | 0b0000_1111 | 0b0001_0111 | 0b0001_1111 => {
                // Segment Register
                op.operation_type = OperationType::Pop;
                op.reg = instruction >> 3 & 0b111;
            }
            // Xchg
            _ => {
                // println!("Unknown operation: {:#X}", op);
            }
        }
        op
    }

    pub fn disassemble(&mut self) {
        while self.text_pos < self.text.len() {}
    }
}
