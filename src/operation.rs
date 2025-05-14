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
    pub mod_rm: u8,
    pub rm: u8,
    pub reg: u8,
    pub data: u16,
    // Mov
    pub addr_low: u8,
    pub addr_high: u8,
    // In/Out
    pub port: u8,
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
