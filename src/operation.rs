use std::fmt::Display;

#[derive(Default, PartialEq)]
pub enum OperationType {
    #[default]
    Undefined,
    // Data Transfer
    Mov,
    Push,
    Pop,
    Xchg,
    In,
    Out,
    Xlat,
    Lea,
    Lds,
    Les,
    Lahf,
    Sahf,
    Pushf,
    Popf,
    Add,
    Adc,
    Inc,
    Aaa,
    Baa,
    Sub,
    Sbb,
    Dec,
    Neg,
    Cmp,
    Aas,
    Das,
    Mul,
    Imul,
    Aam,
    Div,
    Idiv,
    // Arithmetic
    Aad,
    Cbw,
    Cwd,
    // Logic
    Not,
    ShlSal,
    Shr,
    Sar,
    Rol,
    Ror,
    Rcl,
    Rcr,
    And,
    Test,
    Or,
    Xor,
    // String Manipulation
    Rep,
    Movs,
    Cmps,
    Scas,
    Lods,
    Stos,
    // Control Transfer
    Call,
    Jmp,
    Ret,
    Jump,
    Loop,
    Int,
    Into,
    Iret,
    // Processor Control
    Clc,
    Cmc,
    Stc,
    Cld,
    Std,
    Cli,
    Sti,
    Hlt,
    Wait,
    Esc,
    Lock,
}

impl Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            OperationType::Undefined => "(undefined)",
            OperationType::Mov => "MOV",
            OperationType::Push => "PUSH",
            OperationType::Pop => "POP",
            OperationType::Xchg => "XCHG",
            OperationType::In => "IN",
            OperationType::Out => "OUT",
            OperationType::Xlat => "XLAT",
            OperationType::Lea => "LEA",
            OperationType::Lds => "LDS",
            OperationType::Les => "LES",
            OperationType::Lahf => "LAHF",
            OperationType::Sahf => "SAHF",
            OperationType::Pushf => "PUSHF",
            OperationType::Popf => "POPF",
            OperationType::Add => "ADD",
            OperationType::Adc => "ADC",
            OperationType::Inc => "INC",
            OperationType::Aaa => "AAA",
            OperationType::Baa => "BAA",
            OperationType::Sub => "SUB",
            OperationType::Sbb => "SBB",
            OperationType::Dec => "DEC",
            OperationType::Neg => "NEG",
            OperationType::Cmp => "CMP",
            OperationType::Aas => "AAS",
            OperationType::Das => "DAS",
            OperationType::Mul => "MUL",
            OperationType::Imul => "IMUL",
            OperationType::Aam => "AAM",
            OperationType::Div => "DIV",
            OperationType::Idiv => "IDIV",
            OperationType::Aad => "AAD",
            OperationType::Cbw => "CBW",
            OperationType::Cwd => "CWD",
            OperationType::Not => "NOT",
            OperationType::ShlSal => "SHL/SAL",
            OperationType::Shr => "SHR",
            OperationType::Sar => "SAR",
            OperationType::Rol => "ROL",
            OperationType::Ror => "ROR",
            OperationType::Rcl => "RCL",
            OperationType::Rcr => "RCR",
            OperationType::And => "AND",
            OperationType::Test => "TEST",
            OperationType::Or => "OR",
            OperationType::Xor => "XOR",
            OperationType::Rep => "REP",
            OperationType::Movs => "MOVS",
            OperationType::Cmps => "CMPS",
            OperationType::Scas => "SCAS",
            OperationType::Lods => "LODS",
            OperationType::Stos => "STOS",
            OperationType::Call => "CALL",
            OperationType::Jmp => "JMP",
            OperationType::Ret => "RET",
            OperationType::Jump => "JUMP",
            OperationType::Loop => "LOOP",
            OperationType::Int => "INT",
            OperationType::Into => "INTO",
            OperationType::Iret => "IRET",
            OperationType::Clc => "CLC",
            OperationType::Cmc => "CMC",
            OperationType::Stc => "STC",
            OperationType::Cld => "CLD",
            OperationType::Std => "STD",
            OperationType::Cli => "CLI",
            OperationType::Sti => "STI",
            OperationType::Hlt => "HLT",
            OperationType::Wait => "WAIT",
            OperationType::Esc => "ESC",
            OperationType::Lock => "LOCK",
        };
        write!(f, "{}", name)
    }
}

#[derive(Default)]
pub struct Operation {
    pub pos: usize,
    pub operation_type: OperationType,
    pub raws: Vec<u8>,
    pub d: u8,
    pub w: u8,
    pub s: u8,
    pub z: u8,
    pub v: u8,
    pub mod_rm: u8,
    pub rm: u8,
    pub reg: u8,
    pub data: u16,
    pub low: u8,
    pub high: u8,
    pub port: u8,
    pub disp: Option<u16>,
    pub int_type: u8,
}

impl Operation {
    pub fn new() -> Self {
        Operation::default()
    }

    pub fn set_mod_reg_rm(&mut self, mod_reg_rm: u8) {
        self.mod_rm = (mod_reg_rm >> 6) & 0b11;
        self.reg = (mod_reg_rm >> 3) & 0b111;
        self.rm = mod_reg_rm & 0b111;
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut raws = String::new();
        for byte in &self.raws {
            raws.push_str(&format!("{:02x}", byte));
        }
        write!(f, "{:04}: {raws}\t{}", self.pos, self.operation_type)
    }
}
