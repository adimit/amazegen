use std::ops::Index;

use itertools::Itertools;

use crate::maze::{
    algorithms::{dijkstra, find_path},
    arengee::Arengee,
    interface::{Maze, Solution},
};

use super::coordinates::Cartesian;

pub enum Direction {
    ALPHA,
    WEST,
    EAST,
}

#[derive(Debug)]
struct Neighbours {
    alpha: Option<Cartesian<u32>>,
    west: Option<Cartesian<u32>>,
    east: Option<Cartesian<u32>>,
}

impl Neighbours {
    fn new() -> Self {
        Self {
            alpha: None,
            west: None,
            east: None,
        }
    }
}

impl Index<Direction> for Neighbours {
    fn index(&self, index: Direction) -> &Self::Output {
        match index {
            Direction::ALPHA => &self.alpha,
            Direction::WEST => &self.west,
            Direction::EAST => &self.east,
        }
    }

    type Output = Option<Cartesian<u32>>;
}

impl Cartesian<u32> {
    fn checked_north(&self) -> Option<Self> {
        let (x, y) = self.get();
        y.checked_sub(1).map(|y| Cartesian::new(x, y))
    }
    fn checked_west(&self) -> Option<Self> {
        let (x, y) = self.get();
        x.checked_sub(1).map(|x| Cartesian::new(x, y))
    }
    fn checked_east(&self, size: u32) -> Option<Self> {
        let (x, y) = self.get();
        x.checked_add(1).and_then(|x| {
            if x < size {
                Some(Cartesian::new(x, y))
            } else {
                None
            }
        })
    }
    fn checked_south(&self, size: u32) -> Option<Self> {
        let (x, y) = self.get();
        y.checked_add(1).and_then(|y| {
            if y < size {
                Some(Cartesian::new(x, y))
            } else {
                None
            }
        })
    }
}

#[derive(Debug)]
struct DeltaCell {
    coordinates: Cartesian<u32>,
    inaccessible: Neighbours,
    accessible: Neighbours,
}

pub fn is_top<C: Into<Cartesian<u32>>>(coordinates: C) -> bool {
    let (x, y) = coordinates.into().get();
    (x + y) % 2 == 0
}

impl DeltaCell {
    fn new<C: Into<Cartesian<u32>> + Copy>(coordinates: C, size: u32) -> Self {
        if is_top(coordinates) {
            Self::new_top(coordinates.into(), size)
        } else {
            Self::new_bottom(coordinates.into(), size)
        }
    }

    fn new_top(coordinates: Cartesian<u32>, size: u32) -> Self {
        Self {
            coordinates,
            inaccessible: Neighbours {
                alpha: coordinates.checked_north(),
                west: coordinates.checked_west(),
                east: coordinates.checked_east(size),
            },
            accessible: Neighbours::new(),
        }
    }

    fn new_bottom(coordinates: Cartesian<u32>, size: u32) -> Self {
        Self {
            coordinates,
            inaccessible: Neighbours {
                alpha: coordinates.checked_south(size),
                west: coordinates.checked_west(),
                east: coordinates.checked_east(size),
            },
            accessible: Neighbours::new(),
        }
    }

    fn carve(&mut self, neighbour: Cartesian<u32>) {
        self.inaccessible.alpha.inspect(|alpha| {
            if alpha == &neighbour {
                self.inaccessible.alpha = None;
                self.accessible.alpha = Some(neighbour);
            }
        });
        self.inaccessible.west.inspect(|west| {
            if west == &neighbour {
                self.inaccessible.west = None;
                self.accessible.west = Some(neighbour);
            }
        });
        self.inaccessible.east.inspect(|east| {
            if east == &neighbour {
                self.inaccessible.east = None;
                self.accessible.east = Some(neighbour);
            }
        });
    }
}

#[derive(Debug)]
pub struct DeltaMaze {
    size: u32,
    cells: Vec<DeltaCell>,
}

impl DeltaMaze {
    pub fn new(size: u32) -> Self {
        let mut cells = Vec::with_capacity((size * size) as usize);
        for y in 0..size {
            for x in 0..size {
                cells.push(DeltaCell::new(Cartesian::new(x, y), size));
            }
        }
        Self { size, cells }
    }

    /// May only be called on a top cell
    fn set_entrance(&mut self, coords: Cartesian<u32>) {
        let index = coords.regular_index(self.size) as usize;
        self.cells[index].accessible.alpha = Some(coords);
    }
    /// May only be called on a bottom cell
    fn set_exit(&mut self, coords: Cartesian<u32>) {
        let index = coords.regular_index(self.size) as usize;
        self.cells[index].accessible.alpha = Some(coords);
    }

    pub fn has_path(&self, a: &Cartesian<u32>, direction: Direction) -> bool {
        let cell = &self.cells[a.regular_index(self.size) as usize];
        cell.accessible[direction].is_some()
    }
    pub fn size(&self) -> u32 {
        self.size
    }
}

impl Maze for DeltaMaze {
    type Idx = Cartesian<u32>;

    fn carve(&mut self, node: Self::Idx, neighbour: Self::Idx) {
        let a = node.regular_index(self.size);
        let b = neighbour.regular_index(self.size);
        self.cells[a as usize].carve(neighbour);
        self.cells[b as usize].carve(node);
    }

    fn get_walls(&self, node: Self::Idx) -> Vec<Self::Idx> {
        let cell = &self.cells[node.regular_index(self.size) as usize];
        [
            cell.inaccessible.west,
            cell.inaccessible.east,
            cell.inaccessible.alpha,
        ]
        .iter()
        .filter_map(|x| *x)
        .collect()
    }

    fn get_paths(&self, node: Self::Idx) -> Vec<Self::Idx> {
        let cell = &self.cells[node.regular_index(self.size) as usize];
        [
            cell.accessible.west,
            cell.accessible.east,
            cell.accessible.alpha,
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    fn get_random_node(&self, rng: &mut Arengee) -> Self::Idx {
        self.cells[rng.get_portable_usize(0..self.cells.len())].coordinates
    }

    fn get_all_edges(&self) -> Vec<(Self::Idx, Self::Idx)> {
        self.cells
            .iter()
            .flat_map(|c| {
                let mut edges = Vec::with_capacity(3);
                if !is_top(c.coordinates) {
                    c.inaccessible
                        .alpha
                        .map(|alpha| edges.push((c.coordinates, alpha)));
                }
                c.inaccessible
                    .west
                    .map(|west| edges.push((c.coordinates, west)));
                c.inaccessible
                    .east
                    .map(|east| edges.push((c.coordinates, east)));
                edges
            })
            .collect()
    }

    fn get_all_nodes(&self) -> Vec<Self::Idx> {
        self.cells.iter().map(|c| c.coordinates).collect()
    }

    fn get_index(&self, node: Self::Idx) -> usize {
        node.regular_index(self.size) as usize
    }

    fn make_solution(&mut self, rng: &mut Arengee) -> Solution<Self::Idx> {
        let possible_entrances = (0..self.size)
            .map(|x| Cartesian::new(x, 0))
            .filter(|c| is_top(*c))
            .collect_vec();

        let possible_exits = (0..self.size)
            .map(|x| Cartesian::new(x, self.size - 1))
            .filter(|c| !is_top(*c))
            .collect_vec();

        let seed_topo = dijkstra(self, *rng.choice(&possible_entrances));
        let exit: Cartesian<u32> = *possible_exits
            .iter()
            .max_by_key(|c| seed_topo.get(self.get_index(**c)))
            .unwrap_or_else(|| rng.choice(&possible_exits));

        let exit_topo = dijkstra(self, exit);
        let entrance: Cartesian<u32> = *possible_entrances
            .iter()
            .max_by_key(|c| exit_topo.get(self.get_index(**c)))
            .unwrap_or_else(|| rng.choice(&possible_entrances));

        let entrance_topo = dijkstra(self, entrance);
        self.set_entrance(entrance);
        self.set_exit(exit);
        let path = find_path(self, &exit_topo, entrance, exit);

        Solution {
            path,
            distances: entrance_topo,
        }
    }
}

#[cfg(test)]
mod test {
    use super::DeltaMaze;
    use crate::maze::{algorithms::jarník, arengee::Arengee, interface::Maze};

    #[test]
    fn maze_template_creation_is_correct() {
        let maze = DeltaMaze::new(3);
        assert_eq!(maze.get_all_nodes().len(), 9);
        [0, 1, 2].iter().for_each(|y| {
            let westest_cell = &maze.cells[maze.get_index((0, *y).into()) as usize];
            assert_eq!(
                westest_cell.inaccessible.west, None,
                "{:?} shouldn't have a western neighbour",
                westest_cell
            );
            assert_eq!(
                westest_cell.inaccessible.east,
                Some((1, *y).into()),
                "{:?} should have an eastern neighbour",
                westest_cell
            );
            let eastest_cell = &maze.cells[maze.get_index((2, *y).into()) as usize];
            assert_eq!(
                eastest_cell.inaccessible.east,
                None,
                "{:?}{:?}{:?} shouldn't have an eastern neighbour",
                (2, *y),
                maze.get_index((2, *y).into()),
                eastest_cell
            );
            assert_eq!(
                eastest_cell.inaccessible.west,
                Some((1, *y).into()),
                "{:?} should have a western neighbour",
                eastest_cell
            );
        });
        [(0, 0), (2, 0), (1, 2)].iter().for_each(|&(x, y)| {
            let cell = &maze.cells[maze.get_index((x, y).into()) as usize];
            assert_eq!(
                cell.inaccessible.alpha, None,
                "{:?} shouldn't have alpha neighbour",
                cell
            );
        });
        [(1, 0), (0, 1), (1, 1), (2, 1), (0, 2), (2, 2)]
            .iter()
            .for_each(|&(x, y)| {
                let cell = &maze.cells[maze.get_index((x, y).into()) as usize];
                assert!(
                    cell.inaccessible.alpha.is_some(),
                    "{:?} should have alpha neighbour",
                    cell
                );
            });
    }

    #[test]
    fn generate_maze_with_jarník() {
        let mut rng = Arengee::new(1);
        let maze_template = DeltaMaze::new(10);
        println!("{:?}", maze_template);
        let mut maze = jarník(maze_template, &mut rng);
        let solution = maze.make_solution(&mut rng);
        let entrance = solution.path[0];
        assert_eq!(entrance.y(), 0);
        let exit = solution.path.last().unwrap();
        assert_eq!(exit.y(), 9);
    }
}
