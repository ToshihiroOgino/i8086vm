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
                    op.data = u16::from_le_bytes([self.next_byte(), self.next_byte()]);
                } else {
                    op.data = self.next_byte() as u16;
                }
            }
            0b1011_0000..=0b1011_1111 => {
                // Immediate to Register
                op.operation_type = OperationType::Mov;
                op.w = instruction >> 3 & 1;
                op.reg = instruction & 0b111;
                if op.w == 1 {
                    op.data = u16::from_le_bytes([self.next_byte(), self.next_byte()]);
                } else {
                    op.data = self.next_byte() as u16;
                }
            }
            0b1010_0000 | 0b1010_0001 => {
                // Memory to Accumulator
                // Accumulator to Memory
                op.operation_type = OperationType::Mov;
                op.w = instruction & 1;
                op.low = self.next_byte();
                op.high = self.next_byte();
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
            0b0101_1000..=0b0101_1111 => {
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
            0b1000_0110 | 0b1000_0111 => {
                // Register/Memory with Register
                op.operation_type = OperationType::Xchg;
                op.set_mod_reg_rm(self.next_byte());
                op.w = instruction & 1;
            }
            0b1001_0000..=0b1001_0111 => {
                // Register with Accumulator
                op.operation_type = OperationType::Xchg;
                op.reg = instruction & 0b111;
            }
            // In
            0b1110_0100 | 0b1110_0101 => {
                // Fixed Port
                op.operation_type = OperationType::In;
                op.w = instruction & 1;
                op.port = self.next_byte();
            }
            0b1110_1100 | 0b1110_1101 => {
                // Variable Port
                op.operation_type = OperationType::In;
                op.w = instruction & 1;
            }
            // Out
            0b1110_0110 | 0b1110_0111 => {
                // Fixed Port
                op.operation_type = OperationType::Out;
                op.w = instruction & 1;
                op.port = self.next_byte();
            }
            0b1110_1110 | 0b1110_1111 => {
                // Variable Port
                op.operation_type = OperationType::Out;
                op.w = instruction & 1;
            }
            // Xlat
            0b1101_0111 => {
                op.operation_type = OperationType::Xlat;
            }
            // Lea
            0b1000_1101 => {
                op.operation_type = OperationType::Lea;
                op.set_mod_reg_rm(self.next_byte());
            }
            // Lds
            0b1100_0101 => {
                op.operation_type = OperationType::Lds;
                op.set_mod_reg_rm(self.next_byte());
            }
            // Les
            0b1100_0100 => {
                op.operation_type = OperationType::Les;
                op.set_mod_reg_rm(self.next_byte());
            }
            // Lahf
            0b1001_1111 => {
                op.operation_type = OperationType::Lahf;
            }
            // Sahf
            0b1001_1110 => {
                op.operation_type = OperationType::Sahf;
            }
            // Pushf
            0b1001_1100 => {
                op.operation_type = OperationType::Pushf;
            }
            // Popf
            0b1001_1101 => {
                op.operation_type = OperationType::Popf;
            }

            // --- Arithmetic ---
            // Add
            0b0000_0000..=0b0000_0011 => {
                // Register/Memory with Register to Either
                op.operation_type = OperationType::Add;
                op.set_mod_reg_rm(self.next_byte());
                op.d = (instruction >> 1) & 1;
                op.w = instruction & 1;
            }
            0b0000_0100 | 0b0000_0101 => {
                // Immediate to Accumulator
                op.operation_type = OperationType::Add;
                op.w = instruction & 1;
                if op.w == 1 {
                    op.data = u16::from_le_bytes([self.next_byte(), self.next_byte()]);
                } else {
                    op.data = self.next_byte() as u16;
                }
            }
            // Adc
            0b0001_0000..=0b0001_0011 => {
                // Register/Memory with Register to Either
                op.operation_type = OperationType::Adc;
                op.set_mod_reg_rm(self.next_byte());
                op.d = (instruction >> 1) & 1;
                op.w = instruction & 1;
            }
            0b0001_0100 | 0b0001_0101 => {
                // Immediate to Accumulator
                op.operation_type = OperationType::Adc;
                op.w = instruction & 1;
                op.data = if op.w == 1 {
                    u16::from_le_bytes([self.next_byte(), self.next_byte()])
                } else {
                    self.next_byte() as u16
                };
            }
            // Inc
            0b1111_1110 => {
                // Register/Memory
                op.operation_type = OperationType::Inc;
                op.set_mod_reg_rm(self.next_byte());
                op.w = instruction & 1;
            }
            0b0100_0000..=0b0100_0111 => {
                // Register
                op.operation_type = OperationType::Inc;
                op.reg = instruction & 0b111;
            }
            // Aaa
            0b0011_0111 => {
                op.operation_type = OperationType::Aaa;
            }
            // Baa
            0b0010_0111 => {
                op.operation_type = OperationType::Baa;
            }
            //Sub
            0b0010_1000..=0b0010_1011 => {
                // Register/Memory with Register to Either
                op.operation_type = OperationType::Sub;
                op.set_mod_reg_rm(self.next_byte());
                op.d = (instruction >> 1) & 1;
                op.w = instruction & 1;
            }
            0b0010_1100 | 0b0010_1101 => {
                // Immediate from Accumulator
                op.operation_type = OperationType::Sub;
                op.w = instruction & 1;
                op.data = if op.w == 1 {
                    u16::from_le_bytes([self.next_byte(), self.next_byte()])
                } else {
                    self.next_byte() as u16
                };
            }
            // Sbb
            0b0001_1000..=0b0001_1011 => {
                // Register/Memory with Register to Either
                op.operation_type = OperationType::Sbb;
                op.set_mod_reg_rm(self.next_byte());
                op.d = (instruction >> 1) & 1;
                op.w = instruction & 1;
            }
            0b0001_1100 | 0b0001_1101 => {
                // データシートの誤植
                // https://qiita.com/7shi/items/b3911948f9d97b05395e#%E4%BB%95%E6%A7%98%E6%9B%B8
                // Immediate from Accumulator
                op.operation_type = OperationType::Sbb;
                op.w = instruction & 1;
                op.data = if op.w == 1 {
                    u16::from_le_bytes([self.next_byte(), self.next_byte()])
                } else {
                    self.next_byte() as u16
                };
            }
            // Dec
            0b0100_1000..=0b0100_1111 => {
                // Register
                op.operation_type = OperationType::Dec;
                op.reg = instruction & 0b111;
            }
            // Cmp
            0b0011_1000..=0b0011_1011 => {
                // Register/Memory and Register
                op.operation_type = OperationType::Cmp;
                op.set_mod_reg_rm(self.next_byte());
                op.d = (instruction >> 1) & 1;
                op.w = instruction & 1;
            }
            0b0011_1100 | 0b0011_1101 => {
                // Immediate with Accumulator
                op.operation_type = OperationType::Cmp;
                op.w = instruction & 1;
                op.data = if op.w == 1 {
                    u16::from_le_bytes([self.next_byte(), self.next_byte()])
                } else {
                    self.next_byte() as u16
                }
            }
            // Aas
            0b0011_1111 => {
                op.operation_type = OperationType::Aas;
            }
            // Das
            0b0010_1111 => {
                op.operation_type = OperationType::Das;
            }
            // Cbw
            0b1001_1000 => {
                op.operation_type = OperationType::Cbw;
            }
            // Cwd
            0b1001_1001 => {
                op.operation_type = OperationType::Cwd;
            }

            // --- Logic ---
            // And
            0b0010_0000..=0b0010_0011 => {
                // Register/Memory and Register to Either
                op.operation_type = OperationType::And;
                op.set_mod_reg_rm(self.next_byte());
                op.d = (instruction >> 1) & 1;
                op.w = instruction & 1;
            }
            0b0010_0100 | 0b0010_0101 => {
                // Immediate with Accumulator
                op.operation_type = OperationType::And;
                op.w = instruction & 1;
                op.data = if op.w == 1 {
                    u16::from_le_bytes([self.next_byte(), self.next_byte()])
                } else {
                    self.next_byte() as u16
                };
            }
            // Test
            0b1000_0100 | 0b1000_0101 => {
                // Register/Memory and Register
                op.operation_type = OperationType::Test;
                op.set_mod_reg_rm(self.next_byte());
            }
            0b1010_1000 | 0b1010_1001 => {
                // Immediate Data and Accumulator
                op.operation_type = OperationType::Test;
                op.w = instruction & 1;
                op.data = if op.w == 1 {
                    u16::from_le_bytes([self.next_byte(), self.next_byte()])
                } else {
                    self.next_byte() as u16
                };
            }
            // Or
            0b0000_1000..=0b0000_1011 => {
                // Register/Memory and Register to Either
                op.operation_type = OperationType::Or;
                op.set_mod_reg_rm(self.next_byte());
                op.d = (instruction >> 1) & 1;
                op.w = instruction & 1;
            }
            0b0000_1100 | 0b0000_1101 => {
                // Immediate with Accumulator
                op.operation_type = OperationType::Or;
                op.w = instruction & 1;
                op.data = if op.w == 1 {
                    u16::from_le_bytes([self.next_byte(), self.next_byte()])
                } else {
                    self.next_byte() as u16
                };
            }
            // Xor
            0b0011_0000..=0b0011_0011 => {
                // Register/Memory and Register to Either
                op.operation_type = OperationType::Xor;
                op.set_mod_reg_rm(self.next_byte());
                op.d = (instruction >> 1) & 1;
                op.w = instruction & 1;
            }
            0b0011_0100 | 0b0011_0101 => {
                // Immediate with Accumulator
                op.operation_type = OperationType::Xor;
                op.w = instruction & 1;
                op.data = if op.w == 1 {
                    u16::from_le_bytes([self.next_byte(), self.next_byte()])
                } else {
                    self.next_byte() as u16
                };
            }

            // --- String Manipulation ---
            // Rep
            0b1111_0010 | 0b1111_0011 => {
                op.operation_type = OperationType::Rep;
                op.z = instruction & 1;
            }
            // Movs/Cmps/Scas/Lods/Stos
            0b1010_0100..=0b1010_1111 => {
                op.w = instruction & 1;
                op.operation_type = match instruction >> 1 & 0b111 {
                    0b010 => OperationType::Movs,
                    0b011 => OperationType::Cmps,
                    0b111 => OperationType::Scas,
                    0b110 => OperationType::Lods,
                    0b101 => OperationType::Stos,
                    _ => {
                        panic!("Invalid operation. reg is invalid");
                    }
                };
            }

            // --- Control Transfer ---
            // Call
            0b1110_1000 => {
                // Direct within Segment
                op.operation_type = OperationType::Call;
                op.low = self.next_byte();
                op.high = self.next_byte();
            }
            0b1001_1010 => {
                // Direct Intersegment
                op.operation_type = OperationType::Call;
                // offset-low offset-high
                // seg-low seg-high
                op.low = self.next_byte();
                op.high = self.next_byte();
            }
            // Jmp
            0b1110_1001 => {
                // Direct within Segment
                op.operation_type = OperationType::Jmp;
                op.low = self.next_byte();
                op.high = self.next_byte();
            }
            0b1110_1011 => {
                // Direct within Segment-Short
                op.operation_type = OperationType::Jmp;
                op.disp = self.next_byte();
            }
            0b1110_1010 => {
                op.operation_type = OperationType::Jmp;
                op.low = self.next_byte();
                op.high = self.next_byte();
            }
            // Ret
            0b1100_0011 | 0b1100_1011 => {
                // Within Segment
                // Intersegment
                op.operation_type = OperationType::Ret;
            }
            0b1100_0010 | 0b1100_1010 => {
                // Within Segment Adding Immed to Sp
                // Within Segment Adding Immediate to Sp
                op.operation_type = OperationType::Ret;
                op.low = self.next_byte();
                op.high = self.next_byte();
            }
            // Jump
            0b0111_0000..=0b0111_1111 => {
                op.operation_type = OperationType::Jump;
                op.disp = self.next_byte();
            }
            // Loop
            0b1110_0000..=0b1110_0010 => {
                op.operation_type = OperationType::Loop;
                op.disp = self.next_byte();
            }
            // Jump
            0b1110_0011 => {
                // Jump on CX Zero
                op.operation_type = OperationType::Jump;
                op.disp = self.next_byte();
            }
            // Int
            0b1100_1101 => {
                // Type Specified
                op.operation_type = OperationType::Int;
                op.int_type = self.next_byte();
            }
            0b11001100 => {
                // Type 3
                op.operation_type = OperationType::Int;
            }
            // Into
            0b1100_1110 => {
                op.operation_type = OperationType::Into;
            }
            // Iret
            0b1100_1111 => {
                op.operation_type = OperationType::Iret;
            }

            // --- Processor Control ---
            0b1111_1000 => {
                op.operation_type = OperationType::Clc;
            }
            0b1111_0101 => {
                op.operation_type = OperationType::Cmc;
            }
            0b1111_1001 => {
                op.operation_type = OperationType::Stc;
            }
            0b1111_1100 => {
                op.operation_type = OperationType::Cld;
            }
            0b1111_1101 => {
                op.operation_type = OperationType::Std;
            }
            0b1111_1010 => {
                op.operation_type = OperationType::Cli;
            }
            0b1111_1011 => {
                op.operation_type = OperationType::Sti;
            }
            0b1111_0100 => {
                op.operation_type = OperationType::Hlt;
            }
            0b1001_1011 => {
                op.operation_type = OperationType::Wait;
            }
            0b1101_1000..=0b1101_1111 => {
                op.operation_type = OperationType::Esc;
                op.reg = instruction & 0b111;
            }
            0b1111_0000 => {
                op.operation_type = OperationType::Lock;
            }

            // --- Common ---
            // Add/Adc/Sub/Ssb/Cmp/And
            0b1000_0000..=0b1000_0011 => {
                // Immediate to Register/Memory
                op.set_mod_reg_rm(self.next_byte());
                op.operation_type = OperationType::Add;
                op.operation_type = match op.reg {
                    0b000 => OperationType::Add,
                    0b010 => OperationType::Adc,
                    0b101 => OperationType::Sub,
                    0b011 => OperationType::Sbb,
                    0b111 => OperationType::Cmp,
                    0b100 => OperationType::And,
                    0b001 => OperationType::Or,
                    0b110 => OperationType::Xor,
                    _ => {
                        panic!("Invalid operation. reg is invalid");
                    }
                };
                op.s = (instruction >> 1) & 1;
                op.w = instruction & 1;
                if op.w == 1 {
                    op.data = u16::from_le_bytes([self.next_byte(), self.next_byte()]);
                } else {
                    op.data = self.next_byte() as u16;
                }
            }
            // Push/Inc/Dec/Call
            0b1111_1111 => {
                // Register/Memory
                op.set_mod_reg_rm(self.next_byte());
                op.operation_type = match op.reg {
                    0b110 => OperationType::Push,
                    0b000 => OperationType::Inc,
                    0b001 => OperationType::Dec,
                    0b010 | 0b011 => OperationType::Call,
                    0b100 | 0b101 => OperationType::Jmp,
                    _ => {
                        panic!("Invalid operation. reg is invalid");
                    }
                };
            }
            // Neg/Mul/Imul/Div/Idiv/Not
            0b1111_0110 | 0b1111_0111 => {
                op.set_mod_reg_rm(self.next_byte());
                op.w = instruction & 1;
                op.operation_type = match op.reg {
                    0b011 => OperationType::Neg,
                    0b100 => OperationType::Mul,
                    0b101 => OperationType::Imul,
                    0b110 => OperationType::Div,
                    0b111 => OperationType::Idiv,
                    0b010 => OperationType::Not,
                    0b000 => {
                        // Immediate Data and Register/Memory
                        op.data = if op.w == 1 {
                            // 16-bit immediate
                            u16::from_le_bytes([self.next_byte(), self.next_byte()])
                        } else {
                            // 8-bit immediate
                            self.next_byte() as u16
                        };
                        OperationType::Test
                    }
                    _ => {
                        panic!("Invalid operation. reg is invalid");
                    }
                };
            }
            // Aam
            0b1101_0100 => {
                let next = self.next_byte();
                op.operation_type = match next {
                    0b0000_1010 => OperationType::Aam,
                    _ => {
                        panic!("Invalid operation");
                    }
                }
            }
            // Aad
            0b1101_0101 => {
                let next = self.next_byte();
                op.operation_type = match next {
                    0b0000_1010 => OperationType::Aad,
                    _ => {
                        panic!("Invalid operation");
                    }
                }
            }
            // Shl/Sal/Shr/Sar/Rol/Ror/Rcl/Rcr
            0b1101_0000..=0b1101_0011 => {
                op.set_mod_reg_rm(self.next_byte());
                op.v = instruction >> 1 & 1;
                op.w = instruction & 1;
                op.operation_type = match op.reg {
                    0b100 => OperationType::ShlSal,
                    0b101 => OperationType::Shr,
                    0b111 => OperationType::Sar,
                    0b000 => OperationType::Rol,
                    0b001 => OperationType::Ror,
                    0b010 => OperationType::Rcl,
                    0b011 => OperationType::Rcr,
                    _ => {
                        panic!("Invalid operation. reg is invalid");
                    }
                };
            }

            _ => {
                panic!("Unknown operation: {:02x}", instruction);
            }
        }
        op
    }

    pub fn disassemble(&mut self) {
        while self.text_pos < self.text.len() {}
    }
}
