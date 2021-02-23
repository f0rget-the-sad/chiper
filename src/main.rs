use std::env;
use std::fs::File;
use std::io::{self, Read};

const MEMORY_START: usize = 0x200;
const MEMORY_SIZE: usize = 0x1000;

#[derive(Debug)]
// bitfields will save some space, but rust doesn't natively support them
struct Opcode(u8, u8, u8, u8);

struct Chip8 {
    ///  16 8-bit data registers named V0 to VF
    v: [u8; 16],
    /// Memory address register
    i: u16,
    /// Stack pointer
    sp: u16,
    /// Program counter
    pc: u16,
    /*
       uint8_t     delay;
       uint8_t     sound;
       uint8_t     *screen;  //this is memory[0xF00];
    */
    memory: [u8; MEMORY_SIZE],
}

impl Chip8 {
    fn new() -> Chip8 {
        Chip8 {
            v: [0; 16],
            i: 0,
            // TODO: why here not at 0xEA0?
            sp: 0xfa0,
            pc: MEMORY_START as u16,
            memory: [0; MEMORY_SIZE],
        }
    }

    fn load_rom(rom_path: &str) -> io::Result<()> {
        let mut file = File::open(rom_path)?;
        let mut buffer = Vec::<u8>::new();

        // read the whole file into buffer
        file.read_to_end(&mut buffer)?;

        chip8.memory[MEMORY_START..].copy_from_slice(&buffer[..]);
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("usage: chiper <path to rom>");
        panic!("bad usage");
    }

    let mut chip8 = Chip8::new();
    chip8.load_rom(&args[1])?;
    let mut file = File::open()?;
    let mut buffer = Vec::<u8>::new();

    // read the whole file into buffer
    file.read_to_end(&mut buffer)?;

    chip8.memory[MEMORY_START..].copy_from_slice(&buffer[..]);

    //println!("{:02x?}", buffer);
    //disassemble_all(buffer);
    Ok(())
}

fn disassemble_all(buffer: Vec<u8>) {
    let mut pc = MEMORY_START;
    for two_bytes in buffer.chunks(2) {
        let opcode = Opcode(
            two_bytes[0] >> 4,
            two_bytes[0] & 0x0f,
            two_bytes[1] >> 4,
            two_bytes[1] & 0x0f,
        );
        print!("{:04x} {:02x} {:02x} ", pc, two_bytes[0], two_bytes[1]);
        disassemble_opcode(opcode);
        pc += 2;
    }
}

fn disassemble_opcode(opcode: Opcode) {
    print!("{:01x?} ", opcode);
    match opcode.0 {
        0x00 => match opcode.2 << 4 | opcode.3 {
            0xe0 => print!("disp_clear()"),
            0xee => print!("return;"),
            _ => print!("UNKNOWN"),
        },
        0x06 => {
            //Sets VX to NN
            let nn = opcode.2 << 4 | opcode.3;
            print!("V{} = {:02x}", opcode.1, nn);
        }
        0x0a => {
            //Sets I to the address NNN
            let nnn: u16 = (opcode.1 as u16) << 8 | (opcode.2 << 4) as u16 | (opcode.3) as u16;
            print!("I = {:03x}", nnn);
        }
        _ => print!("Opcode is not handled yet"),
    }
    print!("\n");
}
