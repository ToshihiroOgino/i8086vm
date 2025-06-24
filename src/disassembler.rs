use core::panic;
use std::{
    fs::File,
    io::{stdout, BufReader, Read, Write},
};

use crate::{
    dump::Dump,
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
    text: Vec<u8>,
    text_pos: usize,
    dump: Dump,
}

impl Disassembler {
    pub fn new(target_path: &str) -> Self {
        let file = File::open(target_path).expect("Failed to open file");
        let mut reader = BufReader::new(file);

        let header = read_bytes(&mut reader, 32);
        let metadata =
            Metadata::from_bytes(header.try_into().expect("Failed to read metadata section"));

        let text = read_bytes(&mut reader, metadata.text as usize);
        Disassembler {
            text,
            text_pos: 0,
            dump: Dump::new(false),
        }
    }

    pub fn enable_dump(&mut self) {
        self.dump.enabled = true;
    }

    fn next_byte(&mut self, op: &mut Operation) -> u8 {
        let byte = self.text[self.text_pos];
        self.text_pos += 1;
        op.raws.push(byte);
        byte
    }

    fn disp(&mut self, op: &mut Operation) {
        match op.mod_rm {
            0b00 => {
                if op.rm == 0b110 {
                    op.disp = Some(u16::from_le_bytes([self.next_byte(op), self.next_byte(op)]));
                }
            }
            0b01 => {
                op.disp = Some(self.next_byte(op) as u16);
            }
            0b10 => {
                op.disp = Some(u16::from_le_bytes([self.next_byte(op), self.next_byte(op)]));
            }
            0b11 => {
                // No displacement
            }
            _ => {
                panic!("Invalid mod");
            }
        }
    }

    fn next_operation(&mut self) -> Operation {
        let mut op = Operation::new();
        op.pos = self.text_pos;
        let instruction = self.next_byte(&mut op);

        if self.text_pos >= self.text.len() && instruction == 0 {
            op.operation_type = OperationType::Undefined;
            self.dump.none(&op);
            return op;
        }

        match instruction {
            // --- Data Transfer ---
            // Mov
            0b1000_1000..=0b1000_1011 => {
                // Register/Memory to/from Register
                op.operation_type = OperationType::Mov;
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
                op.d = (instruction >> 1) & 1;
                op.w = instruction & 1;
                self.dump.simple_calc1(&op);
            }
            0b1100_0110 | 0b1100_0111 => {
                // Immediate to Register/Memory
                op.operation_type = OperationType::Mov;
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
                op.w = instruction & 1;
                if op.w == 1 {
                    op.data =
                        u16::from_le_bytes([self.next_byte(&mut op), self.next_byte(&mut op)]);
                } else {
                    op.data = self.next_byte(&mut op) as u16;
                }
                self.dump.simple_calc2(&op);
            }
            0b1011_0000..=0b1011_1111 => {
                // Immediate to Register
                op.operation_type = OperationType::Mov;
                op.w = instruction >> 3 & 1;
                op.reg = instruction & 0b111;
                if op.w == 1 {
                    op.data =
                        u16::from_le_bytes([self.next_byte(&mut op), self.next_byte(&mut op)]);
                } else {
                    op.data = self.next_byte(&mut op) as u16;
                }
                self.dump.simple_calc3(&op);
            }
            0b1010_0000..=0b1010_0011 => {
                // Memory to Accumulator
                // Accumulator to Memory
                op.operation_type = OperationType::Mov;
                op.w = instruction & 1;
                op.rm = 0b110;
                op.disp = Some(u16::from_le_bytes([
                    self.next_byte(&mut op),
                    self.next_byte(&mut op),
                ]));
                match (instruction >> 1) & 1 {
                    0 => self.dump.mov4(&op),
                    1 => self.dump.mov5(&op),
                    _ => panic!("Invalid operation"),
                }
            }
            0b1000_1110 | 0b1000_1100 => {
                // Register to Segment Register
                // Segment Register to Register
                op.operation_type = OperationType::Mov;
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
                if op.reg & 0b100 != 0 {
                    panic!("Invalid operation. reg must be 0b0xx in this operation");
                }
            }
            // Push
            0b0101_0000..=0b0101_0111 => {
                // Register
                op.operation_type = OperationType::Push;
                op.reg = instruction & 0b111;
                self.dump.stack2(&op);
            }
            0b0000_0110 | 0b0000_1110 | 0b0001_0110 | 0b0001_1110 => {
                // Segment Register
                op.operation_type = OperationType::Push;
                op.reg = instruction >> 3 & 0b111;
                self.dump.stack3(&op);
            }
            // Pop
            0b1000_1111 => {
                // Register/Memory
                op.operation_type = OperationType::Pop;
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
                if op.reg != 0b110 {
                    panic!("Invalid operation. reg must be 0b110 in this operation");
                }
                self.dump.stack1(&op);
            }
            0b0101_1000..=0b0101_1111 => {
                // Register
                op.operation_type = OperationType::Pop;
                op.reg = instruction & 0b111;
                self.dump.stack2(&op);
            }
            0b0000_0111 | 0b0000_1111 | 0b0001_0111 | 0b0001_1111 => {
                // Segment Register
                op.operation_type = OperationType::Pop;
                op.reg = instruction >> 3 & 0b111;
                self.dump.stack3(&op);
            }
            // Xchg
            0b1000_0110 | 0b1000_0111 => {
                // Register/Memory with Register
                op.operation_type = OperationType::Xchg;
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
                op.w = instruction & 1;
                self.dump.xchg1(&op);
            }
            0b1001_0000..=0b1001_0111 => {
                // Register with Accumulator
                op.operation_type = OperationType::Xchg;
                op.reg = instruction & 0b111;
                self.dump.xchg2(&op);
            }
            // In
            0b1110_0100 | 0b1110_0101 => {
                // Fixed Port
                op.operation_type = OperationType::In;
                op.w = instruction & 1;
                op.port = self.next_byte(&mut op);
                self.dump.in1(&op);
            }
            0b1110_1100 | 0b1110_1101 => {
                // Variable Port
                op.operation_type = OperationType::In;
                op.w = instruction & 1;
                self.dump.in2(&op);
            }
            // Out
            0b1110_0110 | 0b1110_0111 => {
                // Fixed Port
                op.operation_type = OperationType::Out;
                op.w = instruction & 1;
                op.port = self.next_byte(&mut op);
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
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
                self.dump.lea(&op);
            }
            // Lds
            0b1100_0101 => {
                op.operation_type = OperationType::Lds;
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
            }
            // Les
            0b1100_0100 => {
                op.operation_type = OperationType::Les;
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
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
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
                op.d = (instruction >> 1) & 1;
                op.w = instruction & 1;
                self.dump.simple_calc1(&op);
            }
            0b0000_0100 | 0b0000_0101 => {
                // Immediate to Accumulator
                op.operation_type = OperationType::Add;
                op.w = instruction & 1;
                if op.w == 1 {
                    op.data =
                        u16::from_le_bytes([self.next_byte(&mut op), self.next_byte(&mut op)]);
                } else {
                    op.data = self.next_byte(&mut op) as u16;
                }
                self.dump.simple_calc3(&op);
            }
            // Adc
            0b0001_0000..=0b0001_0011 => {
                // Register/Memory with Register to Either
                op.operation_type = OperationType::Adc;
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
                op.d = (instruction >> 1) & 1;
                op.w = instruction & 1;
                self.dump.simple_calc1(&op);
            }
            0b0001_0100 | 0b0001_0101 => {
                // Immediate to Accumulator
                op.operation_type = OperationType::Adc;
                op.w = instruction & 1;
                op.data = if op.w == 1 {
                    u16::from_le_bytes([self.next_byte(&mut op), self.next_byte(&mut op)])
                } else {
                    self.next_byte(&mut op) as u16
                };
                self.dump.simple_calc3(&op);
            }
            // Inc
            0b1111_1110 => {
                // Register/Memory
                op.operation_type = OperationType::Inc;
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
                op.w = instruction & 1;
                self.dump.inc_dec1(&op);
            }
            0b0100_0000..=0b0100_0111 => {
                // Register
                op.operation_type = OperationType::Inc;
                op.reg = instruction & 0b111;
                self.dump.inc_dec2(&op);
            }
            // Aaa
            0b0011_0111 => {
                op.operation_type = OperationType::Aaa;
            }
            // Baa
            0b0010_0111 => {
                op.operation_type = OperationType::Baa;
            }
            // Sub
            0b0010_1000..=0b0010_1011 => {
                // Register/Memory with Register to Either
                op.operation_type = OperationType::Sub;
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
                op.d = (instruction >> 1) & 1;
                op.w = instruction & 1;
                self.dump.simple_calc1(&op);
            }
            0b0010_1100 | 0b0010_1101 => {
                // Immediate from Accumulator
                op.operation_type = OperationType::Sub;
                op.w = instruction & 1;
                op.data = if op.w == 1 {
                    u16::from_le_bytes([self.next_byte(&mut op), self.next_byte(&mut op)])
                } else {
                    self.next_byte(&mut op) as u16
                };
                self.dump.simple_calc3(&op);
            }
            // Ssb
            0b0001_1000..=0b0001_1011 => {
                // Register/Memory with Register to Either
                op.operation_type = OperationType::Sbb;
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
                op.d = (instruction >> 1) & 1;
                op.w = instruction & 1;
                self.dump.simple_calc1(&op);
            }
            0b0001_1100 | 0b0001_1101 => {
                // データシートの誤植
                // https://qiita.com/7shi/items/b3911948f9d97b05395e#%E4%BB%95%E6%A7%98%E6%9B%B8
                // Immediate from Accumulator
                op.operation_type = OperationType::Sbb;
                op.w = instruction & 1;
                op.data = if op.w == 1 {
                    u16::from_le_bytes([self.next_byte(&mut op), self.next_byte(&mut op)])
                } else {
                    self.next_byte(&mut op) as u16
                };
            }
            // Dec
            0b0100_1000..=0b0100_1111 => {
                // Register
                op.operation_type = OperationType::Dec;
                op.reg = instruction & 0b111;
                self.dump.inc_dec2(&op);
            }
            // Cmp
            0b0011_1000..=0b0011_1011 => {
                // Register/Memory and Register
                op.operation_type = OperationType::Cmp;
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
                op.d = (instruction >> 1) & 1;
                op.w = instruction & 1;
                self.dump.simple_calc1(&op);
            }
            0b0011_1100 | 0b0011_1101 => {
                // Immediate with Accumulator
                op.operation_type = OperationType::Cmp;
                op.w = instruction & 1;
                op.data = if op.w == 1 {
                    u16::from_le_bytes([self.next_byte(&mut op), self.next_byte(&mut op)])
                } else {
                    self.next_byte(&mut op) as u16
                };
                self.dump.simple_calc3(&op);
            }
            // Aas
            0b0011_1111 => {
                op.operation_type = OperationType::Aas;
                self.dump.name(&op);
            }
            // Das
            0b0010_1111 => {
                op.operation_type = OperationType::Das;
                self.dump.name(&op);
            }
            // Cbw
            0b1001_1000 => {
                op.operation_type = OperationType::Cbw;
                self.dump.name(&op);
            }
            // Cwd
            0b1001_1001 => {
                op.operation_type = OperationType::Cwd;
                self.dump.name(&op);
            }

            // --- Logic ---
            // And
            0b0010_0000..=0b0010_0011 => {
                // Register/Memory and Register to Either
                op.operation_type = OperationType::And;
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
                op.d = (instruction >> 1) & 1;
                op.w = instruction & 1;
                self.dump.bit_op1(&op);
            }
            0b0010_0100 | 0b0010_0101 => {
                // Immediate with Accumulator
                op.operation_type = OperationType::And;
                op.w = instruction & 1;
                op.data = if op.w == 1 {
                    u16::from_le_bytes([self.next_byte(&mut op), self.next_byte(&mut op)])
                } else {
                    self.next_byte(&mut op) as u16
                };
                self.dump.bit_op3(&op);
            }
            // Test
            0b1000_0100 | 0b1000_0101 => {
                // Register/Memory and Register
                op.operation_type = OperationType::Test;
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
                self.dump.bit_op1(&op);
            }
            0b1010_1000 | 0b1010_1001 => {
                // Immediate Data and Accumulator
                op.operation_type = OperationType::Test;
                op.w = instruction & 1;
                op.data = if op.w == 1 {
                    u16::from_le_bytes([self.next_byte(&mut op), self.next_byte(&mut op)])
                } else {
                    self.next_byte(&mut op) as u16
                };
                self.dump.bit_op3(&op);
            }
            // Or
            0b0000_1000..=0b0000_1011 => {
                // Register/Memory and Register to Either
                op.operation_type = OperationType::Or;
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
                op.d = (instruction >> 1) & 1;
                op.w = instruction & 1;
                self.dump.bit_op1(&op);
            }
            0b0000_1100 | 0b0000_1101 => {
                // Immediate with Accumulator
                op.operation_type = OperationType::Or;
                op.w = instruction & 1;
                op.data = if op.w == 1 {
                    u16::from_le_bytes([self.next_byte(&mut op), self.next_byte(&mut op)])
                } else {
                    self.next_byte(&mut op) as u16
                };
                self.dump.bit_op3(&op);
            }
            // Xor
            0b0011_0000..=0b0011_0011 => {
                // Register/Memory and Register to Either
                op.operation_type = OperationType::Xor;
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
                op.d = (instruction >> 1) & 1;
                op.w = instruction & 1;
                self.dump.bit_op1(&op);
            }
            0b0011_0100 | 0b0011_0101 => {
                // Immediate with Accumulator
                op.operation_type = OperationType::Xor;
                op.w = instruction & 1;
                op.data = if op.w == 1 {
                    u16::from_le_bytes([self.next_byte(&mut op), self.next_byte(&mut op)])
                } else {
                    self.next_byte(&mut op) as u16
                };
                self.dump.bit_op3(&op);
            }

            // --- String Manipulation ---
            // Rep
            0b1111_0010 | 0b1111_0011 => {
                op.operation_type = OperationType::Rep;
                op.z = instruction & 1;
                let next_op = self.next_byte(&mut op);
                op.rep_operation_type = match next_op >> 1 & 0b111 {
                    0b010 => OperationType::Movs,
                    0b011 => OperationType::Cmps,
                    0b111 => OperationType::Scas,
                    0b110 => OperationType::Lods,
                    0b101 => OperationType::Stos,
                    _ => {
                        panic!("Invalid operation. {next_op:04x}");
                    }
                };
                op.w = next_op & 1;
                self.dump.rep(&op);
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
                self.dump.name(&op);
            }

            // --- Control Transfer ---
            // Call
            0b1110_1000 => {
                // Direct within Segment
                op.operation_type = OperationType::Call;
                op.disp = Some(u16::from_le_bytes([
                    self.next_byte(&mut op),
                    self.next_byte(&mut op),
                ]));
                self.dump.call1(&op);
            }
            0b1001_1010 => {
                // Direct Intersegment
                op.operation_type = OperationType::Call;
                // offset-low offset-high
                // seg-low seg-high
                op.disp = Some(u16::from_le_bytes([
                    self.next_byte(&mut op),
                    self.next_byte(&mut op),
                ]));
            }
            // Jmp
            0b1110_1001 => {
                // Direct within Segment
                op.operation_type = OperationType::Jmp;
                op.disp = Some(u16::from_le_bytes([
                    self.next_byte(&mut op),
                    self.next_byte(&mut op),
                ]));
                self.dump.jmp1(&op);
            }
            0b1110_1011 => {
                // Direct within Segment-Short
                op.operation_type = OperationType::Jmp;
                op.disp = Some(self.next_byte(&mut op) as u16);
                self.dump.jmp2(&op);
            }
            0b1110_1010 => {
                op.operation_type = OperationType::Jmp;
                op.disp = Some(u16::from_le_bytes([
                    self.next_byte(&mut op),
                    self.next_byte(&mut op),
                ]));
            }
            // Ret
            0b1100_0011 | 0b1100_1011 => {
                // Within Segment
                // Intersegment
                op.operation_type = OperationType::Ret;
                self.dump.name(&op);
            }
            0b1100_0010 | 0b1100_1010 => {
                // Within Segment Adding Immed to Sp
                // Within Segment Adding Immediate to Sp
                op.operation_type = OperationType::Ret;
                op.disp = Some(u16::from_le_bytes([
                    self.next_byte(&mut op),
                    self.next_byte(&mut op),
                ]));
                if instruction & 0b1000 == 1 {
                    panic!("Not implemented yet");
                } else {
                    self.dump.ret2(&op);
                }
            }
            // Jump
            0b0111_0000..=0b0111_1111 => {
                op.operation_type = match instruction & 0b1111 {
                    0b0000 => OperationType::Jo,
                    0b0001 => OperationType::Jno,
                    0b0010 => OperationType::JbJnae,
                    0b0011 => OperationType::JnbJae,
                    0b0100 => OperationType::JeJz,
                    0b0101 => OperationType::JneJnz,
                    0b0110 => OperationType::JbeJna,
                    0b0111 => OperationType::JnbeJa,
                    0b1000 => OperationType::Js,
                    0b1001 => OperationType::Jns,
                    0b1010 => OperationType::JpJpe,
                    0b1011 => OperationType::JnpJpo,
                    0b1100 => OperationType::JlJnge,
                    0b1101 => OperationType::JnlJge,
                    0b1110 => OperationType::JleJng,
                    0b1111 => OperationType::JnleJg,
                    _ => {
                        panic!("This code is not reachable");
                    }
                };
                op.disp = Some(self.next_byte(&mut op) as u16);
                self.dump.jump(&op);
            }
            // Loop
            0b1110_0000..=0b1110_0010 => {
                op.disp = Some(self.next_byte(&mut op) as u16);
                match instruction & 0b11 {
                    0b10 => {
                        op.operation_type = OperationType::Loop;
                        self.dump.loop1(&op);
                    }
                    0b01 => op.operation_type = OperationType::LoopzLoope,
                    0b00 => op.operation_type = OperationType::LoopnzLoopne,
                    _ => {
                        panic!("This code is not reachable");
                    }
                };
            }
            // Jump
            0b1110_0011 => {
                // Jump on CX Zero
                op.operation_type = OperationType::Jcxz;
                op.disp = Some(self.next_byte(&mut op) as u16);
            }
            // Int
            0b1100_1101 => {
                // Type Specified
                op.operation_type = OperationType::Int;
                op.int_type = self.next_byte(&mut op);
                self.dump.int1(&op);
            }
            0b11001100 => {
                // Type 3
                op.operation_type = OperationType::Int;
                self.dump.int2(&op);
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
                self.dump.name(&op);
            }
            0b1111_0101 => {
                op.operation_type = OperationType::Cmc;
                self.dump.name(&op);
            }
            0b1111_1001 => {
                op.operation_type = OperationType::Stc;
                self.dump.name(&op);
            }
            0b1111_1100 => {
                op.operation_type = OperationType::Cld;
                self.dump.name(&op);
            }
            0b1111_1101 => {
                op.operation_type = OperationType::Std;
                self.dump.name(&op);
            }
            0b1111_1010 => {
                op.operation_type = OperationType::Cli;
                self.dump.name(&op);
            }
            0b1111_1011 => {
                op.operation_type = OperationType::Sti;
                self.dump.name(&op);
            }
            0b1111_0100 => {
                op.operation_type = OperationType::Hlt;
                self.dump.name(&op);
            }
            0b1001_1011 => {
                op.operation_type = OperationType::Wait;
                self.dump.name(&op);
            }
            0b1101_1000..=0b1101_1111 => {
                op.operation_type = OperationType::Esc;
                op.reg = instruction & 0b111;
            }
            0b1111_0000 => {
                op.operation_type = OperationType::Lock;
                self.dump.name(&op);
            }

            // --- Common ---
            // Add/Adc/Sub/Ssb/Cmp/And
            0b1000_0000..=0b1000_0011 => {
                // Immediate to Register/Memory
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
                op.s = (instruction >> 1) & 1;
                op.w = instruction & 1;
                op.data = if op.s == 0 && op.w == 1 {
                    u16::from_le_bytes([self.next_byte(&mut op), self.next_byte(&mut op)])
                } else {
                    self.next_byte(&mut op) as u16
                };
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
                self.dump.simple_calc2(&op)
            }
            // Push/Inc/Dec/Call
            0b1111_1111 => {
                // Register/Memory
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
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
                self.dump.stack1(&op);
            }
            // Neg/Mul/Imul/Div/Idiv/Not
            0b1111_0110 | 0b1111_0111 => {
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
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
                            u16::from_le_bytes([self.next_byte(&mut op), self.next_byte(&mut op)])
                        } else {
                            // 8-bit immediate
                            self.next_byte(&mut op) as u16
                        };
                        OperationType::Test
                    }
                    _ => {
                        panic!("Invalid operation. reg is invalid");
                    }
                };
                if op.operation_type == OperationType::Test {
                    self.dump.test2(&op);
                } else {
                    self.dump.complicate_calc(&op);
                }
            }
            // Aam
            0b1101_0100 => {
                let next = self.next_byte(&mut op);
                op.operation_type = match next {
                    0b0000_1010 => OperationType::Aam,
                    _ => {
                        panic!("Invalid operation");
                    }
                }
            }
            // Aad
            0b1101_0101 => {
                let next = self.next_byte(&mut op);
                op.operation_type = match next {
                    0b0000_1010 => OperationType::Aad,
                    _ => {
                        panic!("Invalid operation");
                    }
                }
            }
            // Shl/Sal/Shr/Sar/Rol/Ror/Rcl/Rcr
            0b1101_0000..=0b1101_0011 => {
                let mod_reg_rm = self.next_byte(&mut op);
                op.set_mod_reg_rm(mod_reg_rm);
                self.disp(&mut op);
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
                self.dump.shift_rotate(&op);
            }

            _ => {
                panic!("Unknown operation: {:02x}", instruction);
            }
        }
        op
    }

    pub fn disassemble(&mut self) {
        while self.text_pos < self.text.len() {
            self.next_operation();
            print!("\n");
            stdout().flush().unwrap();
        }
    }
}
