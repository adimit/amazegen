#![allow(mixed_script_confusables)]
use std::num::ParseIntError;

use amazegen::maze::shape::regular::test_maze;
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
