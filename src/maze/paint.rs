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

impl<'a> MazeFileWriter for PlottersSvgStringWriter<'a> {
    fn write_maze(&mut self, maze: &Maze) -> Result<(), MazePaintError> {
        use plotters::backend::SVGBackend;
        let border = self.border_size as u32;
        let x = maze.extents.0 as u32 + border * 2;
        let y = maze.extents.1 as u32 + border * 2;
        let cell_size = self.cell_size as i32;
        let mut pic = SVGBackend::with_string(self.into_string, (x, y));
        render_maze(&mut pic, maze, border as i32, cell_size)
    }
}

fn render_maze<'a>(
    pic: &'a mut plotters::backend::SVGBackend,
    maze: &'a Maze,
    border: i32,
    cell_size: i32,
) -> Result<(), MazePaintError> {
    use super::Direction::*;
    use plotters::prelude::*;

    let mut h = get_wall_runs(&maze, Up);
    h.push(get_wall_run(&maze, maze.extents.0 - 1, Down));
    let mut v = get_wall_runs(&maze, Left);
    v.push(get_wall_run(&maze, maze.extents.1 - 1, Right));

    for (y, xs) in h.iter().enumerate() {
        let y_offset: i32 = y as i32 * cell_size;
        for (start, end) in xs {
            let x0: i32 = (*start as i32 * cell_size).try_into().unwrap();
            let xe: i32 = ((*end as i32 + 1) * cell_size).try_into().unwrap();
            pic.draw_line(
                ((x0), y_offset + border),
                ((xe + 2 * border), y_offset + border),
                &BLACK.stroke_width((border * 2).try_into().unwrap()),
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
                &BLACK.stroke_width((border * 2).try_into().unwrap()),
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

        render_maze(&mut pic, maze, border, cell_size)
    }
}

#[derive(Debug)]
pub struct PlottersBitmapWriter {
    border_size: usize,
    cell_size: usize,
    file_name: String,
}

impl PlottersBitmapWriter {
    pub fn new(file_name: String, cell_size: usize, border_size: usize) -> Self {
        Self {
            border_size,
            cell_size,
            file_name,
        }
    }
}

impl MazeFileWriter for PlottersBitmapWriter {
    fn write_maze(&mut self, maze: &Maze) -> Result<(), MazePaintError> {
        use plotters::prelude::*;

        use super::Direction::*;
        let mut pic = BitMapBackend::new(
            &self.file_name,
            (
                (maze.extents.0 * self.cell_size).try_into().unwrap(),
                (maze.extents.1 * self.cell_size).try_into().unwrap(),
            ),
        );

        use itertools::Itertools;
        let border_width: i32 = self.border_size.try_into().unwrap();

        let cells = (0..maze.extents.0).cartesian_product(0..maze.extents.1);
        cells.for_each(|(x, y)| {
            if maze.is_visited((x, y)) {
                let x0: i32 = (x * self.cell_size).try_into().unwrap();
                let y0: i32 = (y * self.cell_size).try_into().unwrap();
                let x1: i32 = ((1 + x) * self.cell_size).try_into().unwrap();
                let y1: i32 = ((1 + y) * self.cell_size).try_into().unwrap();

                pic.draw_rect(
                    (x0 + border_width, y0 + border_width),
                    (x1 - border_width, y1 - border_width),
                    &WHITE,
                    true,
                )
                .unwrap();

                maze.get_open_paths((x, y))
                    .iter()
                    .for_each(|direction| match direction {
                        Up => pic
                            .draw_rect(
                                (x0 + border_width, y0),
                                (x1 - border_width, y0 + border_width),
                                &WHITE,
                                true,
                            )
                            .unwrap(),
                        Right => pic
                            .draw_rect(
                                (x1 - border_width, y0 + border_width),
                                (x1, y1 - border_width),
                                &WHITE,
                                true,
                            )
                            .unwrap(),
                        Down => pic
                            .draw_rect(
                                (x0 + border_width, y1 - border_width),
                                (x1 - border_width, y1),
                                &WHITE,
                                true,
                            )
                            .unwrap(),
                        Left => pic
                            .draw_rect(
                                (x0, y0 + border_width),
                                (x0 + border_width, y1 - border_width),
                                &WHITE,
                                true,
                            )
                            .unwrap(),
                    })
            }
        });

        pic.present().unwrap();

        Ok(())
    }
}
