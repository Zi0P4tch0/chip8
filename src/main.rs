#[cfg(test)]
extern crate tempfile;

extern crate rand;
extern crate sdl2;

mod cpu;
mod display;
mod keyboard;
mod sound;

use std::env;
use std::fs::File;
use std::thread;
use std::time::Duration;

use crate::display::Display;
use crate::keyboard::Keyboard;
use crate::sound::Sound;

fn main() {

    let mut cpu = cpu::CPU::new();
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let mut file = File::open(filename).unwrap();
    cpu.load_game(&mut file);

    let sdl_context = sdl2::init().unwrap();

    let mut display = Display::new(&sdl_context);
    let mut keyboard = Keyboard::new(&sdl_context);
    let sound = Sound::new(&sdl_context);

    while let Ok(keypad) = keyboard.poll() {
        cpu.tick(keypad);
        if cpu.sound_timer > 0 {
            sound.start_beep();
        } else {
            sound.stop_beep();
        }
        if cpu.redraw {
            display.draw(&cpu.vram);
        }
        thread::sleep(Duration::from_millis(2));
    }

}
