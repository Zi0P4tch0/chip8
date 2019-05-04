
use sdl2;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels;
use sdl2::rect::Rect;

use crate::cpu::CHIP8_GFX_WIDTH;
use crate::cpu::CHIP8_GFX_HEIGHT;

const SCALE_FACTOR: u32 = 20;
const SCREEN_WIDTH: u32 = (CHIP8_GFX_WIDTH as u32) * SCALE_FACTOR;
const SCREEN_HEIGHT: u32 = (CHIP8_GFX_HEIGHT as u32) * SCALE_FACTOR;

pub struct Display {
    canvas: Canvas<Window>
}

impl Display {

    pub fn new(context: &sdl2::Sdl) -> Display {

        let video_subsys = context.video().unwrap();
        let window = video_subsys
            .window(
                "CHIP 8 emulator in Rust",
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
            )
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Display { canvas }
    }


    pub fn draw(&mut self, pixels: &[[u8; CHIP8_GFX_WIDTH]; CHIP8_GFX_HEIGHT]) {
        for (y, row) in pixels.iter().enumerate() {
            for (x, &col) in row.iter().enumerate() {
                let x = (x as u32) * SCALE_FACTOR;
                let y = (y as u32) * SCALE_FACTOR;
                let color =self.color(col);
                self.canvas.set_draw_color(color);
                let rect = Rect::new(x as i32, y as i32, SCALE_FACTOR, SCALE_FACTOR);
                let _ = self.canvas.fill_rect(rect);
            }
        }
        self.canvas.present();
    }

    fn color(&mut self, value: u8) -> pixels::Color {
        if value == 0 {
            pixels::Color::RGB(0, 0, 0)
        } else {
            pixels::Color::RGB(200, 0, 0)
        }
    }

}
