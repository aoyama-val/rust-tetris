extern crate sdl2;

use std::time::Duration;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

const SCREEN_WIDTH: u32 = 640;
const SCREEN_HEIGHT: u32 = 420;
const CELL_SIZE_PX: u32 = 20;

// SCREEN_WIDTH = 640
// SCREEN_HEIGHT = 420
// CELL_SIZE_PX = 20
// BACKGROUND_COLOR = [0, 0, 0]
// WALL_COLOR = [128, 128, 128]
// BLOCK_COLORS = [
//   [255, 0, 0],
//   [0, 255, 0],
//   [0, 0, 255],
//   [255, 255, 0],
//   [0, 255, 255],
//   [255, 0, 255],
// ]
// WALL_X = 5
// WALL_Y = 0

// define_key SDL::Key::ESCAPE, :exit
// define_key SDL::Key::LEFT, :left
// define_key SDL::Key::RIGHT, :right
// define_key SDL::Key::UP, :up
// define_key SDL::Key::DOWN, :down
// define_key SDL::Key::Z, :rotate_left
// define_key SDL::Key::X, :rotate_right
// define_key SDL::Key::F5, :reload
// define_key SDL::Key::RETURN, :ok
// define_pad_button 0, :ok



pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-tetris", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    let mut game = Game::new();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. }
                    => break 'running,
                Event::KeyDown { keycode: Some(code), .. } => { game.update(code) },
                _ => {}
            }
        }
        if let Err(s) = render(&mut canvas, &game) {
            return Err(s);
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }

    Ok(())
}

fn render(canvas: &mut Canvas<Window>, game: &Game) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(128, 128, 128));
    canvas.clear();
    canvas.set_draw_color(Color::RGB(255, 128, 128));
    canvas.fill_rect(Rect::new(game.obj_x as i32, game.obj_y as i32, 10, 10))?;
    canvas.present();

    Ok(())
}

// ゲームのモデル。できればSDLに依存しないようにしたい
struct Game {
    obj_x: u32,
    obj_y: u32,
}

impl Game {
    fn new() -> Game {
        Game {
            obj_x: SCREEN_WIDTH / 2,
            obj_y: SCREEN_HEIGHT / 2,
        }
    }
    fn update(&mut self, keycode: Keycode) {
        // println!("key pressed: {}", keycode);
        match keycode {
            Keycode::Right => self.obj_x += 5,
            Keycode::Left => self.obj_x -= 5,
            Keycode::Up => self.obj_y -= 5,
            Keycode::Down => self.obj_y += 5,
            _ => {},
        }
    }
}
