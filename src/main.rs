mod maze;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use maze::generator::*;
    use maze::paint::*;
    use maze::*;
    let x_size = 15;
    let y_size = 15;

    let maze: Maze = jarník(x_size, y_size, 10);

    PlottersBitmapWriter::new("./test.png".into(), 40, 4).write_maze(&maze)?;
    PlottersSvgFileWriter::new("./test.svg".into(), 40, 4).write_maze(&maze)?;

    let mut str = String::new();
    PlottersSvgStringWriter::new(&mut str, 40, 4).write_maze(&maze)?;
    println!("{str}");

    Ok(())
}
