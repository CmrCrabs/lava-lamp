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
use std::io::{stdout, Result};

const BASE_CHAR: char = ' ';
const FILL_CHAR: char = '■';
const INITIAL_BLOBS: usize = 12;
const THRESHOLD: f32 = 0.5;


type Grid = Vec<Vec<char>>;
#[derive(Default)]
struct Blob {
    x: f32,
    y: f32,
    velocity: f32,
}

fn main() -> Result<()> {
    let (x,y) = get_dimensions();
    let mut blobs: [Blob; INITIAL_BLOBS] = gen_blobs(x, y);

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    loop {
        let text = draw(&blobs, x, y);
        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(
                Paragraph::new(text)
                    .white(),
                    //.on_black(),
                area,
            );
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
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

fn draw(blobs: &[Blob; INITIAL_BLOBS], x: f32, y: f32) -> String {
    let mut grid: Grid = gen_grid(x, y);
    grid = metaballise(grid, blobs);

    let output_grid = grid.into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<String>>()
        .join("\n");

    output_grid
}

fn get_dimensions() -> (f32, f32) {
    let (x, y) = terminal_size().unwrap();
    (x as f32, y as f32)
}

fn gen_grid(x: f32, y: f32) -> Grid {
    vec![vec![BASE_CHAR; x as usize]; y as usize]
}

fn gen_blobs(x: f32, y: f32) -> [Blob; INITIAL_BLOBS] {
    let mut rng = rand::thread_rng();
    let mut blobs: [Blob; INITIAL_BLOBS] = Default::default();
    for i in 0..blobs.len() {
        blobs[i].x = rng.gen::<f32>() * x as f32;
        blobs[i].y = rng.gen::<f32>() * y as f32;
    }
    blobs
}

fn metaballise(grid: Grid, blobs: &[Blob; INITIAL_BLOBS]) -> Grid {
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

fn heat_lamp(blobs: [(f32, f32); INITIAL_BLOBS]) -> [(f32, f32); INITIAL_BLOBS] {
   blobs 
}
