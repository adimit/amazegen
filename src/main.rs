mod maze {
    use std::{collections::HashMap, hash::Hash};

    // UnorderedEq is a way to compare vectors without paying heed to the order of the elements.
    // It's lifted from this SO answer: https://stackoverflow.com/a/42748484
    #[derive(Debug, Copy, Clone)]
    struct UnorderedEq<'a, T: 'a>(&'a [T]);

    impl<'a, T> UnorderedEq<'a, T>
    where
        T: Eq + Hash,
    {
        fn count(&self) -> HashMap<&T, usize> {
            let mut cnt = HashMap::new();
            for i in self.0 {
                *cnt.entry(i).or_insert(0) += 1
            }
            cnt
        }
    }

    impl<'a, T> PartialEq for UnorderedEq<'a, T>
    where
        T: Eq + Hash,
    {
        fn eq(&self, other: &Self) -> bool {
            self.count() == other.count()
        }
    }

    #[derive(Debug, Clone)]
    pub struct Maze {
        fields: std::vec::Vec<Vec<u8>>,
        pub extents: (usize, usize),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Direction {
        Up,
        Right,
        Down,
        Left,
    }

    impl Direction {
        pub fn reciprocal(&self) -> Direction {
            match self {
                Direction::Left => Direction::Right,
                Direction::Right => Direction::Left,
                Direction::Up => Direction::Down,
                Direction::Down => Direction::Up,
            }
        }

        fn bitmask(&self) -> u8 {
            match self {
                Direction::Left => LEFT,
                Direction::Right => RIGHT,
                Direction::Up => UP,
                Direction::Down => DOWN,
            }
        }
        pub fn iterator() -> impl Iterator<Item = Direction> {
            [
                Direction::Up,
                Direction::Down,
                Direction::Left,
                Direction::Right,
            ]
            .iter()
            .copied()
        }
    }

    const VISIT: u8 = 1u8;
    const LEFT: u8 = 2u8;
    const UP: u8 = 4u8;
    const RIGHT: u8 = 8u8;
    const DOWN: u8 = 16u8;

    impl Maze {
        pub fn new(extents: (usize, usize)) -> Self {
            Maze {
                extents,
                fields: vec![vec![0u8; extents.1]; extents.0],
            }
        }

        pub fn visit(&mut self, (x, y): (usize, usize)) {
            if self.coordinates_in_extents((x, y)) {
                self.fields[x][y] |= VISIT
            } else {
                panic!("Can't visit coordinates ({x}, {y})")
            }
        }

        fn coordinates_in_extents(&self, (x, y): (usize, usize)) -> bool {
            x < self.extents.0 && y < self.extents.1
        }

        pub fn is_visited(&self, (x, y): (usize, usize)) -> bool {
            self.coordinates_in_extents((x, y)) && self.fields[x][y] & VISIT > 0
        }

        pub fn translate(
            &self,
            (x, y): (usize, usize),
            direction: Direction,
        ) -> Option<(usize, usize)> {
            match direction {
                Direction::Left if x > 0 => Some((x - 1, y)),
                Direction::Right if x < self.extents.0 - 1 => Some((x + 1, y)),
                Direction::Up if y > 0 => Some((x, y - 1)),
                Direction::Down if y < self.extents.1 - 1 => Some((x, y + 1)),
                _ => None,
            }
        }

        pub fn move_from(
            &mut self,
            (x, y): (usize, usize),
            direction: Direction,
        ) -> Option<(usize, usize)> {
            let (tx, ty) = self.translate((x, y), direction)?;

            if !self.coordinates_in_extents((x, y)) || !self.coordinates_in_extents((tx, ty)) {
                panic!("origin or target out of bounds: ({x:?}, {y:?}) → {tx:?}, {ty:?}")
            }

            self.fields[x][y] |= VISIT | direction.bitmask();

            self.fields[tx][ty] |= VISIT | direction.reciprocal().bitmask();

            Some((tx, ty))
        }

        pub fn get_open_paths(&self, (x, y): (usize, usize)) -> Vec<Direction> {
            Direction::iterator()
                .filter(|direction| {
                    let mask = direction.bitmask();
                    self.fields[x][y] & mask != 0
                })
                .collect()
        }

        pub fn remove_wall(&mut self, (x, y): (usize, usize), direction: Direction) {
            self.fields[x][y] |= direction.bitmask()
        }

        pub fn get_walls(&self, (x, y): (usize, usize)) -> Vec<Direction> {
            Direction::iterator()
                .filter(|direction| {
                    let mask = direction.bitmask();
                    self.fields[x][y] & mask == 0
                })
                .collect()
        }

        pub fn has_wall(&self, (x, y): (usize, usize), direction: Direction) -> bool {
            self.fields[x][y] & direction.bitmask() == 0
        }

        pub fn get_possible_paths(&self, (x, y): (usize, usize)) -> Vec<Direction> {
            Direction::iterator()
                .filter(|direction| match self.translate((x, y), *direction) {
                    Some((tx, ty)) => {
                        self.fields[tx][ty] & VISIT == 0
                            && self.fields[x][y] & direction.bitmask() == 0
                    }
                    None => false,
                })
                .collect()
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use Direction::*;

        #[test]
        fn is_visited_should_return_true_when_field_has_been_visited() {
            let mut m = Maze::new((12, 12));
            m.visit((3, 2));
            assert!(m.is_visited((3, 2)));
            assert!(!m.is_visited((0, 0)));
        }

        #[test]
        fn visit_is_idempotent() {
            let mut m = Maze::new((12, 12));
            m.visit((5, 5));
            assert_eq!(m.fields[5][5], 1);
            m.visit((5, 5));
            assert_eq!(m.fields[5][5], 1);
        }

        #[test]
        fn move_marks_both_as_visited() {
            let mut m = Maze::new((12, 12));
            m.move_from((1, 1), Down);
            assert!(m.is_visited((1, 1)));
            assert!(m.is_visited((1, 2)));
        }

        #[test]
        fn move_tears_down_the_walls_on_both_sides() {
            let mut m = Maze::new((12, 12));
            m.move_from((1, 1), Down);
            assert_eq!(m.fields[1][1] & DOWN, DOWN);
            assert_eq!(m.fields[1][1] & LEFT, 0);
            assert_eq!(m.fields[1][1] & RIGHT, 0);
            assert_eq!(m.fields[1][1] & UP, 0);

            assert_eq!(m.fields[1][2] & UP, UP);
            assert_eq!(m.fields[1][2] & LEFT, 0);
            assert_eq!(m.fields[1][2] & RIGHT, 0);
            assert_eq!(m.fields[1][2] & DOWN, 0);
        }

        #[test]
        fn get_open_paths_returns_where_there_are_no_walls() {
            let mut m = Maze::new((12, 12));
            m.move_from((2, 2), Left);
            assert_eq!(m.get_open_paths((2, 2)), [Left]);
            assert_eq!(m.get_open_paths((1, 2)), [Right]);
        }

        #[test]
        fn get_walls_returns_where_there_are_walls() {
            let mut m = Maze::new((12, 12));
            m.move_from((2, 2), Left);
            assert_eq!(
                UnorderedEq(&m.get_walls((2, 2))),
                UnorderedEq(&[Right, Up, Down])
            );
            assert_eq!(
                UnorderedEq(&m.get_walls((1, 2))),
                UnorderedEq(&[Left, Up, Down])
            );
        }

        #[test]
        fn get_walls_returns_walls_at_the_edges() {
            let mut m = Maze::new((10, 10));
            m.move_from((0, 0), Down);
            m.move_from((0, 0), Right);
            assert_eq!(UnorderedEq(&m.get_walls((0, 0))), UnorderedEq(&[Left, Up]))
        }

        #[test]
        fn get_possible_paths_filters_edges_of_maze() {
            let m = Maze::new((10, 10));
            assert_eq!(
                UnorderedEq(&m.get_possible_paths((0, 0))),
                UnorderedEq(&[Down, Right])
            );
            assert_eq!(
                UnorderedEq(&m.get_possible_paths((9, 0))),
                UnorderedEq(&[Down, Left])
            );
            assert_eq!(
                UnorderedEq(&m.get_possible_paths((9, 9))),
                UnorderedEq(&[Up, Left])
            );
            assert_eq!(
                UnorderedEq(&m.get_possible_paths((0, 9))),
                UnorderedEq(&[Up, Right])
            );
        }

        #[test]
        fn translate_does_not_allow_going_off_grid() {
            let m = Maze::new((10, 10));
            assert_eq!(m.translate((0, 0), Left), None);
            assert_eq!(m.translate((0, 0), Up), None);
            assert_eq!(m.translate((9, 9), Down), None);
            assert_eq!(m.translate((9, 9), Right), None);
        }

        #[test]
        fn get_possible_paths_filters_visited_cells() {
            let mut m = Maze::new((10, 10));
            m.move_from((1, 1), Left);

            assert_eq!(
                UnorderedEq(&m.get_possible_paths((0, 1))),
                UnorderedEq(&[Up, Down])
            );
            assert_eq!(
                UnorderedEq(&m.get_possible_paths((1, 0))),
                UnorderedEq(&[Left, Right])
            );
        }
    }

    pub mod paint {
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
            fn write_maze(&self, maze: &Maze) -> Result<(), MazePaintError>;
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

        impl MazeFileWriter for PlottersSvgFileWriter {
            fn write_maze(&self, maze: &Maze) -> Result<(), MazePaintError> {
                use super::Direction::*;
                use itertools::Itertools;
                use plotters::prelude::*;
                let border_width: i32 = self.border_size.try_into().unwrap();

                let mut pic = SVGBackend::new(
                    &self.file_name,
                    (
                        (maze.extents.0 * self.cell_size).try_into().unwrap(),
                        (maze.extents.1 * self.cell_size).try_into().unwrap(),
                    ),
                );

                let cells = (0..maze.extents.0).cartesian_product(0..maze.extents.1);
                cells.for_each(|(x, y)| {
                    let x0: i32 = (x * self.cell_size).try_into().unwrap();
                    let y0: i32 = (y * self.cell_size).try_into().unwrap();
                    let x1: i32 = ((1 + x) * self.cell_size).try_into().unwrap();
                    let y1: i32 = ((1 + y) * self.cell_size).try_into().unwrap();

                    let w = maze.get_walls((x, y));
                    w.iter().for_each(|d| match d {
                        Up => pic
                            .draw_rect((x0, y0), (x1, y0 + border_width), &BLACK, true)
                            .unwrap(),
                        Left => pic
                            .draw_rect((x0, y0), (x0 + border_width, y1), &BLACK, true)
                            .unwrap(),
                        Down => pic
                            .draw_rect((x0, y1 - border_width), (x1, y1), &BLACK, true)
                            .unwrap(),
                        Right => pic
                            .draw_rect((x1 - border_width, y0), (x1, y1), &BLACK, true)
                            .unwrap(),
                    })
                });
                pic.present().unwrap();

                Ok(())
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
            fn write_maze(&self, maze: &Maze) -> Result<(), MazePaintError> {
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
    }

    pub mod generator {
        use super::*;
        use rand::prelude::*;
        use rand_chacha::ChaCha8Rng;

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        struct Wall {
            x: usize,
            y: usize,
            d: Direction,
        }

        pub fn jarník(x_size: usize, y_size: usize, seed: u64) -> Maze {
            let mut maze = Maze::new((x_size, y_size));
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let mut vertices: Vec<Wall> = vec![];

            let start = (rng.gen_range(0..x_size), 0);
            maze.visit(start);
            vertices.extend(maze.get_possible_paths(start).iter().map(|d| Wall {
                x: start.0,
                y: start.1,
                d: *d,
            }));

            while !vertices.is_empty() {
                let w = vertices.remove(rng.gen_range(0..vertices.len()));
                match maze.translate((w.x, w.y), w.d) {
                    Some(t) if !maze.is_visited(t) => {
                        maze.move_from((w.x, w.y), w.d);
                        vertices.extend(maze.get_possible_paths(t).iter().map(|d| Wall {
                            x: t.0,
                            y: t.1,
                            d: *d,
                        }))
                    }
                    _ => {}
                }
            }
            maze.remove_wall((rng.gen_range(0..maze.extents.0), 0), Direction::Up);
            maze.remove_wall(
                (rng.gen_range(0..maze.extents.0), maze.extents.1 - 1),
                Direction::Down,
            );
            maze
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use maze::generator::*;
    use maze::paint::*;
    use maze::*;
    let x_size = 15;
    let y_size = 15;

    let maze: Maze = jarník(x_size, y_size, 10);

    PlottersBitmapWriter::new("./test.png".into(), 40, 4).write_maze(&maze)?;
    PlottersSvgFileWriter::new("./test.svg".into(), 40, 4).write_maze(&maze)?;

    Ok(())
}
