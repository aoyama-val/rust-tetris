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
const FPS: u32 = 30;

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
        let mut command = "";
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
                } => {
                    command = match code {
                        Keycode::Left => "left",
                        Keycode::Right => "right",
                        Keycode::Down => "down",
                        Keycode::Z => "rotate_left",
                        Keycode::X => "rotate_right",
                        _ => "",
                    };
                }
                _ => {}
            }
        }
        game.update(command);
        render(&mut canvas, &game)?;
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FPS));
    }

    Ok(())
}

fn render(canvas: &mut Canvas<Window>, game: &Game) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    const CELL_SIZE_PX: u32 = 20;
    const LEFT_WALL_X: i32 = 6;

    // render piles
    canvas.set_draw_color(Color::RGB(128, 128, 128));
    for i in 0..(game.piles.pattern.len()) {
        for j in 0..(game.piles.pattern[i].len()) {
            if game.piles.pattern[i][j] == 1 {
                let x: i32 = (LEFT_WALL_X + j as i32) as i32 * CELL_SIZE_PX as i32;
                let y: i32 = i as i32 * CELL_SIZE_PX as i32;
                canvas.fill_rect(Rect::new(x, y, CELL_SIZE_PX, CELL_SIZE_PX))?;
            }
        }
    }

    // render block
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
                let x = ((LEFT_WALL_X + game.block.pos.x + i as i32) * CELL_SIZE_PX as i32) as i32;
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
    fn new(x: i32, y: i32) -> PosInCell {
        PosInCell { x, y }
    }
}

type Pattern = [[u8; 5]; 5];

enum Shape {
    S0 = 0,
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
}

impl Shape {
    fn max() -> i32 {
        6
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
            6 => Shape::S6,
            _ => panic!("Unknown value for Shape: {}", n),
        }
    }

    fn get_base_pattern(&self) -> Pattern {
        let base: Pattern = match self {
            Shape::S0 => [
                [0, 0, 0, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 0, 1, 0, 0],
            ],
            Shape::S1 => [
                [0, 0, 0, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 0, 1, 1, 0],
                [0, 0, 0, 0, 0],
            ],
            Shape::S2 => [
                [0, 0, 0, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 1, 1, 0, 0],
                [0, 0, 0, 0, 0],
            ],
            Shape::S3 => [
                [0, 0, 0, 0, 0],
                [0, 0, 1, 1, 0],
                [0, 0, 1, 1, 0],
                [0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0],
            ],
            Shape::S4 => [
                [0, 0, 0, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 1, 1, 1, 0],
                [0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0],
            ],
            Shape::S5 => [
                [0, 0, 0, 0, 0],
                [0, 0, 1, 1, 0],
                [0, 1, 1, 0, 0],
                [0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0],
            ],
            Shape::S6 => [
                [0, 0, 0, 0, 0],
                [0, 1, 1, 0, 0],
                [0, 0, 1, 1, 0],
                [0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0],
            ],
        };
        base
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
            pos: PosInCell::new(0, 0),
            shape: Shape::S0,
            rot: 0,
            color: 0,
        }
    }

    fn get_pattern(&self) -> Pattern {
        let base = self.shape.get_base_pattern();
        let mut result = base;
        for _ in 0..self.rot {
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

    fn rotate_pattern(base: Pattern) -> Pattern {
        let mut result: Pattern = [[0; 5]; 5];
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
        block.pos = PosInCell::new(4, 0);
        block.shape = Shape::from_i32(rng.gen_range(0..=Shape::max()));
        block.color = rng.gen_range(0..=2);
        block
    }
}

struct Piles {
    pattern: [[u8; 12]; 21],
}

impl Piles {
    fn new() -> Piles {
        Piles {
            pattern: [[0; 12]; 21],
        }
    }

    fn setup_wall_and_floor(&mut self) {
        for i in 0..21 {
            self.pattern[i][0] = 1;
            self.pattern[i][11] = 1;
        }
        for i in 0..12 {
            self.pattern[20][i] = 1;
        }
    }

    fn is_filled(&self, x: usize, y: usize) -> bool {
        self.pattern[y][x] == 1
    }
}

// ゲームのモデル。SDLに依存しない。
struct Game {
    block: Block,
    piles: Piles,
    rng: ThreadRng,
    frame: i32,
}

impl Game {
    fn new(rng: ThreadRng) -> Game {
        let mut game = Game {
            block: Block::new(),
            piles: Piles::new(),
            rng: rng,
            frame: 0,
        };
        game.piles.setup_wall_and_floor();
        game.set_next_block();
        game
    }

    fn update(&mut self, command: &str) {
        match command {
            "right" => self.move_by_delta(1, 0),
            "left" => self.move_by_delta(-1, 0),
            "down" => self.move_by_delta(0, 1),
            "rotate_left" => self.block.rotate_left(),
            "rotate_right" => self.block.rotate_right(),
            // Keycode::A => self.set_next_block(),
            _ => {}
        }

        if self.frame != 0 && self.frame % 10 == 0 {
            self.move_by_delta(0, 1);
        }

        self.frame += 1;
    }

    fn is_collide(&mut self, x_delta: i32, y_delta: i32) -> bool {
        let pattern = self.block.get_pattern();
        for i in 0..5 {
            for j in 0..5 {
                if pattern[i][j] != 0 {
                    let new_x = self.block.pos.x + j as i32 + x_delta;
                    let new_y = self.block.pos.y + i as i32 + y_delta;
                    if self.piles.is_filled(new_x as usize, new_y as usize) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn move_by_delta(&mut self, x_delta: i32, y_delta: i32) {
        if self.is_collide(x_delta, y_delta) {
            return;
        }
        self.block.move_by_delta(x_delta, y_delta);

        // 床に接触した
        if y_delta > 0 && self.is_collide(0, 1) {
            for i in 0..5 {
                for j in 0..5 {
                    let block_pattern = self.block.get_pattern();
                    if block_pattern[i][j] == 1 {
                        self.piles.pattern[(self.block.pos.y + i as i32) as usize]
                            [(self.block.pos.x + j as i32) as usize] = 1;
                    }
                }
            }
            self.set_next_block();
            if self.is_collide(0, 0) {
                println!("Game over!");
            }
        }
    }

    fn set_next_block(&mut self) {
        self.block = Block::create_randomly(&mut self.rng);
    }
}
