mod maze;

use maze::generator::*;
use maze::paint::*;
use maze::*;

pub fn make_svg_maze(x_size: usize, y_size: usize, seed: u64) -> String {
    let maze: Maze = jarník(x_size, y_size, seed);
    let mut str = String::new();
    PlottersSvgStringWriter::new(&mut str, 40, 4)
        .write_maze(&maze)
        .unwrap();
    str
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use maze::generator::*;
    use maze::paint::*;
    use maze::*;
    let x_size = 15;
    let y_size = 15;

    let maze: Maze = jarník(x_size, y_size, 10);

    PlottersSvgFileWriter::new("./test.svg".into(), 40, 4).write_maze(&maze)?;

    let seed: u64 = rand::prelude::random();
    let s = crate::make_svg_maze(x_size, y_size, seed);
    println!("{seed}\n{s}");

    Ok(())
}
