use std::io::Read;

mod args;
mod disassembler;
mod dump;
mod flag;
mod machine;
mod message;
mod metadata;
mod operation;
mod register;

fn main() {
    let config = match args::parse_args() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error parsing arguments: {}", e);
            std::process::exit(1);
        }
    };

    let mut stream = match std::fs::File::open(&config.target) {
        Ok(file) => std::io::BufReader::new(file),
        Err(e) => {
            eprintln!("Failed to open target file: {}", e);
            std::process::exit(1);
        }
    };

    let mut executable = Vec::new();
    stream
        .read_to_end(&mut executable)
        .expect("Failed to read executable file");

    match config.mode {
        args::AppMode::Disassemble => {
            disassembler::disassemble(&executable, true);
        }
        args::AppMode::Execute => {
            let mut machine =
                machine::Machine::new(&executable, &config.argv, &config.envs, config.debug);
            machine.run();
        }
    }
}
