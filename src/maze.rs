use std::{ffi::OsStr, fs::File, io::BufWriter, path::Path};

use anyhow::{self, Context};
use grid::Grid;
use rand::{prelude::SliceRandom, Rng};
use rgb::{ComponentBytes, RGB8};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Maze {
    width: u32,
    height: u32,
    data: Grid<TileState>,
    visited: Grid<bool>,
}

impl Maze {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: Grid::new(width as usize, height as usize),
            visited: Grid::init(width as usize, height as usize, false),
        }
    }

    pub fn populate<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        let start_x = rng.gen_range(0..self.width) as usize;
        let start_y = rng.gen_range(0..self.height) as usize;

        // now perform a randomized depth first search
        let mut stack: Vec<(usize, usize)> = vec![(start_x, start_y)];
        // cannot be out of range
        unsafe { *self.visited.get_unchecked_mut(start_x, start_y) = true };

        while let Some(&(x, y)) = stack.last() {
            // shuffle the neighbours
            let mut neighbours = vec![
                (x, y + 1, Direction::North),
                (x + 1, y, Direction::East),
                (x, y.saturating_sub(1), Direction::South),
                (x.saturating_sub(1), y, Direction::West),
            ];
            neighbours.shuffle(rng);

            // write to the grid after we have found tiles with no neighbours
            if self.data.get(x, y).is_some() {
                // find a neighbour if one exists
                if let Some((new_x, new_y, _)) = neighbours
                    .iter()
                    .copied()
                    .find(|(x, y, d)| self.is_valid_neighbour(*x, *y, *d))
                {
                    let tile: &mut TileState = self.data.get_mut(x, y).unwrap();
                    *tile = TileState::Empty;

                    unsafe { *self.visited.get_unchecked_mut(new_x, new_y) = true };

                    stack.push((new_x, new_y));
                } else {
                    let tile: &mut TileState = self.data.get_mut(x, y).unwrap();
                    *tile = TileState::Empty;
                    stack.truncate(stack.len() - 1);
                }
            } else {
                // invalid tile
                stack.truncate(stack.len() - 1);
            }
        }

        // first define the start and end positions
        // go along from top left and bottom right.
        // on finding a transition Wall -> Empty place the start / end there
        if let Some(tile) = self.data.iter_mut().find(|tile| *tile == &TileState::Empty) {
            *tile = TileState::Start;
        }

        if let Some(tile) = self
            .data
            .iter_mut()
            .rev()
            .find(|tile| *tile == &TileState::Empty)
        {
            *tile = TileState::End;
        }
    }

    // a tile is a valid neighbour if it is surrounded by walls / or one edge
    // and it is unvisited
    fn is_valid_neighbour(&self, x: usize, y: usize, direction: Direction) -> bool {
        use Direction::*;

        let mut count = 0;

        let xs = x.saturating_sub(1);
        let ys = y.saturating_sub(1);

        // do right / top / top right
        if !matches!(self.data.get(x + 1, y), Some(TileState::Wall) | None) && !matches!(direction, West) {
            count += 1;
        }

        if !matches!(self.data.get(x, y + 1), Some(TileState::Wall) | None) && !matches!(direction, South)
        {
            count += 1;
        }

        if !matches!(self.data.get(x + 1, y + 1), Some(TileState::Wall) | None)
            && !matches!(direction, South | West)
        {
            count += 1;
        }

        if !matches!(self.data.get(xs, y), Some(TileState::Wall) | None) && !matches!(direction, East)
        {
            count += 1;
        }

        if !matches!(self.data.get(xs, y + 1), Some(TileState::Wall) | None)
            && !matches!(direction, South | East)
        {
            count += 1;
        }

        // do bottom and bottom right
        if !matches!(self.data.get(x, ys), Some(TileState::Wall) | None) && !matches!(direction, North)
        {
            count += 1;
        }

        if !matches!(self.data.get(x + 1, ys), Some(TileState::Wall) | None)
            && !matches!(direction, North | West)
        {
            count += 1;
        }

        // bottom left
        if !matches!(self.data.get(xs, ys), Some(TileState::Wall) | None)
            && !matches!(direction, North | East)
        {
            count += 1;
        }

        count == 0 && matches!(self.visited.get(x, y), Some(false))
    }

    pub fn save_to_file<S: AsRef<OsStr> + ?Sized>(&self, s: &S) -> anyhow::Result<()> {
        let path = Path::new(s);
        let file = File::create(path)?;
        let w = &mut BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width, self.height);
        encoder.set_color(png::ColorType::RGB);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .context("Failed to write the header of the PNG.")?;

        let data = self
            .data
            .iter()
            .map(|tile| tile.into())
            .collect::<Vec<RGB8>>();

        writer
            .write_image_data(data.as_bytes())
            .context("Failed to write out the image data of the maze.")?;

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum TileState {
    Wall,
    Empty,
    Start,
    End,
}

impl From<&TileState> for RGB8 {
    fn from(tilestate: &TileState) -> Self {
        use TileState::*;
        match tilestate {
            Wall => RGB8::new(0x00_u8, 0x00_u8, 0x00_u8),
            Empty => RGB8::new(0xFF_u8, 0xFF_u8, 0xFF_u8),
            Start => RGB8::new(0x00_u8, 0xFF_u8, 0x00_u8),
            End => RGB8::new(0xFF_u8, 0x00_u8, 0x00_u8),
        }
    }
}

impl Default for TileState {
    fn default() -> Self {
        TileState::Wall
    }
}
