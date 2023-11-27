#![allow(mixed_script_confusables)]
use std::num::ParseIntError;

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

use amazegen::maze::{
    generator::{growing_tree::GrowingTreeGenerator, MazeGenerator},
    paint::{DrawingInstructions, MazeFileWriter, PlottersSvgStringWriter, WebColour},
    regular::test_maze,
    regular::RectilinearMaze,
};
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
