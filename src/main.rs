use crossterm::{
    terminal::{size, enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{poll, read, Event, KeyCode, KeyEventKind},
    cursor::{MoveTo, Hide, Show},
    style::{SetBackgroundColor, Color},
    execute,
};
use std::io::{stdout, Result};
use rand::prelude::*;
use glam::Vec2;

const FRAME_DELAY: u64 = 5;
const THRESHOLD: f32 = 0.8;
const DENSITY: f32 = 1.25;

const VELOCITY: f32 = 1.5; // clamping breaks at velocities greater than 4.5
const FLUCT: f32 = 0.2;
const COLOR: (f32, f32, f32) = (214.0, 15.0, 205.0);

type Grid = Vec<Vec<Color>>;

#[derive(Default)]
struct Blob {
    coord: Vec2,
    velocity: Vec2,
}

// TODO: smooth the transition from falling to not falling
// TOOD: make it so can adjust settle height
// TODO: make the consts input parameters
// TODO: colors per blob!? // do the thing in temptmemtepm
// TODO: vary threshold by velocity to change it
// TODO: Blob shading?
//
// TODO: make color scale based on distance to bottom // hueshifted slightly to be warmer
// TODO: make background a hueshifted version of color to be darker and complementary?

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
    let mut grid: Grid = vec![vec![Color::Reset; *x as usize]; *y as usize];
    grid = metaballise(grid, blobs);

    for row in grid.into_iter().rev() {
        for cell in row {
            print!("{} ", SetBackgroundColor(cell));
        }
    }
}

fn gen_blobs(x: &f32, y: &f32) -> Vec<Blob> {
    let initial_blobs: u32 = (((x * y) / DENSITY).powf(1.2 / 3.0)) as u32;

    let mut rng = rand::thread_rng();
    let mut blobs: Vec<Blob> = vec![];
    for _ in 0..initial_blobs {
        blobs.push( Blob {
            coord: Vec2::new(rng.gen_range(0.0..1.0) * *x as f32,rng.gen_range(0.0..1.0) * *y as f32),
            velocity: Vec2::new(rng.gen_range(-0.5..0.5), rng.gen_range(-0.3..0.3)) * VELOCITY,
        });
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
                if value >= 1.0 { value = 1.0; }
                out_grid[i][j] = Color::Rgb { 
                    r: ((COLOR.0 * value) as u8), 
                    g: ((COLOR.1 * value) as u8), 
                    b: ((COLOR.2 * value) as u8)
                };
            }
            // add fluid color by hue shifting 
            // scale color based on distance to bottom
        }
    }
    out_grid
}

fn transform(mut blobs: Vec<Blob>,x: f32, y: f32) -> Vec<Blob> {
    for blob in &mut blobs {
        let mut rng = rand::thread_rng();
        let vertical_velocity: Vec2 = Vec2::new(0.0,0.0);

        let mut resultant_velocity = blob.velocity + vertical_velocity;
        if (blob.coord.x + resultant_velocity.x) <= 0.0 || (blob.coord.x + resultant_velocity.x) >=x {
            blob.velocity.x *= -1.0;
            resultant_velocity.x *= -1.0;
        } 
        if (blob.coord.y + resultant_velocity.y) <= 0.0 {
            blob.velocity.y *= -1.0;
            resultant_velocity.y *= -1.0;
        }
        if (blob.coord.y + resultant_velocity.y) >= y {
            blob.velocity.y *= -1.0;
            resultant_velocity.y *= -1.0;
        }

        blob.coord += resultant_velocity * rng.gen_range(1.0 - FLUCT..1.0 + FLUCT);
    }
    blobs
}

fn get_dimensions() -> (f32, f32) {
    let (x, y) = size().unwrap();
    (x as f32, y as f32)
}

//i have a blob
//blob has upwards velocity such that the further you get from line y = 0 the higher the velocity is
//blob has random chance of falling
//when blob is falling its vertical velocity -= k
//such that it reaches a point just above the bottom where 1/y bigger than k and it starts rising
//and then once it starts rising again k is no longer added and cyrcle repeats
