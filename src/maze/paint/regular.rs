use std::iter::once;

use itertools::Itertools;
use plotters::{
    coord::Shift,
    prelude::{DrawingArea, IntoDrawingArea, PathElement, Rectangle, SVGBackend},
    style::{Color, RGBAColor, RGBColor},
};

use crate::maze::{
    feature::Svg,
    interface::{MazeRenderer, Solution},
    paint::Gradient,
    shape::regular::{Direction, Direction::*, RectilinearMaze},
};

use super::{BorderWidth, CellSize, WebColour};

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

// because of Plotters' somewhat impractical API, we need to execute all drawing to the SVG file
// in one function. So we can't just bulid up the picture with calls to the renderer, but need to
// collect all instructions first, then execute them all in one place.
enum PlottersWorkaround {
    Stain(WebColour, WebColour),
    Solve(WebColour),
    Paint(WebColour),
}

pub struct RectilinearRenderer<'a> {
    instructions: Vec<PlottersWorkaround>,
    size: (u32, u32),
    maze: &'a RectilinearMaze,
    cell_size: CellSize,
    solution: &'a Solution<(usize, usize)>,
    border_width: BorderWidth,
}

impl<'a> RectilinearRenderer<'a> {
    pub fn new(
        maze: &'a RectilinearMaze,
        solution: &'a Solution<(usize, usize)>,
        stroke_width: f64,
        cell_size: f64,
    ) -> Self {
        let x = cell_size as u32 * maze.get_extents().0 as u32 + stroke_width as u32 * 2;
        let y = cell_size as u32 * maze.get_extents().1 as u32 + stroke_width as u32 * 2;
        Self {
            instructions: Vec::new(),
            size: (x, y),
            maze,
            cell_size: CellSize(cell_size as usize),
            solution,
            border_width: BorderWidth(stroke_width as usize),
        }
    }

    fn render_maze(&self, pic: &DrawingArea<SVGBackend, Shift>, colour: &WebColour) {
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
    }

    fn stain_maze(&self, pic: &DrawingArea<SVGBackend, Shift>, colours: (WebColour, WebColour)) {
        let cell_size = self.cell_size.0 as i32;
        let border = self.border_width.0 as i32;
        let gradient = Gradient::new(colours, self.maze, self.solution);

        for (x, y) in (0..self.maze.get_extents().0).cartesian_product(0..self.maze.get_extents().1)
        {
            let x0: i32 = cell_size * x as i32 + border;
            let y0: i32 = cell_size * y as i32 + border;
            let x1: i32 = x0 + cell_size;
            let y1: i32 = y0 + cell_size;
            let shade = gradient.compute(&(x, y));
            let style = RGBColor(shade.r, shade.g, shade.b).filled();
            pic.draw(&Rectangle::new([(x0 - 2, y0 - 2), (x1 + 2, y1 + 2)], style))
                .unwrap();
        }
    }

    pub fn solve_maze(&self, pic: &DrawingArea<SVGBackend, Shift>, colour: WebColour) {
        let border = self.border_width.0 as i32;
        let cell_size = self.cell_size.0 as i32;
        let path_offset = border + (cell_size / 2);
        let to_coord = |a| cell_size * a as i32 + path_offset;
        let offset_entrance = {
            let (x, _y) = self.maze.get_entrance();
            (to_coord(x), 0)
        };
        let offset_exit = {
            let (x, y) = self.maze.get_exit();
            (to_coord(x), to_coord(y) + path_offset)
        };
        let path: Vec<_> = once(offset_entrance)
            .chain(
                self.solution
                    .path
                    .iter()
                    .map(|&(x, y)| (to_coord(x), to_coord(y))),
            )
            .chain(once(offset_exit))
            .collect();
        pic.draw(&PathElement::new(
            path,
            Into::<RGBAColor>::into(colour).stroke_width(border as u32 * 4),
        ))
        .unwrap();
    }
}

impl MazeRenderer<RectilinearMaze> for RectilinearRenderer<'_> {
    fn stain(&mut self, gradient: (WebColour, WebColour)) {
        self.instructions
            .push(PlottersWorkaround::Stain(gradient.0, gradient.1));
    }

    fn solve(&mut self, stroke_colour: WebColour) {
        self.instructions
            .push(PlottersWorkaround::Solve(stroke_colour));
    }

    fn paint(&mut self, border: WebColour) {
        self.instructions.push(PlottersWorkaround::Paint(border));
    }

    fn render(&self) -> Svg {
        let mut str = String::new();
        {
            let pic = SVGBackend::with_string(&mut str, self.size).into_drawing_area();
            for instruction in &self.instructions {
                match instruction {
                    PlottersWorkaround::Stain(a, b) => self.stain_maze(&pic, (*b, *a)),
                    PlottersWorkaround::Solve(colour) => self.solve_maze(&pic, *colour),
                    PlottersWorkaround::Paint(colour) => self.render_maze(&pic, colour),
                }
            }
            pic.present().unwrap()
        }

        Svg(str)
    }
}

#[cfg(test)]
mod test {
    use crate::maze::shape::regular::RectilinearMaze;

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
