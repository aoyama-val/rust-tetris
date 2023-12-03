use rand::prelude::*;
use std::fs::File;
use std::io::Read;
use std::time;
use toml::Table;

// Game関連定数
pub const BOARD_X_LEN: usize = 12;
pub const BOARD_X_MIN: usize = 0;
pub const BOARD_X_MAX: usize = BOARD_X_LEN - 1;
pub const BOARD_Y_LEN: usize = 21;
pub const BOARD_Y_MIN: usize = 0;
pub const BOARD_Y_MAX: usize = BOARD_Y_LEN - 1;
pub const LEFT_WALL_X: i32 = 6;

pub type Pattern = [[u8; 5]; 5];

fn print_pattern<const W: usize, const H: usize, T: std::fmt::Debug>(pattern: [[T; W]; H]) {
    for line in pattern {
        println!("{:?}", line);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Shape {
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
        // 回転していない状態の形状
        // 各角度での形状を持つようにした方が良かった。そうしないと、四角形が回転しただけで移動してしまうなど不自然
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
pub struct Block {
    pub x: i32,
    pub y: i32,
    pub shape: Shape,
    pub rot: i8,
    pub color: u8,
}
impl Block {
    fn new() -> Block {
        Block {
            x: 0,
            y: 0,
            shape: Shape::S0,
            rot: 0,
            color: 0,
        }
    }

    pub fn get_pattern(&self) -> Pattern {
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
        self.x += x_delta;
        self.y += y_delta;
    }

    fn create_randomly(rng: &mut StdRng, created_count: u32) -> Block {
        let mut block = Block::new();
        block.x = 4;
        block.y = 0;
        block.shape = Shape::from_i32(rng.gen_range(0..=Shape::max()));
        // block.shape = Shape::from_i32(0);
        block.color = (created_count % 3) as u8;
        block
    }
}

// 壁と床を含めた堆積物を表す構造体
// 壁と床は別にした方が良かったかも
pub struct Piles {
    pub pattern: [[u8; BOARD_X_LEN]; BOARD_Y_LEN], // 0:なし 1:壁or床 2〜:ブロック残骸
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
pub struct Game {
    pub rng: StdRng,
    pub is_over: bool,
    pub frame: i32,
    pub settle_wait: u32,
    pub piles: Piles,
    pub block: Block,
    pub next_block: Block,
    pub block_created_count: u32,
}

impl Game {
    pub fn new() -> Game {
        let now = time::SystemTime::now();
        let timestamp = now
            .duration_since(time::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();
        let rng = StdRng::seed_from_u64(timestamp);

        let mut game = Game {
            rng: rng,
            is_over: false,
            frame: 0,
            settle_wait: 0,
            piles: Piles::new(),
            block: Block::new(),
            next_block: Block::new(),
            block_created_count: 0,
        };
        game.piles.setup_wall_and_floor();
        game.next_block = Block::create_randomly(&mut game.rng, game.block_created_count);
        game.block_created_count += 1;

        game.piles = Piles {
            pattern: [
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
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            ],
        };

        game.spawn_block();
        game
    }

    pub fn load_config(&mut self) {
        let filename = "tetris.toml";
        let mut f = match File::open(filename) {
            Ok(f) => f,
            Err(_) => return,
        };

        let mut content = String::new();
        f.read_to_string(&mut content).unwrap();

        let parsed = content.parse::<Table>().unwrap();
        if let Some(value) = parsed.get("seed") {
            if let Some(seed) = value.as_integer() {
                println!("seed = {}", seed);
                let rng = StdRng::seed_from_u64(seed as u64);
                self.rng = rng;
            }
        }

        if let Some(value) = parsed.get("pattern") {
            if let Some(pattern) = value.as_str() {
                let mut p: Piles = Piles {
                    pattern: [[0; BOARD_X_LEN]; BOARD_Y_LEN],
                };
                for (i, line) in pattern.lines().enumerate() {
                    for (j, col) in line.split_ascii_whitespace().enumerate() {
                        p.pattern[i][j] = col.parse::<u8>().unwrap();
                    }
                    // println!("line = [{}]", line);
                }
                println!("pattern:");
                print_pattern(p.pattern);
                self.piles = p;
            }
        }
    }

    pub fn update(&mut self, command: &str) {
        if self.is_over {
            return;
        }

        if self.settle_wait > 0 {
            self.settle_wait -= 1;
            if self.settle_wait == 0 {
                // 床に接触した
                if self.is_collide(0, 1) {
                    self.settle_block();
                    self.spawn_block();
                }
            }
        }

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
            } else {
                self.settle_wait = 15;
            }
        }
        self.check_erase_row();

        self.frame += 1;
    }

    fn is_collide(&mut self, x_delta: i32, y_delta: i32) -> bool {
        let pattern = self.block.get_pattern();
        for i in 0..5 {
            for j in 0..5 {
                if pattern[i][j] != 0 {
                    let new_x = self.block.x + j as i32 + x_delta;
                    let new_y = self.block.y + i as i32 + y_delta;
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
    }

    fn settle_block(&mut self) {
        for i in 0..5 {
            for j in 0..5 {
                let block_pattern = self.block.get_pattern();
                if block_pattern[i][j] == 1 {
                    self.piles.pattern[(self.block.y + i as i32) as usize]
                        [(self.block.x + j as i32) as usize] = 2 + self.block.color;
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
        self.block = self.next_block;
        self.next_block = Block::create_randomly(&mut self.rng, self.block_created_count);
        self.block_created_count += 1;
    }

    fn check_erase_row(&mut self) {
        let filled_rows = self.get_filled_rows();
        if filled_rows.len() > 0 {
            // println!("Before:");
            // print_pattern(self.piles.pattern);

            // そろった行を消す
            let max_filled_row = filled_rows[filled_rows.len() - 1];
            for y in (0..=max_filled_row).rev() {
                for x in 1..=(BOARD_X_MAX - 1) {
                    let above = y as i32 - filled_rows.len() as i32;
                    if above >= 0 {
                        self.piles.pattern[y][x] = self.piles.pattern[above as usize][x];
                    } else {
                        self.piles.pattern[y][x] = 0;
                    }
                }
            }

            // println!("After:");
            // print_pattern(self.piles.pattern);
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

#[cfg(test)]
mod tests {
    use super::*;

    // 消える行がある場合
    #[test]
    fn test_check_erase_row() {
        let mut game = Game::new();
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
            [1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1],
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
                [1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            ]
        );
    }

    // 消える行がない場合
    #[test]
    fn test_check_erase_row2() {
        let mut game = Game::new();
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
}
