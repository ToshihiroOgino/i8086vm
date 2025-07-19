#[derive(PartialEq)]
pub enum AppMode {
    Disassemble,
    Execute,
}

pub struct ArgsConfig {
    pub mode: AppMode,
    pub target: String,
}

pub fn parse_args() -> Result<ArgsConfig, String> {
    let args: Vec<String> = std::env::args().collect();

    let mode = match args.get(1).map(|s| s.as_str()) {
        Some("-d") => AppMode::Disassemble,
        Some("-m") => AppMode::Execute,
        Some(_) | None => {
            return Err(
                "Invalid mode specified. Use -d for disassemble or -m for execute.".to_string(),
            )
        }
    };

    let target = match args.get(2) {
        Some(t) => t.clone(),
        None => return Err("No target specified.".to_string()),
    };

    Ok(ArgsConfig { mode, target })
}
