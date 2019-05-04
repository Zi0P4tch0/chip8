use std::fs::File;
use std::io::Read;
use rand;
use rand::Rng;

const CHIP8_RAM_SIZE: usize = 4096;
pub const CHIP8_GFX_WIDTH: usize = 64;
pub const CHIP8_GFX_HEIGHT: usize = 32;
const CHIP8_N_REGISTERS: usize = 16;
const CHIP8_N_KEYS: usize = 16;
const CHIP8_STACK_DEPTH: usize = 16;
const CHIP8_PROGRAM_START: usize = 0x200;

#[derive(Debug, PartialEq)]
enum ProgramCounter {
    Next,
    Skip,
    Jump(usize),
}

pub struct CPU {
    ram: [u8; CHIP8_RAM_SIZE],
    pub vram: [[u8; CHIP8_GFX_WIDTH]; CHIP8_GFX_HEIGHT],
    v: [u8; CHIP8_N_REGISTERS],
    i: usize,
    pc: usize,
    stack: [usize; CHIP8_STACK_DEPTH],
    sp: usize,
    delay_timer: u8,
    pub sound_timer: u8,
    keypad: [bool; CHIP8_N_KEYS],
    waiting_keypad: bool,
    waiting_keypad_register: usize,
    pub redraw: bool
}

impl CPU {

    pub fn new() -> CPU {
        let mut cpu = CPU {
            ram: [0; CHIP8_RAM_SIZE],
            vram: [[0; CHIP8_GFX_WIDTH]; CHIP8_GFX_HEIGHT],
            v: [0; CHIP8_N_REGISTERS],
            i: 0,
            pc: CHIP8_PROGRAM_START,
            stack: [0; CHIP8_STACK_DEPTH],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; CHIP8_N_KEYS],
            waiting_keypad: false,
            waiting_keypad_register: 0,
            redraw: false
        };
        for i in 0..FONT_SET.len() {
            cpu.ram[0x50+i] = FONT_SET[i]
        }
        cpu
    }

    pub fn load_game(&mut self, file: &mut File) {
        for byte in file.bytes() {
            match byte {
                Ok(b) => {
                    self.ram[self.pc] = b;
                    self.pc += 1;
                }
                Err(_e) => {

                }
            }
        }
        self.pc = 0x200;
    }

    pub fn tick(&mut self, keypad: [bool; 16]) {
        self.keypad = keypad;
        self.redraw = false;
        if !self.waiting_keypad {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
                self.sound_timer -= 1;
            }
            let opcode = self.get_opcode();
            self.exec_opcode(opcode);
        } else {
            for i in 0..self.keypad.len() {
                if self.keypad[i] {
                    self.v[self.waiting_keypad_register] = i as u8;
                    self.waiting_keypad = false;
                    self.waiting_keypad_register = 0x0;
                    break;
                }
            }
        }
    }

    fn get_opcode(&self) -> u16 {
        (self.ram[self.pc] as u16) << 8 | (self.ram[self.pc + 1] as u16)
    }

    fn exec_opcode(&mut self, opcode: u16) {

        let nibbles = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8,
        );

        let nnn = (opcode & 0x0FFF) as usize;
        let kk = (opcode & 0x00FF) as u8;
        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let n = nibbles.3 as usize;

        let pc: Option<ProgramCounter> = match nibbles {
            (0x0, 0x0, 0xE, 0x0) => Some(self.op_00e0()),
            (0x0, 0x0, 0xE, 0xE) => Some(self.op_00ee()),
            (0x1, _,   _,   _  ) => Some(self.op_1nnn(nnn)),
            (0x2, _,   _,   _  ) => Some(self.op_2nnn(nnn)),
            (0x3, _,   _,   _  ) => Some(self.op_3xkk(x, kk)),
            (0x4, _,   _,   _  ) => Some(self.op_4xkk(x, kk)),
            (0x5, _,   _,   0x0) => Some(self.op_5xy0(x ,y)),
            (0x6, _,   _,   _  ) => Some(self.op_6xkk(x, kk)),
            (0x7, _,   _,   _  ) => Some(self.op_7xkk(x, kk)),
            (0x8, _,   _,   0x0) => Some(self.op_8xy0(x, y)),
            (0x8, _,   _,   0x1) => Some(self.op_8xy1(x, y)),
            (0x8, _,   _,   0x2) => Some(self.op_8xy2(x, y)),
            (0x8, _,   _,   0x3) => Some(self.op_8xy3(x, y)),
            (0x8, _,   _,   0x4) => Some(self.op_8xy4(x, y)),
            (0x8, _,   _,   0x5) => Some(self.op_8xy5(x, y)),
            (0x8, _,   _,   0x6) => Some(self.op_8xy6(x)),
            (0x8, _,   _,   0x7) => Some(self.op_8xy7(x, y)),
            (0x8, _,   _,   0xE) => Some(self.op_8xye(x)),
            (0x9, _,   _,   0x0) => Some(self.op_9xy0(x, y)),
            (0xA, _,   _,   _  ) => Some(self.op_annn(nnn)),
            (0xB, _,   _,   _  ) => Some(self.op_bnnn(nnn)),
            (0xC, _,   _,   _  ) => Some(self.op_cxkk(x, kk)),
            (0xD, _,   _,   _  ) => Some(self.op_dxyn(x, y, n)),
            (0xE, _,   0x9, 0xE) => Some(self.op_ex9e(x)),
            (0xE, _,   0xA, 0x1) => Some(self.op_exa1(x)),
            (0xF, _,   0x0, 0x7) => Some(self.op_fx07(x)),
            (0xF, _,   0x0, 0xA) => Some(self.op_fx0a(x)),
            (0xF, _,   0x1, 0x5) => Some(self.op_fx15(x)),
            (0xF, _,   0x1, 0x8) => Some(self.op_fx18(x)),
            (0xF, _,   0x1, 0xE) => Some(self.op_fx1e(x)),
            (0xF, _,   0x2, 0x9) => Some(self.op_fx29(x)),
            (0xF, _,   0x3, 0x3) => Some(self.op_fx33(x)),
            (0xF, _,   0x5, 0x5) => Some(self.op_fx55(x)),
            (0xF, _,   0x6, 0x5) => Some(self.op_fx65(x)),
            _ => None
        };

        match pc {
            Some(value) => {
                match value {
                    ProgramCounter::Next => self.pc += 2,
                    ProgramCounter::Skip => self.pc += 4,
                    ProgramCounter::Jump(address) =>  self.pc = address ,
                }
            }
            None => {
                eprintln!("Unknown opcode: {:#X?}.", opcode);
            }
        }
    }

    // 00E0 - CLS
    fn op_00e0(&mut self) -> ProgramCounter {
        for y in 0..CHIP8_GFX_HEIGHT {
            for x in 0..CHIP8_GFX_WIDTH {
                self.vram[y][x] = 0;
            }
        }
        self.redraw = true;
        ProgramCounter::Next
    }

    // 00EE - RET
    fn op_00ee(&mut self) -> ProgramCounter {
        self.sp -= 1;
        let address = self.stack[self.sp] + 2;
        ProgramCounter::Jump(address)
    }

    // 1nnn - JP addr
    fn op_1nnn(&mut self, nnn: usize) -> ProgramCounter {
        ProgramCounter::Jump(nnn)
    }

    // 2nnn - CALL addr
    fn op_2nnn(&mut self, nnn: usize) -> ProgramCounter {
        self.stack[self.sp] = self.pc;
        self.sp += 1;
        ProgramCounter::Jump(nnn)
    }

    // 3xkk - SE Vx, byte
    fn op_3xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        if self.v[x] == kk {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }

    // 4xkk - SNE Vx, byte
    fn op_4xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        if self.v[x] != kk {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
     }

    // 5xy0 - SE Vx, Vy
    fn op_5xy0(&mut self, x: usize, y: usize) -> ProgramCounter {
        if self.v[x] == self.v[y] {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }

    // 6xkk - LD Vx, byte
    fn op_6xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        self.v[x] = kk;
        ProgramCounter::Next
    }

    // 7xkk - ADD Vx, byte
    fn op_7xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        self.v[x] = (self.v[x] as u16 + kk as u16) as u8;
        ProgramCounter::Next
    }

    // 8xy0 - LD Vx, Vy
    fn op_8xy0(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] = self.v[y];
        ProgramCounter::Next
    }

    // 8xy1 - OR Vx, Vy
    fn op_8xy1(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] |= self.v[y];
        ProgramCounter::Next
    }

    // 8xy2 - AND Vx, Vy
    fn op_8xy2(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] &= self.v[y];
        ProgramCounter::Next
    }

    // 8xy3 - XOR Vx, Vy
    fn op_8xy3(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] ^= self.v[y];
        ProgramCounter::Next
    }

    // 8xy4 - ADD Vx, Vy
    fn op_8xy4(&mut self, x: usize, y: usize) -> ProgramCounter {
        let vx = self.v[x] as u16;
        let vy = self.v[y] as u16;
        let result = vx + vy;
        self.v[x] = result as u8;
        self.v[0xF] = if result > 0xFF { 1 } else { 0 };
        ProgramCounter::Next
    }

    // 8xy5 - SUB Vx, Vy
    fn op_8xy5(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[0xF] = if self.v[x] > self.v[y] { 1 } else { 0 };
        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
        ProgramCounter::Next
    }

    // 8xy6 - SHR Vx {, Vy}
    fn op_8xy6(&mut self, x: usize) -> ProgramCounter {
        self.v[0xF] = self.v[x] & 0x1;
        self.v[x] >>= 1;
        ProgramCounter::Next
    }

    // 8xy7 - SUBN Vx, Vy
    // Set Vx = Vy - Vx, set VF = NOT borrow.
    // If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results
    // stored in Vx.
    fn op_8xy7(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[0xF] = if self.v[y] > self.v[x] { 1 } else { 0 };
        self.v[x] = self.v[y].wrapping_sub(self.v[x]);
        ProgramCounter::Next
    }

    // 8xyE - SHL Vx {, Vy}
    // Set Vx = Vx SHL 1.
    // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0.
    // Then Vx is multiplied by 2.
    fn op_8xye(&mut self, x: usize) -> ProgramCounter {
        self.v[0xF] = (self.v[x] & 0b10000000) >> 7;
        self.v[x] <<= 1;
        ProgramCounter::Next
    }

    // 9xy0 - SNE Vx, Vy
    // Skip next instruction if Vx != Vy.
    // The values of Vx and Vy are compared, and if they are not equal,
    // the program counter is increased by 2.
    fn op_9xy0(&mut self, x: usize, y: usize) -> ProgramCounter {
        if self.v[x] != self.v[y] {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }

    // Annn - LD I, addr
    // Set I = nnn.
    // The value of register I is set to nnn.
    fn op_annn(&mut self, nnn: usize) -> ProgramCounter {
        self.i = nnn;
        ProgramCounter::Next
    }

    // Bnnn - JP V0, addr
    // Jump to location nnn + V0.
    // The program counter is set to nnn plus the value of V0.
    fn op_bnnn(&mut self, nnn: usize) -> ProgramCounter {
        ProgramCounter::Jump(self.v[0x0] as usize + nnn)
    }

    // Cxkk - RND Vx, byte
    // Set Vx = random byte AND kk.
    // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk.
    // The results are stored in Vx.
    fn op_cxkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        let mut rng = rand::thread_rng();
        self.v[x] = rng.gen::<u8>() & kk;
        ProgramCounter::Next
    }

    // Dxyn - DRW Vx, Vy, nibble
    // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    // The interpreter reads n bytes from memory, starting at the address stored in I.
    // These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
    // Sprites are XORed onto the existing screen. If this causes any pixels to be erased,
    // VF is set to 1, otherwise it is set to 0. If the sprite is positioned so part of it is
    // outside the coordinates of the display, it wraps around to the opposite side of the screen.
    fn op_dxyn(&mut self, x: usize, y: usize, n: usize) -> ProgramCounter {
        self.v[0xF] = 0;
        for byte in 0..n {
            let y = (self.v[y] as usize + byte) % CHIP8_GFX_HEIGHT;
            for bit in 0..8 {
                let x = (self.v[x] as usize + bit) % CHIP8_GFX_WIDTH;
                let color = (self.ram[self.i + byte] >> (7 - bit)) & 1;
                self.v[0xF] |= color & self.vram[y][x];
                self.vram[y][x] ^= color;
            }
        }
        self.redraw = true;
        ProgramCounter::Next
    }

    // Ex9E - SKP Vx
    // Skip next instruction if key with the value of Vx is pressed.
    // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down
    // position, PC is increased by 2.
    fn op_ex9e(&mut self, x: usize) -> ProgramCounter {
        if self.keypad[self.v[x] as usize] {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }

    // ExA1 - SKNP Vx
    // Skip next instruction if key with the value of Vx is not pressed.
    // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up
    // position, PC is increased by 2.
    fn op_exa1(&mut self, x: usize) -> ProgramCounter {
        if !self.keypad[self.v[x] as usize] {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }

    // Fx07 - LD Vx, DT
    // Set Vx = delay timer value.
    // The value of DT is placed into Vx.
    fn op_fx07(&mut self, x: usize) -> ProgramCounter {
        self.v[x] = self.delay_timer;
        ProgramCounter::Next
    }

    // Fx0A - LD Vx, K
    // Wait for a key press, store the value of the key in Vx.
    // All execution stops until a key is pressed, then the value of that key is stored in Vx.
    fn op_fx0a(&mut self, x: usize) -> ProgramCounter {
        self.waiting_keypad = true;
        self.waiting_keypad_register = x;
        ProgramCounter::Next
    }

    // Fx15 - LD DT, Vx
    // Set delay timer = Vx.
    // DT is set equal to the value of Vx.
    fn op_fx15(&mut self, x: usize) -> ProgramCounter {
        self.delay_timer = self.v[x];
        ProgramCounter::Next
    }

    // Fx18 - LD ST, Vx
    // Set sound timer = Vx.
    // ST is set equal to the value of Vx.
    fn op_fx18(&mut self, x: usize) -> ProgramCounter {
        self.sound_timer = self.v[x];
        ProgramCounter::Next
    }

    // Fx1E - ADD I, Vx
    // Set I = I + Vx.
    // The values of I and Vx are added, and the results are stored in I.
    fn op_fx1e(&mut self, x: usize) -> ProgramCounter {
        self.i += self.v[x] as usize;
        self.v[0xF] = if self.i > 0x0F00 { 1 } else { 0 };
        ProgramCounter::Next
    }

    // Fx29 - LD F, Vx
    // Set I = location of sprite for digit Vx.
    // The value of I is set to the location for the hexadecimal sprite corresponding to the value
    // of Vx.
    fn op_fx29(&mut self, x: usize) -> ProgramCounter {
        self.i = (self.v[x] as usize) * 5;
        ProgramCounter::Next
    }

    // Fx33 - LD B, Vx
    // Store BCD representation of Vx in memory locations I, I+1, and I+2.
    // The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at
    // location in I, the tens digit at location I+1, and the ones digit at location I+2.
    fn op_fx33(&mut self, x: usize) -> ProgramCounter {
        self.ram[self.i] = self.v[x] / 100;
        self.ram[self.i + 1] = (self.v[x] % 100) / 10;
        self.ram[self.i + 2] = self.v[x] % 10;
        ProgramCounter::Next
    }

    // LD [I], Vx
    // The interpreter copies the values of registers V0 through Vx
    // into memory, starting at the address in I.
    fn op_fx55(&mut self, x: usize) -> ProgramCounter {
        for i in 0..x + 1 {
            self.ram[self.i + i] = self.v[i];
        }
        ProgramCounter::Next
    }

    // LD Vx, [I]
    // The interpreter reads values from memory starting at location
    // I into registers V0 through Vx.
    fn op_fx65(&mut self, x: usize) -> ProgramCounter {
        for i in 0..x + 1 {
            self.v[i] = self.ram[self.i + i];
        }
        ProgramCounter::Next
    }
}


static FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

#[cfg(test)]
#[path = "./cpu_tests.rs"]
mod cpu_tests;