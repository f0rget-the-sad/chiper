#[allow(clippy::print_with_newline)]

mod chip8;
mod screen;

use std::env;
use std::io::{self, Error, ErrorKind};
use std::process;
use chip8::Chip8;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("usage: chiper <path to rom>");
        process::exit(1);
    }
    let screen = screen::sdl_init().map_err(|e| Error::new(ErrorKind::Other, e))?;
    let mut chip8 = Chip8::new(screen);
    chip8.load_rom(&args[1])?;
    chip8.dump_memory();
    if env::var("DEBUGGER").is_err() {
        chip8.emulate();
    } else {
        chip8.debugger()?;
    }
    Ok(())
}
