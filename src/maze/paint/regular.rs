use std::cell::RefCell;

use itertools::Itertools;
use plotters::{
    coord::Shift,
    prelude::{DrawingArea, IntoDrawingArea, PathElement, Rectangle, SVGBackend},
    style::{Color, RGBAColor, RGBColor},
};

use crate::maze::{
    regular::{Direction, Direction::*, RectilinearMaze},
    solver::Solver,
    Maze,
};

use super::{
    BorderWidth, CellSize, DrawingInstructions, MazeFileWriter, MazePaintError,
    PlottersSvgFileWriter, PlottersSvgStringWriter, Visuals, WebColour,
};

impl<'a> Visuals<'a, RectilinearMaze> {
    pub fn render_maze(&self, colour: &WebColour) -> Result<(), MazePaintError> {
        let cell_size = self.cell_size.0 as i32;
        let border = self.border_width.0 as i32;

        let mut h = get_wall_runs(self.maze, Up);
        h.push(get_wall_run(self.maze, self.maze.get_extents().0 - 1, Down));
        let mut v = get_wall_runs(self.maze, Left);
        v.push(get_wall_run(
            self.maze,
            self.maze.get_extents().1 - 1,
            Right,
        ));
        let style = {
            let svg_colour: RGBAColor = (*colour).into();
            svg_colour.stroke_width((border * 2).try_into().unwrap())
        };

        for (y, xs) in h.iter().enumerate() {
            let y_offset: i32 = y as i32 * cell_size;
            for (start, end) in xs {
                let x0: i32 = *start as i32 * cell_size;
                let xe: i32 = (*end as i32 + 1) * cell_size;
                self.pic
                    .draw(&PathElement::new(
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
                self.pic
                    .draw(&PathElement::new(
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

    pub fn stain_maze(&self, colours: (WebColour, WebColour)) -> Result<(), MazePaintError> {
        let cell_size = self.cell_size.0 as i32;
        let border = self.border_width.0 as i32;
        let solver = self.get_solver();
        let distances = solver.get_distances_from_origin();
        let max_distance: usize = *distances
            .iter()
            .map(move |dim| dim.iter().max().unwrap_or(&0))
            .max()
            .unwrap_or(&0);

        fn get_colour(absolute: u8, fraction: f64) -> u8 {
            (absolute as f64 * fraction) as u8
        }

        for (x, y) in (0..self.maze.get_extents().0).cartesian_product(0..self.maze.get_extents().1)
        {
            let x0: i32 = cell_size * x as i32 + border;
            let y0: i32 = cell_size * y as i32 + border;
            let x1: i32 = x0 + cell_size;
            let y1: i32 = y0 + cell_size;
            let intensity = (max_distance - distances[x][y]) as f64 / max_distance as f64;
            let inverse = 1.0 - intensity;
            let style = RGBColor(
                get_colour(colours.0.r, intensity) + get_colour(colours.1.r, inverse),
                get_colour(colours.0.g, intensity) + get_colour(colours.1.g, inverse),
                get_colour(colours.0.b, intensity) + get_colour(colours.1.b, inverse),
            )
            .filled();
            self.pic
                .draw(&Rectangle::new([(x0 - 2, y0 - 2), (x1 + 2, y1 + 2)], style))
                .unwrap();
        }
        Ok(())
    }

    fn get_solver(&self) -> Solver<'a, RectilinearMaze> {
        self.solver
            .borrow_mut()
            .get_or_insert_with(|| Solver::new(self.maze, self.maze.get_entrance()))
            .clone()
    }

    pub fn solve_maze(&self, colour: WebColour) -> Result<(), MazePaintError> {
        {
            let border = self.border_width.0 as i32;
            let cell_size = self.cell_size.0 as i32;
            let path_offset = border + (cell_size / 2);
            let to_coord = |a| cell_size * a as i32 + path_offset;
            let mut solution: Vec<(i32, i32)> = {
                let exit = self.maze.get_exit();
                vec![(to_coord(exit.0), to_coord(exit.1) + path_offset)]
            };
            let solver = self.get_solver();
            solution.extend(solver.solve_maze().iter().map(|(x, y)| {
                let x0: i32 = to_coord(*x);
                let y0: i32 = to_coord(*y);
                (x0, y0)
            }));
            solution.push((to_coord(self.maze.get_entrance().0), 0));
            self.pic
                .draw(&PathElement::new(
                    solution,
                    Into::<RGBAColor>::into(colour).stroke_width(border as u32 * 4),
                ))
                .unwrap();
            Ok(())
        }
    }
}

impl MazeFileWriter<(usize, usize), RectilinearMaze> for PlottersSvgFileWriter {
    fn write_maze<I>(
        &mut self,
        maze: &RectilinearMaze,
        instructions: I,
    ) -> Result<(), MazePaintError>
    where
        I: IntoIterator<Item = DrawingInstructions>,
    {
        write_to_backend(
            |(x, y)| SVGBackend::new(&self.file_name, (x, y)).into_drawing_area(),
            maze,
            self.cell_size,
            self.border_size,
            instructions,
        )
    }
}

impl<'a> MazeFileWriter<(usize, usize), RectilinearMaze> for PlottersSvgStringWriter<'a> {
    fn write_maze<I>(
        &mut self,
        maze: &RectilinearMaze,
        instructions: I,
    ) -> Result<(), MazePaintError>
    where
        I: IntoIterator<Item = DrawingInstructions>,
    {
        write_to_backend(
            |(x, y)| SVGBackend::with_string(self.into_string, (x, y)).into_drawing_area(),
            maze,
            self.cell_size,
            self.border_size,
            instructions,
        )
    }
}

fn write_to_backend<'a, F, I>(
    make_drawing_area: F,
    maze: &'a RectilinearMaze,
    cell_size: usize,
    border_size: usize,
    instructions: I,
) -> Result<(), MazePaintError>
where
    F: FnOnce((u32, u32)) -> DrawingArea<SVGBackend<'a>, Shift>,
    I: IntoIterator<Item = DrawingInstructions>,
{
    let border = border_size as u32;
    let x = cell_size as u32 * maze.get_extents().0 as u32 + border * 2;
    let y = cell_size as u32 * maze.get_extents().1 as u32 + border * 2;

    let visual = Visuals {
        border_width: BorderWidth(border as usize),
        cell_size: CellSize(cell_size),
        pic: make_drawing_area((x, y)),
        maze,
        solver: RefCell::new(None),
    };

    for instruction in instructions {
        instruction.execute(&visual).unwrap();
    }

    visual.pic.present().unwrap();

    Ok(())
}

fn get_wall_runs(maze: &RectilinearMaze, direction: Direction) -> Vec<Vec<(usize, usize)>> {
    match direction {
        Up | Down => (0..maze.get_extents().1)
            .map(move |y| get_wall_run(maze, y, direction))
            .collect::<Vec<_>>(),
        Left | Right => (0..maze.get_extents().0)
            .map(move |x| get_wall_run(maze, x, direction))
            .collect::<Vec<_>>(),
    }
}

fn get_wall_run(maze: &RectilinearMaze, line: usize, direction: Direction) -> Vec<(usize, usize)> {
    // The match arms would have an incompatible closure type, which is
    // why we duplicate the code here. There might be a better option,
    // but I'm not aware of it.
    match direction {
        Up | Down => (0..maze.get_extents().0)
            .group_by(move |x| maze.has_wall((*x, line), direction))
            .into_iter()
            .filter(|(key, _)| *key)
            .map(|(_, group)| {
                let run = group.collect::<Vec<_>>();
                (*run.first().unwrap(), *run.last().unwrap())
            })
            .collect::<Vec<_>>(),
        Left | Right => (0..maze.get_extents().1)
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
    use crate::maze::regular::RectilinearMaze;

    use super::*;

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
        let mut maze = RectilinearMaze::new((10, 2));
        maze.move_from_to((1, 0), (1, 1));
        maze.move_from_to((5, 0), (5, 1));

        assert_eq!(
            get_wall_runs(&maze, Up),
            [vec![(0, 9)], vec![(0, 0), (2, 4), (6, 9)]]
        );
    }

    #[test]
    fn get_wall_runs_works_vertically() {
        let mut maze = RectilinearMaze::new((2, 10));
        maze.move_from_to((0, 2), (1, 2));
        maze.move_from_to((0, 5), (1, 5));

        assert_eq!(
            get_wall_runs(&maze, Left),
            [vec![(0, 9)], vec![(0, 1), (3, 4), (6, 9)]]
        );
    }
}
