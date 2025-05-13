mod args;
mod disassembler;
mod metadata;
mod register;
mod operation;

fn main() {
    let config = args::parse_args();

    match config.mode {
        args::AppMode::Disassemble => {
            // disassembler::disassemble(&config.target);
            let mut disassembler = disassembler::Disassembler::new(&config.target);
            disassembler.disassemble();
        }
        args::AppMode::ExecuteWithLogs => {
            println!("Not implemented yet");
        }
        args::AppMode::None => {
            println!("No mode specified. Use -d for disassemble or -m for execute with logs.");
            std::process::exit(1);
        }
    }
}
