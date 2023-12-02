extern crate rand;
extern crate sdl2;

use rand::rngs::ThreadRng;
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, BlendMode};
use sdl2::video::Window;
use std::time::Duration;

// Game関連定数
const BOARD_X_LEN: usize = 12;
const BOARD_X_MIN: usize = 0;
const BOARD_X_MAX: usize = BOARD_X_LEN - 1;
const BOARD_Y_LEN: usize = 21;
const BOARD_Y_MIN: usize = 0;
const BOARD_Y_MAX: usize = BOARD_Y_LEN - 1;
const LEFT_WALL_X: i32 = 6;

// Presenter関連定数
const SCREEN_WIDTH: u32 = 640;
const SCREEN_HEIGHT: u32 = 420;
const CELL_SIZE_PX: u32 = 20;
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
    canvas.set_blend_mode(BlendMode::Blend);
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

    // render piles
    canvas.set_draw_color(Color::RGB(128, 128, 128));
    for i in 0..(game.piles.pattern.len()) {
        for j in 0..(game.piles.pattern[i].len()) {
            if game.piles.pattern[i][j] >= 1 {
                let color = get_color(game.piles.pattern[i][j]);
                canvas.set_draw_color(color);
                let x: i32 = (LEFT_WALL_X + j as i32) as i32 * CELL_SIZE_PX as i32;
                let y: i32 = i as i32 * CELL_SIZE_PX as i32;
                canvas.fill_rect(Rect::new(x, y, CELL_SIZE_PX, CELL_SIZE_PX))?;
            }
        }
    }

    // render block
    render_block(canvas, &game.block, LEFT_WALL_X + game.block.pos.x as i32, game.block.pos.y as i32)?;

    // render next block
    render_block(canvas, &game.next_block, 21, 0)?;

    if game.is_over {
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 128));
        canvas.fill_rect(Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT))?;
    }

    canvas.present();

    Ok(())
}

fn render_block(canvas: &mut Canvas<Window>, block: &Block, x_in_cell: i32, y_in_cell: i32) -> Result<(), String> {
    let block_color = match block.color {
        0 => Color::RGB(255, 128, 128),
        1 => Color::RGB(128, 255, 128),
        2 => Color::RGB(128, 128, 255),
        _ => Color::RGB(255, 255, 255),
    };
    canvas.set_draw_color(block_color);
    let pattern = block.get_pattern();
    for j in 0..pattern.len() {
        for i in 0..pattern[j].len() {
            if pattern[j][i] == 1 {
                let x = (x_in_cell + i as i32) * CELL_SIZE_PX as i32;
                let y = (y_in_cell + j as i32) * CELL_SIZE_PX as i32;
                canvas.fill_rect(Rect::new(x, y, CELL_SIZE_PX, CELL_SIZE_PX))?;
            }
        }
    }
    Ok(())
}

fn get_color(color_num: u8) -> Color {
    match color_num {
        0 => Color::RGB(0, 0, 0),
        1 => Color::RGB(128, 128, 128),
        2 => Color::RGB(255, 128, 128),
        3 => Color::RGB(128, 255, 128),
        4 => Color::RGB(128, 128, 255),
        _ => Color::RGB(255, 255, 255),
    }
}

#[derive(Debug, Clone, Copy)]
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

fn print_pattern<const W: usize, const H: usize, T: std::fmt::Debug>(pattern: [[T;W];H]) {
    for line in pattern {
        println!("{:?}", line);
    }
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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

    fn rotate(&mut self, dir: i32) {
        if dir > 0 {
            self.rot = (self.rot + 1) % 4;
        } else {
            self.rot = (self.rot + 3) % 4;
        }
    }

    fn move_by_delta(&mut self, x_delta: i32, y_delta: i32) {
        self.pos.x += x_delta;
        self.pos.y += y_delta;
    }

    fn create_randomly(rng: &mut ThreadRng, created_count: u32) -> Block {
        let mut block = Block::new();
        block.pos = PosInCell::new(4, 0);
        block.shape = Shape::from_i32(rng.gen_range(0..=Shape::max()));
        // block.shape = Shape::from_i32(0);
        block.color = (created_count % 3) as u8;
        block
    }
}

struct Piles {
    pattern: [[u8; BOARD_X_LEN]; BOARD_Y_LEN], // 0:なし 1:壁or床 2〜:ブロック残骸
}

impl Piles {
    fn new() -> Piles {
        Piles {
            pattern: [[0; BOARD_X_LEN]; BOARD_Y_LEN],
        }
    }

    fn setup_wall_and_floor(&mut self) {
        for i in BOARD_Y_MIN..=BOARD_Y_MAX {
            self.pattern[i][BOARD_X_MIN] = 1;
            self.pattern[i][BOARD_X_MAX] = 1;
        }
        for i in BOARD_X_MIN..=BOARD_X_MAX {
            self.pattern[BOARD_Y_MAX][i] = 1;
        }
    }

    fn is_filled(&self, x: usize, y: usize) -> bool {
        self.pattern[y][x] >= 1
    }
}

// ゲームのモデル。SDLに依存しない。
struct Game {
    rng: ThreadRng,
    is_over: bool,
    frame: i32,
    erase_row_wait: i32,
    piles: Piles,
    block: Block,
    next_block: Block,
    block_created_count: u32,
}

impl Game {
    fn new(rng: ThreadRng) -> Game {
        let mut game = Game {
            rng: rng,
            is_over: false,
            frame: 0,
            erase_row_wait: 0,
            piles: Piles::new(),
            block: Block::new(),
            next_block: Block::new(),
            block_created_count: 0,
        };
        game.piles.setup_wall_and_floor();
        game.next_block = Block::create_randomly(&mut game.rng, game.block_created_count);
        game.block_created_count += 1;

        game.spawn_block();
        game
    }

    fn update(&mut self, command: &str) {
        if self.is_over {
            return
        }
        if self.erase_row_wait <= 0 {
            match command {
                "right" => self.move_by_delta(1, 0),
                "left" => self.move_by_delta(-1, 0),
                "down" => self.move_by_delta(0, 1),
                "rotate_left" => self.rotate(1),
                "rotate_right" => self.rotate(-1),
                _ => {}
            }

            if self.frame != 0 && self.frame % 20 == 0 {
                self.move_by_delta(0, 1);
                if self.is_collide(0, 0) {
                    println!("Game over!");
                    self.is_over = true;
                }
            }
            self.check_erase_row();
        } else {
            self.erase_row_wait -= 1;
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
        if !self.is_collide(x_delta, y_delta) {
            self.block.move_by_delta(x_delta, y_delta);
        }

        // 床に接触した
        if y_delta > 0 && self.is_collide(0, 1) {
            self.settle_block();
            self.spawn_block();
        }
    }

    fn settle_block(&mut self) {
        for i in 0..5 {
            for j in 0..5 {
                let block_pattern = self.block.get_pattern();
                if block_pattern[i][j] == 1 {
                    self.piles.pattern[(self.block.pos.y + i as i32) as usize]
                        [(self.block.pos.x + j as i32) as usize] = 2  +self.block.color;
                }
            }
        }
    }

    fn rotate(&mut self, dir: i32) {
        self.block.rotate(dir);
        if self.is_collide(0, 0) {
            self.block.rotate(-dir);
        }
    }

    fn spawn_block(&mut self) {
        self.block = self.next_block.clone();
        self.next_block = Block::create_randomly(&mut self.rng, self.block_created_count);
        self.block_created_count += 1;
    }

    fn check_erase_row(&mut self) {
        let filled_rows = self.get_filled_rows();
        if filled_rows.len() > 0 {
            println!("Before:");
            print_pattern(self.piles.pattern);

            // そろった行を空にする
            for y in &filled_rows {
                for x in 1..=(BOARD_X_MAX - 1) {
                    self.piles.pattern[*y][x] = 0;
                }
            }

            // 下に支えるブロック片がないブロック片を落下させる
            for x in 1..=(BOARD_X_MAX - 1) {    // 各列に対して
                for y in (BOARD_Y_MIN..=(BOARD_Y_MAX - 1)).rev() {  // 下から見ていく
                    if self.piles.pattern[y][x] > 0 {  // ブロック片が存在するなら
                        let mut below = y + 1;  // それより下でブロック片が存在するマスを探す
                        while below < BOARD_Y_MAX && self.piles.pattern[below][x] == 0 {
                            below += 1;
                        }
                        if below != y + 1 {
                            self.piles.pattern[below - 1][x] = self.piles.pattern[y][x];    // ブロック片を下に移動
                            self.piles.pattern[y][x] = 0;   // もとのマスを空にする
                        }
                    }
                }
            }
            self.erase_row_wait = 20;

            println!("After:");
            print_pattern(self.piles.pattern);
        }
    }

    fn get_filled_rows(&self) -> Vec<usize> {
        let mut result = Vec::<usize>::new();
        for y in BOARD_Y_MIN..=(BOARD_Y_MAX - 1) {
            if (1..=(BOARD_X_MAX - 1)).all(|x| self.piles.is_filled(x, y)) {
                result.push(y);
            }
        }
        result
    }
}

mod tests {
    use super::*;

    // 消える行がある場合
    #[test]
    fn test_check_erase_row() {
        let rng = rand::thread_rng();
        let mut game = Game::new(rng);
        game.piles.pattern = [
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];

        game.check_erase_row();

        assert_eq!(
            game.piles.pattern,
            [
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            ]
        );
    }

    // 消える行がない場合
    #[test]
    fn test_check_erase_row2() {
        let rng = rand::thread_rng();
        let mut game = Game::new(rng);
        game.piles.pattern = [
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];

        game.check_erase_row();

        assert_eq!(
            game.piles.pattern,
            [
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                ]
        );
    }

    //
    #[test]
    fn test_check_erase_row3() {
        let rng = rand::thread_rng();
        let mut game = Game::new(rng);
        game.piles.pattern = [
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];

        game.check_erase_row();

        assert_eq!(
            game.piles.pattern,
            [
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            ]
        );
    }
}
