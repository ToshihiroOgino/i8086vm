# Disassembler and Emulator of i8086

CPU: i8086

OS: minix

## Setup

```bash
tar -xzvf setuptools.tar.gz
cd setuptools
make
sudo make install
sudo make setup
```

## Generate Binary for This App

```bash
# Set PATH for m2cc command (m2cc is installed at setup section)
export PATH=/usr/local/core/bin:$PATH
# Compile assembly code
m2cc -.o asem.s -o asem.out
# Compile C code
m2cc main.c -o main.out
```

## Usage

- Disassemble the binary file `a.out`: `cargo run -- -d a.out`
- Run `a.out`: `cargo run -- a.out`
- Run `a.out` with some arguments: `cargo run -- a.out arg1 arg2`
- Run `a.out` with detail: `cargo run -- -m a.out`

## Architecture

This project implements a complete i8086 CPU emulator and disassembler written in Rust. The architecture consists of several key components:

### Core Components

- **Machine (`machine.rs`)**: The main CPU emulator that simulates i8086 processor behavior, including instruction execution, memory management, and system calls.
- **Disassembler (`disassembler.rs`)**: Converts binary machine code back to human-readable assembly instructions for debugging and analysis.
- **Register (`register.rs`)**: Models the complete i8086 register set including general-purpose registers (AX, BX, CX, DX), index registers (SI, DI), stack pointers (SP, BP), segment registers (CS, DS, ES, SS), and the instruction pointer (IP).
- **Operation (`operation.rs`)**: Defines the instruction set architecture with support for data transfer, arithmetic, logical, string, and control flow operations.

### Supporting Modules

- **Args (`args.rs`)**: Command-line argument parsing with support for disassembly mode (`-d`) and execution mode (`-m`).
- **Metadata (`metadata.rs`)**: Handles executable file format parsing to extract header information, segment sizes, and entry points.
- **Flag (`flag.rs`)**: Implements CPU status flags (Zero, Carry, Sign, Overflow, etc.) for instruction execution.
- **Dump (`dump.rs`)**: Provides debugging output capabilities for memory and register state inspection.
- **Message (`message.rs`)**: System call interface for handling OS interactions like I/O operations.

### Execution Flow

1. **Binary Loading**: The emulator loads executable files and parses their metadata structure.
2. **Mode Selection**: Either disassembles the binary to assembly code or executes it in the virtual machine.
3. **Instruction Processing**: The machine fetches, decodes, and executes instructions while maintaining proper CPU state.
4. **System Integration**: Supports Minix system calls and environment variable handling for compatibility with the target OS.

The project targets Minix OS binaries compiled with the m2cc compiler and provides a faithful emulation of i8086 processor behavior for educational and debugging purposes.

## Comments and overall impressions

There are many types of instructions, so I ended up writing a very large switch-case statement. There might have been a better way to implement this.

Through this project, I gained a deeper understanding of process memory organization, CPU register architecture, and the structure of instruction sets, which was a valuable learning experience.
