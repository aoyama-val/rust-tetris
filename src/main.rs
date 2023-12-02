extern crate rand;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, BlendMode};
use sdl2::video::Window;
use std::time::Duration;

mod model;
use model::*;

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
    render_block(canvas, &game.block, LEFT_WALL_X + game.block.x, game.block.y)?;

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
    let block_color = get_color(block.color + 2);
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
