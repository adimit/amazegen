#![allow(mixed_script_confusables)]
mod maze;

use std::num::ParseIntError;

use crate::maze::generator::growing_tree::GrowingTreeGenerator;
use crate::maze::generator::MazeGenerator;
use crate::maze::paint::*;
use crate::maze::polar::test_maze;
use crate::maze::regular::RectilinearMaze;

pub fn make_svg_maze(x_size: usize, y_size: usize, seed: u64) -> String {
    let maze: RectilinearMaze =
        GrowingTreeGenerator::<(usize, usize)>::new((x_size, y_size), seed).generate();
    let mut str = String::new();
    PlottersSvgStringWriter::new(&mut str, 40, 4)
        .write_maze(
            &maze,
            [DrawingInstructions::DrawMaze(
                WebColour::from_string("ffffff").unwrap(),
            )],
        )
        .unwrap();
    str
}

use thiserror::Error;

#[derive(Debug, Error)]
enum MazeError {
    #[error("Not a number.")]
    NotANumber(#[from] ParseIntError),
}

fn main() -> Result<(), ()> {
    test_maze();
    Ok(())
}
