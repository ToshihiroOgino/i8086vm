pub struct Machine {
    register: [u32; 16],
    program_counter: usize,
    halted: bool,
    stack: Vec<u32>,
    heap: Vec<u32>,
}

impl Machine {}
