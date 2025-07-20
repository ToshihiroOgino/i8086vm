#[derive(PartialEq)]
pub enum AppMode {
    Disassemble,
    Execute,
}

pub struct ArgsConfig {
    pub mode: AppMode,
    pub target: String,
    pub argv: Vec<String>,
    pub envs: Vec<String>,
    pub debug: bool,
}

pub fn parse_args() -> Result<ArgsConfig, String> {
    let mut args: Vec<String> = std::env::args().collect();
    args.remove(0);

    let mut debug = true;

    let mode = match args.get(0).map(|s| s.as_str()) {
        Some("-d") => {
            args.remove(0);
            AppMode::Disassemble
        }
        Some("-m") => {
            args.remove(0);
            AppMode::Execute
        }
        _ => {
            debug = false;
            AppMode::Execute
        }
    };

    let target = match args.get(0) {
        Some(t) => t.clone(),
        None => return Err("No target specified.".to_string()),
    };

    let argv = args.iter().map(|s| s.to_string()).collect();
    let mut envs = Vec::new();
    envs.push("PATH=/usr:/usr/bin".to_string());

    Ok(ArgsConfig {
        mode,
        target,
        argv,
        envs,
        debug,
    })
}
