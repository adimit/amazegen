use super::Maze;
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

pub fn get_wall_runs(maze: &Maze, direction: super::Direction) -> Vec<Vec<(usize, usize)>> {
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

pub fn get_wall_run(maze: &Maze, line: usize, direction: super::Direction) -> Vec<(usize, usize)> {
    use super::Direction::*;
    use itertools::Itertools;

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
                (run.first().unwrap().clone(), run.last().unwrap().clone())
            })
            .collect::<Vec<_>>(),
        Left | Right => (0..maze.extents.1)
            .group_by(move |y| maze.has_wall((line, *y), direction))
            .into_iter()
            .filter(|(key, _)| *key)
            .map(|(_, group)| {
                let run = group.collect::<Vec<_>>();
                (run.first().unwrap().clone(), run.last().unwrap().clone())
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

impl<'a> MazeFileWriter for PlottersSvgStringWriter<'a> {
    fn write_maze(&mut self, maze: &Maze) -> Result<(), MazePaintError> {
        use plotters::backend::SVGBackend;
        let border = self.border_size as u32;
        let cell_size = self.cell_size as i32;
        let x = cell_size as u32 * maze.extents.0 as u32 + border * 2;
        let y = cell_size as u32 * maze.extents.1 as u32 + border * 2;
        let mut pic = SVGBackend::with_string(self.into_string, (x, y));
        render_maze(&mut pic, maze, border as i32, cell_size, &self.colour)
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

impl Into<plotters::style::RGBAColor> for WebColour {
    fn into(self) -> plotters::style::RGBAColor {
        plotters::style::RGBAColor(self.r, self.g, self.b, self.a as f64 / 255.0)
    }
}

fn render_maze<'a>(
    pic: &'a mut plotters::backend::SVGBackend,
    maze: &'a Maze,
    border: i32,
    cell_size: i32,
    colour: &WebColour,
) -> Result<(), MazePaintError> {
    use super::Direction::*;
    use plotters::prelude::*;

    let mut h = get_wall_runs(&maze, Up);
    h.push(get_wall_run(&maze, maze.extents.0 - 1, Down));
    let mut v = get_wall_runs(&maze, Left);
    v.push(get_wall_run(&maze, maze.extents.1 - 1, Right));
    let svg_colour: RGBAColor = (*colour).into();
    let style = svg_colour.stroke_width((border * 2).try_into().unwrap());

    for (y, xs) in h.iter().enumerate() {
        let y_offset: i32 = y as i32 * cell_size;
        for (start, end) in xs {
            let x0: i32 = (*start as i32 * cell_size).try_into().unwrap();
            let xe: i32 = ((*end as i32 + 1) * cell_size).try_into().unwrap();
            pic.draw_line(
                ((x0), y_offset + border),
                ((xe + 2 * border), y_offset + border),
                &style,
            )
            .unwrap();
        }
    }
    for (x, ys) in v.iter().enumerate() {
        let x_offset: i32 = x as i32 * cell_size;
        for (start, end) in ys {
            let y0: i32 = (*start as i32 * cell_size).try_into().unwrap();
            let ye: i32 = ((*end as i32 + 1) * cell_size).try_into().unwrap();
            pic.draw_line(
                (x_offset + border, (y0)),
                (x_offset + border, (ye + 2 * border)),
                &style,
            )
            .unwrap();
        }
    }
    pic.present().unwrap();

    Ok(())
}

impl MazeFileWriter for PlottersSvgFileWriter {
    fn write_maze(&mut self, maze: &Maze) -> Result<(), MazePaintError> {
        use plotters::backend::SVGBackend;
        let xmax: u32 = (maze.extents.0 * self.cell_size).try_into().unwrap();
        let ymax: u32 = (maze.extents.1 * self.cell_size).try_into().unwrap();
        let border: i32 = self.border_size.try_into().unwrap();
        let double_border: u32 = (border * 2).try_into().unwrap();
        let mut pic = SVGBackend::new(
            &self.file_name,
            (xmax + double_border, ymax + double_border),
        );
        let cell_size: i32 = self.cell_size.try_into().unwrap();

        render_maze(&mut pic, maze, border, cell_size, &self.colour)
    }
}
