use crate::maze::solver::{dijkstra, solve};

use super::Maze;
use itertools::Itertools;
use plotters::{
    coord::Shift,
    prelude::{
        DrawingArea, DrawingAreaErrorKind, DrawingBackend, IntoDrawingArea, PathElement, Rectangle,
        SVGBackend,
    },
    style::{Color, IntoTextStyle, RGBAColor, RGBColor, BLACK, RED, WHITE},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MazePaintError {
    #[error("Error drawing maze")]
    Paint,
    #[error("Error saving picture")]
    Save(#[from] std::io::Error),
}

pub trait MazeFileWriter {
    fn write_maze(&mut self, maze: &Maze) -> Result<(), MazePaintError>;
}

#[derive(Debug)]
pub struct PlottersSvgFileWriter {
    border_size: usize,
    cell_size: usize,
    file_name: String,
    colour: WebColour,
}

impl PlottersSvgFileWriter {
    pub fn new(file_name: String, cell_size: usize, border_size: usize, colour: WebColour) -> Self {
        Self {
            border_size,
            cell_size,
            file_name,
            colour,
        }
    }
}

fn get_wall_runs(maze: &Maze, direction: super::Direction) -> Vec<Vec<(usize, usize)>> {
    use super::Direction::*;
    match direction {
        Up | Down => (0..maze.extents.1)
            .map(move |y| get_wall_run(maze, y, direction))
            .collect::<Vec<_>>(),
        Left | Right => (0..maze.extents.0)
            .map(move |x| get_wall_run(maze, x, direction))
            .collect::<Vec<_>>(),
    }
}

fn get_wall_run(maze: &Maze, line: usize, direction: super::Direction) -> Vec<(usize, usize)> {
    use super::Direction::*;

    // The match arms would have an incompatible closure type, which is
    // why we duplicate the code here. There might be a better option,
    // but I'm not aware of it.
    match direction {
        Up | Down => (0..maze.extents.0)
            .group_by(move |x| maze.has_wall((*x, line), direction))
            .into_iter()
            .filter(|(key, _)| *key)
            .map(|(_, group)| {
                let run = group.collect::<Vec<_>>();
                (*run.first().unwrap(), *run.last().unwrap())
            })
            .collect::<Vec<_>>(),
        Left | Right => (0..maze.extents.1)
            .group_by(move |y| maze.has_wall((line, *y), direction))
            .into_iter()
            .filter(|(key, _)| *key)
            .map(|(_, group)| {
                let run = group.collect::<Vec<_>>();
                (*run.first().unwrap(), *run.last().unwrap())
            })
            .collect::<Vec<_>>(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::maze::Direction::*;

    #[test]
    fn deserialise_web_colour_from_triplet() {
        assert_eq!(
            WebColour::from_string("00ffa0").unwrap(),
            WebColour {
                r: 0,
                g: 255,
                b: 160,
                a: 255
            }
        );
    }

    #[test]
    fn deserialise_web_colour_from_quadruplet() {
        assert_eq!(
            WebColour::from_string("00ffa0b0").unwrap(),
            WebColour {
                r: 0,
                g: 255,
                b: 160,
                a: 176
            }
        );
    }

    #[test]
    fn get_wall_runs_should_recognize_runs() {
        let mut maze = Maze::new((10, 2));
        maze.move_from((1, 0), Down);
        maze.move_from((5, 0), Down);

        assert_eq!(
            get_wall_runs(&maze, Up),
            [vec![(0, 9)], vec![(0, 0), (2, 4), (6, 9)]]
        );
    }

    #[test]
    fn get_wall_runs_works_vertically() {
        let mut maze = Maze::new((2, 10));
        maze.move_from((0, 2), Right);
        maze.move_from((0, 5), Right);

        assert_eq!(
            get_wall_runs(&maze, Left),
            [vec![(0, 9)], vec![(0, 1), (3, 4), (6, 9)]]
        );
    }
}

#[derive(Debug)]
pub struct PlottersSvgStringWriter<'a> {
    border_size: usize,
    cell_size: usize,
    into_string: &'a mut String,
    colour: WebColour,
}

impl<'a> PlottersSvgStringWriter<'a> {
    pub fn new(
        buffer: &'a mut String,
        cell_size: usize,
        border_size: usize,
        colour: WebColour,
    ) -> Self {
        Self {
            cell_size,
            border_size,
            into_string: buffer,
            colour,
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

fn render_maze<'a>(
    pic: &DrawingArea<SVGBackend, Shift>,
    maze: &'a Maze,
    border_width: BorderWidth,
    cell: CellSize,
    colour: &WebColour,
) -> Result<(), MazePaintError> {
    use super::Direction::*;
    let cell_size = cell.0 as i32;
    let border = border_width.0 as i32;

    let mut h = get_wall_runs(maze, Up);
    h.push(get_wall_run(maze, maze.extents.0 - 1, Down));
    let mut v = get_wall_runs(maze, Left);
    v.push(get_wall_run(maze, maze.extents.1 - 1, Right));
    let style = {
        let svg_colour: RGBAColor = (*colour).into();
        svg_colour.stroke_width((border * 2).try_into().unwrap())
    };

    for (y, xs) in h.iter().enumerate() {
        let y_offset: i32 = y as i32 * cell_size;
        for (start, end) in xs {
            let x0: i32 = *start as i32 * cell_size;
            let xe: i32 = (*end as i32 + 1) * cell_size;
            pic.draw(&PathElement::new(
                [
                    (x0, y_offset + border),
                    (xe + 2 * border, y_offset + border),
                ],
                style,
            ))
            .unwrap();
        }
    }

    for (x, ys) in v.iter().enumerate() {
        let x_offset: i32 = x as i32 * cell_size;
        for (start, end) in ys {
            let y0: i32 = *start as i32 * cell_size;
            let ye: i32 = (*end as i32 + 1) * cell_size;
            pic.draw(&PathElement::new(
                [
                    (x_offset + border, (y0)),
                    (x_offset + border, (ye + 2 * border)),
                ],
                style,
            ))
            .unwrap();
        }
    }

    Ok(())
}

fn stain_maze(
    pic: &DrawingArea<SVGBackend, Shift>,
    maze: &Maze,
    border: BorderWidth,
    cell_size: CellSize,
    colours: (WebColour, WebColour),
) -> Result<(), MazePaintError> {
    let cell_size = cell_size.0 as i32;
    let border = border.0 as i32;
    let distances = dijkstra(maze);
    let max_distance: usize = *distances
        .iter()
        .map(move |dim| dim.iter().max().unwrap_or(&0))
        .max()
        .unwrap_or(&0);

    fn get_colour(absolute: u8, fraction: f64) -> u8 {
        (absolute as f64 * fraction) as u8
    }

    for (x, y) in (0..maze.extents.0).cartesian_product(0..maze.extents.1) {
        let x0: i32 = cell_size * x as i32 + border;
        let y0: i32 = cell_size * y as i32 + border;
        let x1: i32 = x0 + cell_size + 1;
        let y1: i32 = y0 + cell_size + 1;
        let intensity = (max_distance - distances[x][y]) as f64 / max_distance as f64;
        let inverse = 1.0 - intensity;
        let c1: (u8, u8, u8) = (185, 50, 125);
        let c2: (u8, u8, u8) = (255, 220, 128);
        let style = RGBColor(
            get_colour(colours.0.r, intensity) + get_colour(colours.1.r, inverse),
            get_colour(colours.0.g, intensity) + get_colour(colours.1.g, inverse),
            get_colour(colours.0.b, intensity) + get_colour(colours.1.b, inverse),
        )
        .filled();
        pic.draw(&Rectangle::new([(x0, y0), (x1, y1)], style))
            .unwrap();
    }
    Ok(())
}

fn solve_maze(
    pic: &DrawingArea<SVGBackend, Shift>,
    maze: &Maze,
    border: BorderWidth,
    cell_size: CellSize,
    colour: WebColour,
) -> Result<(), MazePaintError> {
    {
        let border = border.0 as i32;
        let cell_size = cell_size.0 as i32;
        let path_offset = border + (cell_size / 2);
        let to_coord = |a| cell_size * a as i32 + path_offset;
        let entrance = maze.get_entrance();
        let exit = maze.get_exit();
        let mut solution: Vec<(i32, i32)> =
            vec![(to_coord(exit.0), to_coord(exit.1) + path_offset)];
        solution.extend(solve(maze).iter().map(|(x, y)| {
            let x0: i32 = to_coord(*x);
            let y0: i32 = to_coord(*y);
            (x0, y0)
        }));
        solution.push((to_coord(entrance.0), 0));
        pic.draw(&PathElement::new(
            solution,
            Into::<RGBAColor>::into(colour).stroke_width(border as u32 * 4),
        ))
        .unwrap();
        Ok(())
    }
}

pub struct CellSize(usize);
pub struct BorderWidth(usize);

enum DrawingInstructions {
    DrawMaze(WebColour),
    ShowSolution(WebColour),
    StainMaze((WebColour, WebColour)),
}

fn write_to_backend<'a, F, I>(
    make_drawing_area: F,
    maze: &Maze,
    cell_size: usize,
    border_size: usize,
    instructions: I,
) -> Result<(), MazePaintError>
where
    F: FnOnce((u32, u32)) -> DrawingArea<SVGBackend<'a>, Shift>,
    I: IntoIterator<Item = DrawingInstructions>,
{
    let border = border_size as u32;
    let x = cell_size as u32 * maze.extents.0 as u32 + border * 2;
    let y = cell_size as u32 * maze.extents.1 as u32 + border * 2;

    let pic = make_drawing_area((x, y));

    for instruction in instructions {
        instruction
            .execute(
                &pic,
                maze,
                BorderWidth(border as usize),
                CellSize(cell_size),
            )
            .unwrap();
    }

    pic.present().unwrap();
    Ok(())
}

impl DrawingInstructions {
    fn execute(
        &self,
        pic: &DrawingArea<SVGBackend, Shift>,
        maze: &Maze,
        border_width: BorderWidth,
        cell_size: CellSize,
    ) -> Result<(), MazePaintError> {
        use DrawingInstructions::*;
        match self {
            DrawMaze(colour) => render_maze(pic, maze, border_width, cell_size, colour),
            ShowSolution(colour) => solve_maze(pic, maze, border_width, cell_size, *colour),
            StainMaze(colours) => stain_maze(pic, maze, border_width, cell_size, *colours),
        }
    }
}

impl MazeFileWriter for PlottersSvgFileWriter {
    fn write_maze(&mut self, maze: &Maze) -> Result<(), MazePaintError> {
        write_to_backend(
            |(x, y)| SVGBackend::new(&self.file_name, (x, y)).into_drawing_area(),
            maze,
            self.cell_size,
            self.border_size,
            [DrawingInstructions::DrawMaze(self.colour)],
        )
    }
}

impl<'a> MazeFileWriter for PlottersSvgStringWriter<'a> {
    fn write_maze(&mut self, maze: &Maze) -> Result<(), MazePaintError> {
        write_to_backend(
            |(x, y)| SVGBackend::with_string(self.into_string, (x, y)).into_drawing_area(),
            maze,
            self.cell_size,
            self.border_size,
            [DrawingInstructions::DrawMaze(self.colour)],
        )
    }
}
