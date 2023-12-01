extern crate rand;
extern crate sdl2;

use rand::rngs::ThreadRng;
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Duration;

const SCREEN_WIDTH: u32 = 640;
const SCREEN_HEIGHT: u32 = 420;
const CELL_SIZE_PX: u32 = 20;
const BACKGROUND_COLOR: Color = Color::RGB(128, 128, 128);
const BOX_COLOR: Color = Color::RGB(255, 128, 128);

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

    let rng = rand::thread_rng();
    let mut game = Game::new(rng);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => game.update(code),
                _ => {}
            }
        }
        render(&mut canvas, &game)?;
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }

    Ok(())
}

fn render(canvas: &mut Canvas<Window>, game: &Game) -> Result<(), String> {
    canvas.set_draw_color(BACKGROUND_COLOR);
    canvas.clear();

    let block_color = match game.block.color {
        0 => Color::RGB(255, 128, 128),
        1 => Color::RGB(128, 255, 128),
        2 => Color::RGB(128, 128, 255),
        _ => Color::RGB(255, 255, 255),
    };
    canvas.set_draw_color(block_color);
    let pattern = game.block.get_pattern();
    for j in 0..pattern.len() {
        for i in 0..pattern[j].len() {
            if pattern[j][i] == 1 {
                let x = ((game.block.pos.x + i as i32) * CELL_SIZE_PX as i32) as i32;
                let y = ((game.block.pos.y + j as i32) * CELL_SIZE_PX as i32) as i32;
                canvas.fill_rect(Rect::new(x, y, CELL_SIZE_PX, CELL_SIZE_PX))?;
            }
        }
    }

    canvas.present();

    Ok(())
}

struct PosInCell {
    x: i32,
    y: i32,
}

impl PosInCell {
    fn new() -> PosInCell {
        PosInCell { x: 0, y: 0 }
    }
}

enum Shape {
    S0 = 0,
    S1,
    S2,
    S3,
    S4,
    S5,
}

impl Shape {
    fn max() -> i32 {
        5
    }
}

impl Shape {
    fn from_i32(n: i32) -> Shape {
        match n {
            0 => Shape::S0,
            1 => Shape::S1,
            2 => Shape::S2,
            3 => Shape::S3,
            4 => Shape::S4,
            5 => Shape::S5,
            _ => panic!("Unknown value for Shape: {}", n),
        }
    }
}

struct Block {
    pos: PosInCell,
    shape: Shape,
    rot: i8,
    color: u8,
}
impl Block {
    fn new() -> Block {
        Block {
            pos: PosInCell::new(),
            shape: Shape::S0,
            rot: 0,
            color: 0,
        }
    }

    fn get_pattern(&self) -> [[u8; 5]; 5] {
        // TODO: self.shapeで場合分け
        let mut base: [[u8; 5]; 5] = [
            [0, 0, 0, 0, 0],
            [0, 0, 1, 0, 0],
            [0, 1, 1, 1, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ];
        let mut result = base;
        for n in 0..self.rot {
            result = Self::rotate_pattern(result);
        }
        match self.rot {
            0 => result,
            1 => result,
            2 => result,
            3 => result,
            _ => panic!(),
        }
    }

    fn rotate_pattern(base: [[u8; 5]; 5]) -> [[u8; 5]; 5] {
        let mut result: [[u8; 5]; 5] = [[0; 5]; 5];
        for i in 0..5 {
            for j in 0..5 {
                result[4 - j][i] = base[i][j];
            }
        }
        result
    }

    fn rotate_right(&mut self) {
        self.rot = (self.rot + 3) % 4;
    }

    fn rotate_left(&mut self) {
        self.rot = (self.rot + 1) % 4;
    }

    fn move_by_delta(&mut self, x_delta: i32, y_delta: i32) {
        self.pos.x += x_delta;
        self.pos.y += y_delta;
    }

    fn create_randomly(rng: &mut ThreadRng) -> Block {
        let mut block = Block::new();
        block.shape = Shape::from_i32(rng.gen_range(0..=Shape::max()));
        block.color = rng.gen_range(0..=2);
        block
    }
}

// ゲームのモデル。できればSDLに依存しないようにしたい
struct Game {
    block: Block,
    rng: ThreadRng,
}

impl Game {
    fn new(rng: ThreadRng) -> Game {
        let mut game = Game {
            block: Block::new(),
            rng: rng,
        };
        game.set_next_block();
        game
    }

    fn update(&mut self, keycode: Keycode) {
        // println!("key pressed: {}", keycode);
        match keycode {
            Keycode::Right => self.block.move_by_delta(1, 0),
            Keycode::Left => self.block.move_by_delta(-1, 0),
            Keycode::Up => self.block.move_by_delta(0, -1),
            Keycode::Down => self.block.move_by_delta(0, 1),
            Keycode::Z => self.block.rotate_left(),
            Keycode::X => self.block.rotate_right(),
            _ => {}
        }
    }

    // fn create_next_block(&mut self) -> Block {
    //     let mut block = Block::new();
    //     block.shape = Shape::from_i32(self.rng.gen_range(0..=Shape::max()));
    //     block.color = self.rng.gen_range(0..=2);
    //     block
    // }

    fn set_next_block(&mut self) {
        self.block = Block::create_randomly(&mut self.rng);
    }
}
