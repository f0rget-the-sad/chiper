extern crate sdl2;

use crate::Chip8;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;

trait Screen {
    /// Creates new Screen
    //fn new() -> Result<Self, String>
    //where
    //    Self: Sized;

    /// Clears the screen
    fn clear(&mut self);

    /// Draw white square at pox `x`, `y` with given `width`, `height`
    fn fill_rect(&mut self, x: i32, y: i32);
}

struct SdlScreen {
    canvas: Canvas<Window>,
}

impl SdlScreen {
    const PX_SIZE: u32 = 16;
    const WIDTH: u32 = Chip8::SCREEN_WIDTH * SdlScreen::PX_SIZE;
    const HEIGHT: u32 = Chip8::SCREEN_HEIGHT * SdlScreen::PX_SIZE;

    fn from_sdl_conext(sdl_context: &Sdl) -> Result<Self, String> {
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window(
                "rust-sdl2 demo: Cursor",
                SdlScreen::WIDTH,
                SdlScreen::HEIGHT,
            )
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window
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
        // XXX: not sure about present here, better to use once per loop
        self.canvas.present();
    }

    fn fill_rect(&mut self, x: i32, y: i32) {
        self.canvas.set_draw_color(Color::WHITE);
        self.canvas
            .fill_rect(Rect::new(x, y, SdlScreen::PX_SIZE, SdlScreen::PX_SIZE))
            .unwrap();
        self.canvas.present();
    }
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
                Event::MouseButtonDown { x, y, .. } => {
                    screen.fill_rect((0 + cnt * SdlScreen::PX_SIZE) as i32, 0);
                    cnt += 1;
                }
                _ => {}
            }
        }
    }

    Ok(())
}