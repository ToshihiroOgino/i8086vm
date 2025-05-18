#[allow(dead_code)]
#[derive(Default)]
pub enum OperationType {
    #[default]
    None,
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
    //Arithmetic
    Aad,
    Cbw,
    Cwd,
    //Logic
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
    //String Manipulation
    Rep,
    Movs,
    Cmps,
    Scas,
    Lods,
    Stos,
    //Control Transfer
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

#[allow(dead_code)]
#[derive(Default)]
pub struct Operation {
    pub operation_type: OperationType,
    pub raws: Vec<u8>,
    pub opcode: u8,
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
    pub disp: u8,
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
