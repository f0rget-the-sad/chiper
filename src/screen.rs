extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;

use crate::chip8::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub trait Screen {
    /// Creates new Screen
    //fn new() -> Result<Self, String>
    //where
    //    Self: Sized;

    /// Clears the screen
    fn clear(&mut self);

    /// Draw white pixel at pox `x`, `y`
    fn draw_px(&mut self, x: i32, y: i32);

    /// Draw black pixel at pox `x`, `y`
    fn clear_px(&mut self, x: i32, y: i32);

    /// Visualize all changes
    fn present(&mut self);
}

pub struct SdlScreen {
    canvas: Canvas<Window>,
}

impl SdlScreen {
    const PX_SIZE: u32 = 16;
    const WIDTH: u32 = SCREEN_WIDTH * SdlScreen::PX_SIZE;
    const HEIGHT: u32 = SCREEN_HEIGHT * SdlScreen::PX_SIZE;

    fn from_sdl_conext(sdl_context: &Sdl) -> Result<Self, String> {
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window(
                "Chiper: CHIP-8 emulator",
                SdlScreen::WIDTH,
                SdlScreen::HEIGHT,
            )
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window
            .into_canvas()
            .software()
            .build()
            .map_err(|e| e.to_string())?;

        Ok(SdlScreen { canvas: canvas })
    }
}

impl Screen for SdlScreen {
    fn clear(&mut self) {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
    }

    fn draw_px(&mut self, x: i32, y: i32) {
        self.canvas.set_draw_color(Color::WHITE);
        self.canvas
            .fill_rect(Rect::new(
                x * SdlScreen::PX_SIZE as i32,
                y * SdlScreen::PX_SIZE as i32,
                SdlScreen::PX_SIZE,
                SdlScreen::PX_SIZE,
            ))
            .unwrap();
    }

    fn clear_px(&mut self, x: i32, y: i32) {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas
            .fill_rect(Rect::new(
                x * SdlScreen::PX_SIZE as i32,
                y * SdlScreen::PX_SIZE as i32,
                SdlScreen::PX_SIZE,
                SdlScreen::PX_SIZE,
            ))
            .unwrap();
    }

    fn present(&mut self) {
        self.canvas.present();
    }
}

pub fn sdl_init() -> Result<SdlScreen, String> {
    let sdl_context = sdl2::init()?;
    let mut screen = SdlScreen::from_sdl_conext(&sdl_context)?;
    screen.clear();
    Ok(screen)
}

pub fn run() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let mut screen = SdlScreen::from_sdl_conext(&sdl_context)?;

    screen.clear();

    let mut events = sdl_context.event_pump()?;

    let mut cnt = 0;
    'mainloop: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                //Event::MouseButtonDown { x, y, .. } => {
                //    screen.fill_rect((0 + cnt * SdlScreen::PX_SIZE) as i32, 0);
                //    cnt += 1;
                //}
                Event::KeyDown {
                    keycode: Option::Some(Keycode::Q),
                    ..
                } => {
                    cnt -= 1;
                    screen.clear_px(cnt as i32, 0);
                }
                Event::KeyDown {
                    keycode: Option::Some(Keycode::W),
                    ..
                } => {
                    screen.draw_px(cnt as i32, 0);
                    cnt += 1;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
