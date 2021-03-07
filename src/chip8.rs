use std::fs::File;
use std::io::{self, stdin, Read, Write};
use std::thread;
use std::time;

use crate::screen::Screen;

const MEMORY_START: usize = 0x200;
const MEMORY_SIZE: usize = 0x1000;
pub const SCREEN_WIDTH: u32 = 62;
pub const SCREEN_HEIGHT: u32 = 32;

const SCREEN_MEMORY_START: u32 = 0xf00;
//const SCREEN_MEMORY_END: u32 = 0xfff;

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

    // return as usize since it's used only as index for V[] registers
    fn x(&self) -> usize {
        Opcode::low_nib(self.0).into()
    }

    // return as usize since it's used only as index for V[] registers
    fn y(&self) -> usize {
        Opcode::high_nib(self.1).into()
    }

    fn n(&self) -> u8 {
        Opcode::low_nib(self.1)
    }

    fn nnn(&self) -> u16 {
        (Opcode::low_nib(self.0) as u16) << 8 | self.1 as u16
    }

    fn disassemble(&self, pc: usize) {
        debug!("{:04x}:\t{:02x} {:02x}\t", pc, self.0, self.1);
        match Opcode::high_nib(self.0) {
            0x00 => match self.1 {
                0xe0 => {
                    debug!("dclr");
                }
                0xee => {
                    debug!("ret");
                }
                _ => {
                    debug!("UNKNOWN");
                }
            },
            0x01 => {
                // Jumps to address NNN.
                debug!("jmp\t\t{:03x}", self.nnn());
            }
            0x03 => {
                // Skips the next instruction if VX equals NN.
                // Usually the next instruction is a jump to skip a code block
                debug!("skipifeq\t\tV{:01x}, {:02x}", self.x(), self.1);
            }
            0x04 => {
                // Skips the next instruction if VX doesn't equal NN. (Usually the next instruction
                // is a jump to skip a code block)
                debug!("skipifne\t\tV{:01x}, {:02x}", self.x(), self.1);
            }
            0x05 => {
                // Skips the next instruction if VX equals VY.
                // Usually the next instruction is a jump to skip a code block
                debug!("skipifeq\t\tV{:01x}, V{:01x}", self.x(), self.y());
            }
            0x06 => {
                // Sets VX to NN
                debug!("mov\t\tV{:01x}, {:02x}", self.x(), self.1);
            }
            0x07 => {
                // Adds NN to VX. (Carry flag is not changed)
                debug!("add\t\tV{:01x}, {:02x}", self.x(), self.1);
            }
            0x0a => {
                //Sets I to the address NNN

                debug!("mov\t\tI, {:03x}", self.nnn());
            }
            0x0d => {
                // draw(Vx,Vy,N)
                debug!(
                    "draw\t\tV{:01x}, V{:01x}, {:01x}",
                    self.x(),
                    self.y(),
                    self.n()
                );
            }
            0x0f => match self.1 {
                0x1e => {
                    debug!("add\t\tI, V{:01x}", self.x());
                }
                _ => {
                    debug!("Opcode is not handled yet");
                }
            },
            _ => {
                debug!("Opcode is not handled yet");
            }
        }
        debug!("\n");
    }
}

pub struct Chip8<T> {
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

    screen: T,
}

impl<T: Screen> Chip8<T> {
    pub fn new(screen: T) -> Chip8<T> {
        Chip8 {
            v: [0; 16],
            i: 0,
            // TODO: why here not at 0xEA0?
            sp: 0xfa0,
            pc: MEMORY_START,
            memory: [0; MEMORY_SIZE],
            used_memory: 0,
            screen: screen,
        }
    }

    pub fn load_rom(&mut self, rom_path: &str) -> io::Result<()> {
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
    pub fn dump_registers(&self) {
        print!("REGISTERS:\n");
        print!("V = [");
        for i in 0..16 {
            print!("{:01x}:{:02x} ", i, self.v[i]);
        }
        print!("]\n");

        print!("I  = {:02x?}\n", self.i);
        print!("SP = {:02x?}\n", self.sp);
        print!("PC = {:02x?}\n", self.pc);
    }

    pub fn dump_memory(&self) {
        let mut pc = MEMORY_START;
        for two_bytes in self.memory[MEMORY_START..MEMORY_START + self.used_memory].chunks(2) {
            let opcode = Opcode(two_bytes[0], two_bytes[1]);
            opcode.disassemble(pc);
            pc += 2;
        }
    }

    fn inc_pc(&mut self) {
        self.pc += 2;
    }

    fn emulate_op(&mut self) {
        let opcode = Opcode(self.memory[self.pc], self.memory[self.pc + 1]);
        //opcode.disassemble(self.pc);

        match Opcode::high_nib(opcode.0) {
            0x00 => match opcode.1 {
                0xe0 => self.op_disp_clear(),
                //0xee => {
                //    debug!("return;");
                //}
                _ => unimplemented!(),
            },
            0x01 => {
                // Jumps to address NNN.
                let target = opcode.nnn();
                // TODO: Infinite loop
                //assert!(target as usize != self.pc);
                if target as usize == self.pc {
                    loop {
                        thread::sleep(time::Duration::from_millis(10));
                    }
                }
                self.pc = target.into();
            }
            0x03 => {
                // Skips the next instruction if VX equals NN.
                // Usually the next instruction is a jump to skip a code block
                if self.v[opcode.x()] == opcode.1 {
                    self.inc_pc();
                }
            }
            0x04 => {
                // Skips the next instruction if VX doesn't equal NN.
                // Usually the next instruction is a jump to skip a code block
                if self.v[opcode.x()] != opcode.1 {
                    self.inc_pc();
                }
            }
            0x05 => {
                // Skips the next instruction if VX equals VY
                // Usually the next instruction is a jump to skip a code block
                if self.v[opcode.x()] == self.v[opcode.y()] {
                    self.inc_pc();
                }
            }
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
            0x0d => {
                self.op_draw(
                    self.v[opcode.x()].into(),
                    self.v[opcode.y()].into(),
                    opcode.n(),
                );
            }
            0x0f => match opcode.1 {
                0x1e => {
                    // Adds VX to I. VF is not affected
                    self.i = self.i.wrapping_add(self.v[opcode.x()].into());
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }

        match Opcode::high_nib(opcode.0) {
            // one of the JUMP instruction, this will change the PC by itself,
            // not need to increment it
            0x01 => {}
            // regular opcode, move forward to the next one
            _ => self.inc_pc(),
        }
    }

    /// Clears the screen
    fn op_disp_clear(&mut self) {
        // TODO: think should we use sdl2 or webasm, or both
        // Ideally would be to provide trait:Display(Renderer) and anyone who implements
        // it can be passed to chip8 to be use as graphical interface
        self.screen.clear();
        self.screen.present();
    }

    /// Draw the sprite
    fn op_draw(&mut self, x: usize, y: usize, len: u8) {
        // Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height
        // of N+1 pixels. Each row of 8 pixels is read as bit-coded starting from memory
        // location I; I value doesn’t change after the execution of this instruction. As
        // described above, VF is set to 1 if any screen pixels are flipped from set to
        // unset when the sprite is drawn, and to 0 if that doesn’t happen
        let mut cy;
        for i in 0..len {
            cy = y + i as usize;
            if cy >= SCREEN_HEIGHT as usize {
                // sprite goes out of screen, stop drawing
                break;
            }

            let sprite_line = self.memory[(self.i + i as u16) as usize];
            let mut cx = x;
            for bi in (0..8).rev() {
                let mut px = ((sprite_line & (1 << bi)) != 0) as u8;

                if px != 0 {
                    if cx >= SCREEN_WIDTH as usize {
                        // sprite goes out of screen, stop drawing line
                        break;
                    }
                    // Determine the address of the effected byte on the screen
                    let screen_line_idx = SCREEN_MEMORY_START as usize + cy * 8 + cx / 8;
                    let screen_line = self.memory[screen_line_idx];
                    // Determine the effected bit in the byte
                    let screen_px = screen_line & (1 << (cx % 8));
                    if screen_px != 0 {
                        self.v[0xf] = 1;
                    }

                    // Write the effected bit to the screen memory
                    self.memory[screen_line_idx] ^= px;

                    // draw px
                    px ^= screen_px;
                    if px == 0 {
                        self.screen.clear_px(cx as i32, cy as i32);
                    } else {
                        self.screen.draw_px(cx as i32, cy as i32);
                    }
                }
                cx += 1;
            }
        }
        self.screen.present();
    }

    pub fn emulate(&mut self) {
        loop {
            self.emulate_op();
        }
    }

    pub fn debugger(&mut self) -> io::Result<()> {
        print!("Enter debug mode:\n");
        print!("\t'r' - to run program\n");
        print!("\t'n' - for next instruction\n");
        print!("\t'q' - to exit\n");
        let mut buffer = String::new();
        let mut last_cmd = String::new();
        loop {
            print!("(chiper - db) ");
            io::stdout().flush().ok().expect("Could not flush stdout");
            stdin().read_line(&mut buffer)?;
            let mut cmd = buffer.trim_end();
            if cmd.is_empty() {
                cmd = &last_cmd;
            } else {
                last_cmd = cmd.to_string();
            }
            match cmd {
                "n" => {
                    self.emulate_op();
                    self.dump_registers();
                }
                "r" => loop {
                    self.emulate_op();
                    self.dump_registers();
                },
                "q" => {
                    break;
                }
                unknown => {
                    eprint!("Unknown debug command '{}'\n", unknown);
                }
            }
            buffer.clear();
        }
        Ok(())
    }
}
