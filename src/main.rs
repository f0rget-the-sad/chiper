use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::process;

const MEMORY_START: usize = 0x200;
const MEMORY_SIZE: usize = 0x1000;

// TODO: use logger?
#[cfg(debug_assertions)]
macro_rules! debug {
    ($( $args:expr ),*) => { print!( $( $args ),* ); }
}

#[cfg(not(debug_assertions))]
macro_rules! debug {
    ($( $args:expr ),*) => {};
}

#[derive(Debug)]
struct Opcode(u8, u8);

impl Opcode {
    fn high_nib(byte: u8) -> u8 {
        byte >> 4
    }

    fn low_nib(byte: u8) -> u8 {
        byte & 0x0f
    }

    fn x(&self) -> usize {
        Opcode::low_nib(self.0).into()
    }

    fn y(&self) -> usize {
        Opcode::high_nib(self.1).into()
    }

    fn nnn(&self) -> u16 {
        (Opcode::low_nib(self.0) as u16) << 8 | self.1 as u16
    }

    fn disassemble(&self) {
        match Opcode::high_nib(self.0) {
            0x00 => match self.1 {
                0xe0 => {
                    debug!("disp_clear()");
                }
                0xee => {
                    debug!("return;");
                }
                _ => {
                    debug!("UNKNOWN");
                }
            },
            0x06 => {
                // Sets VX to NN
                debug!("V{} = {:02x}", self.x(), self.1);
            }
            0x07 => {
                // Adds NN to VX. (Carry flag is not changed)
                debug!("V{} += {:02x}", self.x(), self.1);
            }
            0x0a => {
                //Sets I to the address NNN
                debug!("I = {:03x}", self.nnn());
            }
            _ => {
                debug!("Opcode is not handled yet");
            }
        }
        debug!("\n");
    }
}

struct Chip8 {
    ///  16 8-bit data registers named V0 to VF
    v: [u8; 16],
    /// Memory address register
    i: u16,
    /// Stack pointer
    sp: u16,
    /// Program counter
    pc: usize,
    /*
       uint8_t     delay;
       uint8_t     sound;
       uint8_t     *screen;  //this is memory[0xF00];
    */
    /// RAM
    memory: [u8; MEMORY_SIZE],
    /// amount of memory occupied by rom
    used_memory: usize,
}

impl Chip8 {
    fn new() -> Chip8 {
        Chip8 {
            v: [0; 16],
            i: 0,
            // TODO: why here not at 0xEA0?
            sp: 0xfa0,
            pc: MEMORY_START,
            memory: [0; MEMORY_SIZE],
            used_memory: 0,
        }
    }

    fn load_rom(&mut self, rom_path: &str) -> io::Result<()> {
        let mut file = File::open(rom_path)?;
        let mut buffer = Vec::<u8>::new();

        // read the whole file into buffer
        file.read_to_end(&mut buffer)?;

        // CHIP-8 convention puts programs in memory at `MEMORY_START`
        // They will all have hardcoded addresses expecting that
        self.memory[MEMORY_START..MEMORY_START + buffer.len()].copy_from_slice(&buffer[..]);
        self.used_memory = buffer.len();
        Ok(())
    }

    /// Dump all Chip8 registers, but not memory
    fn dump_registers(&self) {
        print!("REGISTERS:\n");
        print!("\tV  = {:02x?}\n", self.v);
        print!("\tI  = {:02x?}\n", self.i);
        print!("\tSP = {:02x?}\n", self.sp);
        print!("\tPC = {:02x?}\n", self.pc);
    }

    fn dump_memory(&self) {
        let mut pc = MEMORY_START;
        for two_bytes in self.memory[MEMORY_START..MEMORY_START + self.used_memory].chunks(2) {
            let opcode = Opcode(two_bytes[0], two_bytes[1]);
            debug!("{:04x} {:02x} {:02x} ", pc, two_bytes[0], two_bytes[1]);
            opcode.disassemble();
            pc += 2;
        }
    }

    fn emulate_op(&mut self) {
        let opcode = Opcode(self.memory[self.pc], self.memory[self.pc + 1]);
        debug!("{:04x}: {:02x}{:02x}\t", self.pc, opcode.0, opcode.1);
        opcode.disassemble();

        // TODO: not sure it's good idea to do this before parsing opcode
        // may be overwritten by jumps?
        self.pc += 2;

        match Opcode::high_nib(opcode.0) {
            0x00 => match opcode.1 {
                0xe0 => self.op_disp_clear(),
                //0xee => {
                //    debug!("return;");
                //}
                _ => unimplemented!(),
            },
            0x06 => {
                //Sets VX to NN
                self.v[opcode.x()] = opcode.1;
            }
            0x07 => {
                // Adds NN to VX. (Carry flag is not changed)
                self.v[opcode.x()] = self.v[opcode.x()].wrapping_add(opcode.1);
            }
            0x0a => {
                //Sets I to the address NNN
                self.i = opcode.nnn();
            }
            _ => unimplemented!(),
        }
    }

    /// Clears the screen
    fn op_disp_clear(&mut self) {
        // TODO: think should we use sdl2 or webasm, or both
        // Ideally would be to provide trait:Display(Renderer) and anyone who implements
        // it can be passed to chip8 to be use as graphical interface
        //self.op_unimplemented();
    }

    /// skips unimplemented instructions
    fn op_unimplemented(&mut self) {}
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("usage: chiper <path to rom>");
        process::exit(1);
    }
    debug!("STARTINGG ======== with {}\n", args[1]);
    let mut chip8 = Chip8::new();
    chip8.load_rom(&args[1])?;
    //chip8.dump_memory();
    for _ in 0..10 {
        chip8.emulate_op();
        chip8.dump_registers();
    }
    Ok(())
}
