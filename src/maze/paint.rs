pub mod rect;
pub mod sigma;
pub mod svg;
pub mod theta;

use std::cmp::max;

use super::interface::{Maze, MazeRenderer, Metadata, Solution};

use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct WebColour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Error, Debug)]
pub enum ColourReadError {
    #[error("Illegal character")]
    IllegalFormat(#[from] hex::FromHexError),
    #[error("Illegal length")]
    IllegalLength(usize),
}

impl WebColour {
    pub fn from_string(input: &str) -> Result<Self, ColourReadError> {
        let u8v = hex::decode(input)?;
        match u8v.len() {
            3 => Ok(WebColour {
                r: u8v[0],
                g: u8v[1],
                b: u8v[2],
                a: u8::MAX,
            }),
            4 => Ok(WebColour {
                r: u8v[0],
                g: u8v[1],
                b: u8v[2],
                a: u8v[3],
            }),
            l => Err(ColourReadError::IllegalLength(l)),
        }
    }

    pub fn to_web_string(self) -> String {
        format!(
            "rgba({},{},{},{})",
            self.r,
            self.g,
            self.b,
            self.a as f64 / 255.0
        )
    }

    pub fn blend(&self, f: f64) -> Self {
        WebColour {
            r: (self.r as f64 * f) as u8,
            g: (self.g as f64 * f) as u8,
            b: (self.b as f64 * f) as u8,
            a: self.a,
        }
    }

    pub fn add(&self, other: &Self) -> Self {
        WebColour {
            r: self.r.saturating_add(other.r),
            g: self.g.saturating_add(other.g),
            b: self.b.saturating_add(other.b),
            a: max(self.a, other.a),
        }
    }
}

pub struct CellSize(usize);

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DrawingInstructions {
    DrawMaze(WebColour),
    ShowSolution(WebColour),
    StainMaze((WebColour, WebColour)),
}

impl DrawingInstructions {
    pub fn run<M, R>(&self, renderer: &mut R)
    where
        M: crate::maze::interface::Maze,
        R: MazeRenderer<M>,
    {
        use DrawingInstructions::*;
        match self {
            DrawMaze(_) => {}
            ShowSolution(colour) => renderer.solve(*colour),
            StainMaze(gradient) => renderer.stain(*gradient),
        }
    }
}

#[derive(Debug)]
pub struct Gradient<'a, M: Maze> {
    max_distance: usize,
    distances: &'a [usize],
    maze: &'a M,
    a: WebColour,
    b: WebColour,
}

impl<'a, M: Maze> Gradient<'a, M> {
    fn new((a, b): (WebColour, WebColour), maze: &'a M, solution: &'a Solution<M::Idx>) -> Self {
        Self {
            a,
            b,
            maze,
            distances: &solution.distances,
            max_distance: *solution.distances.iter().max().unwrap(),
        }
    }

    fn compute(&self, index: &M::Idx) -> WebColour {
        let intensity = (self.max_distance - self.distances[self.maze.get_index(*index)]) as f64
            / self.max_distance as f64;
        let inverse = 1.0 - intensity;
        self.a.blend(intensity).add(&self.b.blend(inverse))
    }
}

#[derive(Debug)]
pub struct RenderedMaze {
    document: ::svg::Document,
}

impl RenderedMaze {
    pub fn new(document: ::svg::Document) -> Self {
        Self { document }
    }

    pub fn append_metadata(&mut self, metadata: &Metadata) {
        // get view box

        // append enough space for metadata

        // apend metadata
    }

    pub fn to_string(&self) -> String {
        let mut strbuf: Vec<u8> = Vec::new();
        ::svg::write(&mut strbuf, &self.document).unwrap();
        String::from_utf8(strbuf).unwrap()
    }
}
