use std::{
    fmt::{
        Display, 
        Formatter
    }, 
    collections::{
        HashMap, 
        HashSet
    }
};

struct Grid<T> {
    _data: Vec<T>,
    width: usize,
    height: usize
}

impl<T: Clone> Grid<T> {
    fn new(width: usize, height: usize, default: T) -> Grid<T> {
        Grid {
            _data: vec![default; width * height],
            width,
            height,
        }
    }

    fn get(&self, x: usize, y: usize) -> &T {
        &self._data[y * self.width + x]
    }

    fn get_mut(&mut self, x: usize, y: usize) -> &mut T {
        &mut self._data[y * self.width + x]
    }
}

impl<T: Clone + Display> Display for Grid<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                write!(f, "{} ", self.get(x, y))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Copy)]
enum Piece {
    Red,
    Yellow,
    Empty
}

impl Display for Piece {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Piece::Red => "0",
            Piece::Yellow => "O",
            Piece::Empty => " "
        })?;
        Ok(())
    }
}

type Position = (usize, usize);

struct Board {
    grid: Grid<Piece>,
    drop_zones: Vec<usize>,
    threats: HashMap<Piece, HashSet<Position>>,
    winner: Option<Piece>
}

impl Board {
    fn new_with_size(width: usize, height: usize) -> Board {
        let mut threats: HashMap<Piece, HashSet<Position>> = HashMap::new();
        threats.insert(Piece::Red, HashSet::new());
        threats.insert(Piece::Yellow, HashSet::new());
        Board {
            grid: Grid::new(width, height, Piece::Empty),
            drop_zones: vec![height - 1; width],
            threats: threats,
            winner: None
        }
    }

    fn new() -> Board {
        Board::new_with_size(7, 6)
    }

    fn get(&self, x: usize, y: usize) -> &Piece {
        &self.grid.get(x, y)
    }
    
    fn set(&mut self, x: usize, y: usize, piece: Piece) {
        *self.grid.get_mut(x, y) = piece;
        if self.threats.get(&piece).unwrap().contains(&(x, y)) {
            self.winner = Some(piece);
        }
        self.update(x, y);
    }

    fn drop(&mut self, x: usize, piece: Piece) -> Option<usize> {
        if self.drop_zones[x] > 0 {
            let drop_spot = self.drop_zones[x];
            self.drop_zones[x] -= 1;
            self.set(x, drop_spot, piece);
            Some(drop_spot)
        } else {
            None
        }
    }

    fn update(&mut self, x: usize, y: usize) {
        let pos_sets = vec![(-1, -1), (-1, 0), (-1, 1), (0, 1)]
            .iter()
            .map(|(dx, dy)| (-3..3)
                .map(|d| ((x as i32) + d * dx, (y as i32) + d * dy))
                .filter(|(px, py)| *px > 0 && *py > 0 && *px < self.grid.width as i32 && *py < self.grid.height as i32)
                .map(|(px, py)| (px as usize, py as usize))
                .map(|(px, py)| ((px, py), self.grid.get(px, py)))
                .collect::<Vec<_>>())
            .filter(|poses| poses.len() >= 4)
            .collect::<Vec<_>>();

        for poses in pos_sets {
            let mut tallies: HashMap<Piece, HashSet<Position>> = HashMap::new();
            for piece_type in vec![Piece::Red, Piece::Yellow, Piece::Empty] {
                tallies.insert(piece_type, HashSet::new());
            }
            
            for i in 0..poses.len() {
                let (pos, piece) = poses[i];
                tallies.get_mut(piece).unwrap().insert(pos);
                if i >= 4 {
                    let (early_pos, early_piece) = poses[i-4];
                    tallies.get_mut(early_piece).unwrap().remove(&early_pos);
                }
                if i >= 3 {
                    let empties = tallies.get(&Piece::Empty).unwrap();
                    if empties.len() == 1 {
                        let gap = empties.iter().next().unwrap();
                        for piece in vec![Piece::Red, Piece::Yellow] {
                            if tallies.get(&piece).unwrap().len() == 3 {
                                self.threats.get_mut(&piece).unwrap().insert(*gap);
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.grid.fmt(f)
    }
}

fn main() {
    let mut board = Board::new();
    board.drop(2, Piece::Yellow);
    println!("{}", board);
    board.drop(3, Piece::Yellow);
    println!("{}", board);
    board.drop(4, Piece::Yellow);
    println!("{}", board);
    board.drop(6, Piece::Yellow);
    println!("{}", board);
    board.drop(5, Piece::Yellow);
    print!("{}", board);
}