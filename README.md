# Disassembler and Emulator of i8086

CPU: i8086
OS: minix

## Setup

```bash
tar -xzvf setuptools.tar.gz
cd setuptools
make
sudo make setup
sudo make install
```

## Generate Binary for This App

```bash
# for m2cc command (m2cc is installed at setup section)
export PATH=/usr/local/core/bin:$PATH
# compile assembly code
m2cc -.o asam.s -o asam.out
# compile C code
m2cc main.c -o main.out
```

## Usage

- Disassemble the binary file `a.out`: `cargo run -- -d a.out`
- Run `a.out`: `cargo run -- a.out`
- Run `a.out` with some arguments: `cargo run -- a.out arg1 arg2`
- Run `a.out` with detail: `cargo run -- -m a.out`
