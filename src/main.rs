mod maze;

use std::env;
use std::ffi::OsString;
use std::num::ParseIntError;
use std::str::FromStr;

use crate::maze::generator::MazeGenerator;
use crate::maze::generator::{growing_tree::GrowingTreeGenerator, kruskal::Kruskal};
use crate::maze::paint::*;
use crate::maze::regular::RectilinearMaze;

pub fn make_svg_maze(x_size: usize, y_size: usize, seed: u64) -> String {
    let maze: RectilinearMaze = GrowingTreeGenerator::new((x_size, y_size), seed).generate();
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

fn usage() {
    println!(
        "\
Usage:

    maze [ x [ y [ seed ] ] ]

  Where x defaults to 15, y defaults to x, and seed defaults to
  a random unsigned 64bit integer. The output file will be called
  maze-{{x}}-{{y}}-{{seed}}.svg."
    );
    std::process::exit(128);
}

use thiserror::Error;

#[derive(Debug, Error)]
enum MazeError {
    #[error("Error parsing utf8 string.")]
    ErrorParsingUtf8,
    #[error("Not a number.")]
    NotANumber(#[from] ParseIntError),
}

fn os_string_to_number<T>(s: &OsString) -> Result<T, MazeError>
where
    T: FromStr<Err = ParseIntError>,
{
    str::parse::<T>(s.to_str().ok_or(MazeError::ErrorParsingUtf8)?).map_err(MazeError::NotANumber)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args_os().collect::<Vec<_>>();
    use crate::maze::generator::*;
    use crate::maze::paint::*;

    let x_size = args.get(1).map(os_string_to_number).unwrap_or(Ok(15))?;
    let y_size = args.get(2).map(os_string_to_number).unwrap_or(Ok(x_size))?;

    let seed = args
        .get(3)
        .map(os_string_to_number)
        .unwrap_or(Ok(fastrand::u64(..)))?;

    // let maze = GrowingTreeGenerator::new((x_size, y_size), seed).generate();
    let maze = Kruskal::new((x_size, y_size), seed).generate();

    PlottersSvgFileWriter::new(format!("maze-{x_size}-{y_size}-{seed}.svg"), 40, 4).write_maze(
        &maze,
        [DrawingInstructions::DrawMaze(
            WebColour::from_string("000000").unwrap(),
        )],
    )?;

    Ok(())
}
