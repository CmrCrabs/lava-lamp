use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Stylize, Terminal},
    widgets::Paragraph,
};
use termion::terminal_size;
use rand::prelude::*;
use glam::Vec2;
use std::io::{stdout, Result};

const FRAME_DELAY: u64 = 16;
const BASE_CHAR: char = ' ';
const FILL_CHAR: char = '■';

const THRESHOLD: f32 = 0.5;
const DENSITY: f32 = 1.25;

const VELOCITY: f32 = 0.5;
const GRAVITY: f32 = 1.0;
const HEAT: f32 = 1.0;
const RESISTANCE: f32 = 0.8;

type Grid = Vec<Vec<char>>;

#[derive(Default)]
struct Blob {
    x: f32,
    y: f32,
    velocity: Vec2,
}

// TODO: make the consts input parameters
// gravity
// heat
// fluid resistance
// collisions
// mass per blob (value for centre)
// scale collisions based on per blob mass
// TODO: colors per blob!?
// TODO: working edge clamping / edge detection

fn main() -> Result<()> {
    let (mut x,mut y) = get_dimensions();
    let mut blobs: Vec<Blob> = gen_blobs(&x, &y);

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut text;

    loop {
        (x,y) = get_dimensions();
        blobs = transform(blobs, x, y);
        text = draw(&blobs, &x, &y);
        terminal.draw(|frame| {

            let area = frame.size();
            frame.render_widget(
                Paragraph::new(text)
                    .white(),
                    //.on_white(),
                area,
            );
        })?;

        if event::poll(std::time::Duration::from_millis(FRAME_DELAY))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press
                    && key.code == KeyCode::Char('q')
                {
                    break;
                }
            }
        }
    } 

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn draw(blobs: &Vec<Blob>, x: &f32, y: &f32) -> String {
    let mut grid: Grid = gen_grid(x, y);
    grid = metaballise(grid, blobs);

    let output_grid = grid.into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .rev()
        .collect::<Vec<String>>()
        .join("\n");

    output_grid
}

fn get_dimensions() -> (f32, f32) {
    let (x, y) = terminal_size().unwrap();
    (x as f32, y as f32)
}

fn gen_grid(x: &f32, y: &f32) -> Grid {
    vec![vec![BASE_CHAR; *x as usize]; *y as usize]
}

fn gen_blobs(x: &f32, y: &f32) -> Vec<Blob> {
    let initial_blobs: u32 = ((1.0 / DENSITY * x * y).powf(1.0 / 3.0)) as u32;

    let mut rng = rand::thread_rng();
    let mut blobs: Vec<Blob> = vec![];
    for _ in 0..initial_blobs {
        let mut temp: Blob = Default::default();
        temp.x = rng.gen::<f32>() * *x as f32;
        temp.y = rng.gen::<f32>() * *y as f32;
        temp.velocity = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)) * VELOCITY;
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
                    (j as f32 - blob.x).powf(2.0) + 
                    (i as f32 - blob.y).powf(2.0)
                ).sqrt().recip();
            } 
            if value >= THRESHOLD {
                out_grid[i][j] = FILL_CHAR;
            }
        }
    }
    out_grid
}

fn transform(mut blobs: Vec<Blob>,x: f32, y: f32) -> Vec<Blob> {
    for blob in &mut blobs {
        let vertical_velocity = Vec2::new(0.0, 0.0);
        let resultant_velocity = blob.velocity + vertical_velocity;

        if blob.x <= 0.0 || blob.x >=x {
            blob.velocity.x *= -1.0;
        } 
        else if blob.y <= 0.0 || blob.y >= y {
            blob.velocity.y *= -1.0;
        }
        blob.x +=  resultant_velocity.x;
        blob.y += resultant_velocity.y;

    }
    blobs
}

