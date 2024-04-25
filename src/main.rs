use crossterm::{
    terminal::{size, enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{poll, read, Event, KeyCode, KeyEventKind},
    cursor::{MoveTo, Hide, Show},
    style::{SetBackgroundColor, Color},
    execute,
};
use rand::prelude::*;
use glam::Vec2;
use std::io::{stdout, Result};

const FRAME_DELAY: u64 = 16;
const BASE_CHAR: char = ' ';
const FILL_CHAR: char = '■';

const THRESHOLD: f32 = 0.5;
const DENSITY: f32 = 1.05;

const VELOCITY: f32 = 1.5; // clamping breaks at velocities greater than 4.5
const _GRAVITY: f32 = 1.0;
const _HEAT: f32 = 1.0;
const _RESISTANCE: f32 = 0.8;

type Grid = Vec<Vec<bool>>;

#[derive(Default)]
struct Blob {
    coord: Vec2,
    velocity: Vec2,
}

// TODO: working edge clamping / edge detection
// TODO: gravity
// TODO: heat
// TODO: fluid resistance
// TODO: heat + fluid resistance > gravity
// TODO: heat value fluctuates slightly?
// TODO: random chance for blob to drop by ignoring heat?
// TODO: make the consts input parameters
// TODO: colors per blob!?
//TODO: background color

fn main() -> Result<()> {
    let (mut x,mut y) = get_dimensions();
    let mut blobs: Vec<Blob> = gen_blobs(&x, &y);
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    loop {
        (x,y) = get_dimensions();
        blobs = transform(blobs, x, y);
        print!("{}{}", MoveTo(0,0), Hide);
        draw(&blobs, &x, &y);

        if poll(std::time::Duration::from_millis(FRAME_DELAY))? { 
            if let Event::Key(key) = read()? {
                if key.kind == KeyEventKind::Press
                    && key.code == KeyCode::Char('q')
                {
                    break;
                }
            }
        }

    }
    execute!(stdout(), LeaveAlternateScreen)?;
    print!("{}", Show);
    disable_raw_mode()?;
    Ok(())
}

fn draw(blobs: &Vec<Blob>, x: &f32, y: &f32) {
    let mut grid: Grid = gen_grid(x, y);
    grid = metaballise(grid, blobs);

    for row in grid {
        for cell in row {
            match cell {
                true => print!("{} ",SetBackgroundColor(Color::White)),
                false => print!("{} ", SetBackgroundColor(Color::Reset)),
            }
        }
    }
}

fn gen_blobs(x: &f32, y: &f32) -> Vec<Blob> {
    let initial_blobs: u32 = (((x * y) / DENSITY).powf(1.0 / 3.0)) as u32;

    let mut rng = rand::thread_rng();
    let mut blobs: Vec<Blob> = vec![];
    for _ in 0..initial_blobs {
        let temp = Blob {
            coord: Vec2::new(rng.gen::<f32>() * *x as f32,rng.gen::<f32>() * *y as f32),
            velocity: Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)) * VELOCITY,
        };
        blobs.push(temp);
    }
    blobs
}

fn metaballise(grid: Grid, blobs: &Vec<Blob>) -> Grid {
    let mut out_grid = grid.clone();
    for i in 0..grid.len() {
        for j in 0..grid[i].len() {
            let mut value: f32 = 0.0; 
            for blob in blobs {
                value += (
                    (j as f32 - blob.coord.x).powf(2.0) + 
                    (i as f32 - blob.coord.y).powf(2.0)
                ).sqrt().recip();
            } 
            if value >= THRESHOLD {
                out_grid[i][j] = true;
            }
        }
    }
    out_grid
}

fn transform(mut blobs: Vec<Blob>,x: f32, y: f32) -> Vec<Blob> {
    for blob in &mut blobs {
        let vertical_velocity = Vec2::new(0.0, 0.0);
        let resultant_velocity = blob.velocity + vertical_velocity;

        if (blob.coord.x + resultant_velocity.x) <= 0.0 || (blob.coord.x + resultant_velocity.x) >=x {
            blob.velocity.x *= -1.0;
        } 
        else if (blob.coord.y + resultant_velocity.y) <= 0.0 || (blob.coord.y + resultant_velocity.y) >= y {
            blob.velocity.y *= -1.0;
        }
        blob.coord +=  resultant_velocity;
    }
    blobs
}

fn get_dimensions() -> (f32, f32) {
    let (x, y) = size().unwrap();
    (x as f32, y as f32)
}

fn gen_grid(x: &f32, y: &f32) -> Grid {
    vec![vec![false; *x as usize]; *y as usize]
}
