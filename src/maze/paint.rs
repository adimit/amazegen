pub mod regular;

use std::cell::RefCell;

use super::{regular::RectilinearMaze, solver::Solver, Maze, Node};

use plotters::{
    coord::Shift,
    prelude::{DrawingArea, SVGBackend},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MazePaintError {
    #[error("Error drawing maze")]
    Paint,
    #[error("Error saving picture")]
    Save(#[from] std::io::Error),
}

pub trait MazeFileWriter<C: Node, M: Maze<NodeType = C>> {
    fn write_maze<I>(&mut self, maze: &M, instructions: I) -> Result<(), MazePaintError>
    where
        I: IntoIterator<Item = DrawingInstructions>;
}

#[derive(Debug)]
pub struct PlottersSvgFileWriter {
    border_size: usize,
    cell_size: usize,
    file_name: String,
}

impl PlottersSvgFileWriter {
    pub fn new(file_name: String, cell_size: usize, border_size: usize) -> Self {
        Self {
            border_size,
            cell_size,
            file_name,
        }
    }
}

#[derive(Debug)]
pub struct PlottersSvgStringWriter<'a> {
    border_size: usize,
    cell_size: usize,
    into_string: &'a mut String,
}

impl<'a> PlottersSvgStringWriter<'a> {
    pub fn new(buffer: &'a mut String, cell_size: usize, border_size: usize) -> Self {
        Self {
            cell_size,
            border_size,
            into_string: buffer,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct WebColour {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
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
                a: u8::max_value(),
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
}

impl From<WebColour> for plotters::style::RGBAColor {
    fn from(val: WebColour) -> Self {
        plotters::style::RGBAColor(val.r, val.g, val.b, val.a as f64 / 255.0)
    }
}

pub struct CellSize(usize);
pub struct BorderWidth(usize);

#[derive(Debug, PartialEq, Eq)]
pub enum DrawingInstructions {
    DrawMaze(WebColour),
    ShowSolution(WebColour),
    StainMaze((WebColour, WebColour)),
}

impl PartialOrd for DrawingInstructions {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DrawingInstructions {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::*;
        use DrawingInstructions::*;
        match (self, other) {
            (StainMaze(_), ShowSolution(_)) => Less,
            (ShowSolution(_), StainMaze(_)) => Greater,
            (StainMaze(_), DrawMaze(_)) => Less,
            (DrawMaze(_), StainMaze(_)) => Greater,
            (DrawMaze(_), ShowSolution(_)) => Less,
            (ShowSolution(_), DrawMaze(_)) => Greater,
            _ => Equal,
        }
    }
}

struct Visuals<'a, M: Maze> {
    pic: DrawingArea<SVGBackend<'a>, Shift>,
    maze: &'a M,
    border_width: BorderWidth,
    cell_size: CellSize,
    solver: RefCell<Option<Solver<'a, M>>>,
}

impl DrawingInstructions {
    fn execute(&self, p: &Visuals<RectilinearMaze>) -> Result<(), MazePaintError> {
        use DrawingInstructions::*;
        match self {
            DrawMaze(colour) => p.render_maze(colour),
            ShowSolution(colour) => p.solve_maze(*colour),
            StainMaze(colours) => p.stain_maze(*colours),
        }
    }
}
