#[derive(PartialEq)]
pub enum AppMode {
    None,
    Disassemble,
    Execute,
}

pub struct ArgsConfig {
    pub mode: AppMode,
    pub target: String,
}

pub fn parse_args() -> ArgsConfig {
    let args: Vec<String> = std::env::args().collect();
    let mut mode = AppMode::None;
    let mut target = String::new();

    for arg in args.iter() {
        match arg.as_str() {
            "-d" => mode = AppMode::Disassemble,
            "-m" => mode = AppMode::Execute,
            _ => target = arg.clone(),
        }
    }

    ArgsConfig { mode, target }
}
