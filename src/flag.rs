use std::fmt::Display;

#[allow(unused)]
#[derive(Debug)]
pub struct Flag {
    pub carry: bool,
    pub overflow: bool,
    pub sign: bool,
    pub zero: bool,
    pub auxiliary: bool,
    pub parity: bool,
}

impl Flag {
    pub fn new() -> Self {
        Flag {
            carry: false,
            overflow: false,
            sign: false,
            zero: false,
            auxiliary: false,
            parity: false,
        }
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
