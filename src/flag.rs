use std::fmt::Display;

#[derive(Debug)]
pub struct Flag {
    pub carry: bool,
    pub overflow: bool,
    pub sign: bool,
    pub zero: bool,
}

impl Flag {
    pub fn new() -> Self {
        Flag {
            carry: false,
            overflow: false,
            sign: false,
            zero: false,
        }
    }

    pub fn set_cosz(&mut self, carry: bool, overflow: bool, sign: bool, zero: bool) {
        self.carry = carry;
        self.overflow = overflow;
        self.sign = sign;
        self.zero = zero;
    }
}

impl Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let c = if self.carry { 'C' } else { '-' };
        let o = if self.overflow { 'O' } else { '-' };
        let s = if self.sign { 'S' } else { '-' };
        let z = if self.zero { 'Z' } else { '-' };
        write!(f, "{}{}{}{}", c, o, s, z)
    }
}
