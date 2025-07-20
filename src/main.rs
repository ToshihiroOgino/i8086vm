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

    let metadata = match metadata::Metadata::load_from_stream(&mut stream) {
        Ok(meta) => meta,
        Err(e) => {
            eprintln!("Failed to load metadata: {}", e);
            std::process::exit(1);
        }
    };

    dbg!(&metadata);

    match config.mode {
        args::AppMode::Disassemble => {
            disassembler::disassemble(stream, &metadata, true);
        }
        args::AppMode::Execute => {
            let mut machine = machine::Machine::new(stream, metadata, &config.argv, &config.envs);
            machine.run();
        }
    }
}
