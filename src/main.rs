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
use rgb_hsv::{self, hsv_to_rgb, rgb_to_hsv};

const FRAME_DELAY: u64 = 5;
const THRESHOLD: f32 = 0.8;
const DENSITY: f32 = 1.35;
type Grid = Vec<Vec<Color>>;

#[derive(Default)]
struct Blob {
    coord: Vec2,
    velocity: Vec2,
    falling: bool,
}

struct Params {
    velocity: f32,
    fluct: f32,
    color: (f32, f32, f32),
    background_enable: bool,
    epilepsy: bool,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            velocity: 5.5,
            fluct: 0.2,
            color: (255.0,255.0,255.0),
            background_enable: true,
            epilepsy: true,
        }
    }
}

// TODO: make the consts input parameters

fn main() -> Result<()> {
    let mut params: Params = Default::default();
    params.color = gen_color();
    let (mut x,mut y) = get_dimensions();
    let mut blobs: Vec<Blob> = gen_blobs(&x, &y, &params);

    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let mut i = 0;
    loop {
        i+= 1;
        (x,y) = get_dimensions();
        blobs = transform(blobs, x, y, &params);
        print!("{}{}", MoveTo(0,0), Hide);
        draw(&blobs, &x, &y, &params);
        if i % 6 == 0 && params.epilepsy { params.color = gen_color(); }

        if poll(std::time::Duration::from_millis(FRAME_DELAY))? { 
            if let Event::Key(key) = read()? {
                if key.kind == KeyEventKind::Press
                    && key.code == KeyCode::Char('q')
                {
                    break;
                }
                else if key.kind == KeyEventKind::Press
                    && key.code == KeyCode::Char('r')
                {
                    params.color = gen_color();
                }
            }
        }
    }
    execute!(stdout(), LeaveAlternateScreen)?;
    print!("{}", Show);
    disable_raw_mode()?;
    Ok(())
}

fn draw(blobs: &Vec<Blob>, x: &f32, y: &f32, params: &Params) {
    let grid = metaballise(blobs, x, y, params);

    for row in grid.into_iter().rev() {
        for cell in row {
            print!("{} ", SetBackgroundColor(cell));
        }
    }
}

fn gen_blobs(x: &f32, y: &f32, params: &Params) -> Vec<Blob> {
    let initial_blobs: u32 = (((x * y) / DENSITY).powf(1.2 / 3.0)) as u32;

    let mut rng = rand::thread_rng();
    let mut blobs: Vec<Blob> = vec![];
    for _ in 0..initial_blobs {
        blobs.push( Blob {
            coord: Vec2::new(rng.gen_range(0.0..1.0) * *x,rng.gen_range(0.0..1.0) * *y),
            velocity: Vec2::new(rng.gen_range(-0.5..0.5), rng.gen_range(-0.3..0.3)) * params.velocity,
            falling: false,
        });
    }
    blobs
}

fn metaballise(blobs: &Vec<Blob>,x: &f32, y: &f32, params: &Params) -> Grid {
    let mut grid: Grid = vec![vec![Color::Reset; *x as usize]; *y as usize];

    for i in 0..grid.len() {
        for j in 0..grid[i].len() {
            let mut value: f32 = 0.0; 
            let color: (f32, f32, f32) = params.color;

            for blob in blobs {
                value += (
                    (j as f32 - blob.coord.x).powf(2.0) + 
                    (i as f32 - blob.coord.y).powf(2.0)
                ).sqrt().recip();
            } 
            if value >= THRESHOLD {
                if value >= 1.0 { value = 1.0; }
                let mut hsv = rgb_to_hsv(color);
                hsv = (hsv.0 - (0.28 * value), hsv.1 * value, hsv.2 * value);
                let rgb = hsv_to_rgb((hsv.0 - (0.2 * hsv.0 * linear_interpolation(i as f32, grid.len() as f32 + 0.5)), hsv.1, hsv.2));
                grid[i][j] = Color::Rgb { r: (rgb.0 as u8), g: (rgb.1 as u8), b: (rgb.2 as u8) };
            } else if params.background_enable {
                let mut rgb = (color.0 * value, color.1 * value, color.2 * value);
                let hsv = rgb_to_hsv(rgb);
                rgb = hsv_to_rgb((hsv.0 - (0.5 * hsv.0 * linear_interpolation(i as f32, grid.len() as f32 + 0.5)), hsv.1, hsv.2));
                grid[i][j] = Color::Rgb { r: ((0.2 * (255.0 - rgb.0)) as u8), g: ((0.2 * (255.0 - rgb.1)) as u8), b: ((0.2 * (255.0 - rgb.2)) as u8) };
            }
        }
    }
    grid
}

fn transform(mut blobs: Vec<Blob>,x: f32, y: f32, params: &Params) -> Vec<Blob> {
    for blob in &mut blobs {
        let mut rng = rand::thread_rng();
        let mut vertical_velocity: Vec2 = Vec2::new(0.0,params.velocity * 1.2 * linear_interpolation(blob.coord.y, y));
        if !blob.falling && rng.gen_range(0.0..1.0) > 0.98 && blob.coord.y > (7.0 * y / 10.0) {
            blob.falling = true;
        }
        if blob.falling {
            vertical_velocity.y -= params.velocity * (1.2 - linear_interpolation(blob.coord.y, y)); 
        }

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

        blob.coord += resultant_velocity * rng.gen_range(1.0 - params.fluct..1.0 + params.fluct);
        if blob.falling && resultant_velocity.y >= (-0.001 + rng.gen_range(0.0..0.005)){ blob.falling = false;};
    }
    blobs
}

fn get_dimensions() -> (f32, f32) {
    let (x, y) = size().unwrap();
    (x as f32, y as f32)
}

fn linear_interpolation(j: f32, y: f32) -> f32 {
    if j == 0.0 { return 1.0 - 0.0000001 }
    let normalized_j = j / y;
    let result = if j == y {
        0.0
    } else {
        (1.0 - normalized_j) * 0.0 + normalized_j * 1.0
    };
    1.0 - result
}

fn gen_color() -> (f32, f32, f32) {
    let mut rng = rand::thread_rng();
    let mut rgb: [f32; 3] = [
        rng.gen_range(20.0..50.0),
        rng.gen_range(50.0..150.0),
        rng.gen_range(150.0..255.0),
    ];
    for i in (1..rgb.len()).rev() {
        let j = rng.gen_range(0..=i);
        rgb.swap(i, j);
    }

    (rgb[0],rgb[1],rgb[2])
}
