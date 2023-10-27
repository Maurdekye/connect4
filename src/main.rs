use std::{
    fmt::{
        Display, 
        Formatter
    }, 
    collections::{
        HashMap, 
        HashSet
    }, io::stdin
};

trait Search {
    fn score<T: Ord, Eq>(&self) -> T;
    fn moves(&self) -> HashSet<Self> where Self: Sized;
}

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
        match self {
            Piece::Red => write!(f, "Red")?,
            Piece::Yellow => write!(f, "Yellow")?,
            Piece::Empty => write!(f, "Empty")?
        };
        Ok(())
    }
}

type Position = (usize, usize);

struct Board {
    grid: Grid<Piece>,
    drop_zones: Vec<usize>,
    threats: HashMap<Piece, HashSet<Position>>,
    winner: Option<Piece>,
    next_move: Piece
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
            winner: None,
            next_move: Piece::Yellow
        }
    }

    fn new() -> Board {
        Board::new_with_size(7, 6)
    }

    // fn get(&self, x: usize, y: usize) -> &Piece {
    //     &self.grid.get(x, y)
    // }
    
    fn set(&mut self, x: usize, y: usize, piece: Piece) {
        *self.grid.get_mut(x, y) = piece;
        if self.threats.get(&piece).unwrap().contains(&(x, y)) {
            self.winner = Some(piece);
        }
        self.threats.iter_mut().for_each(|(_, threat_set)| {threat_set.remove(&(x, y));});
        self.update(x, y);
    }

    fn drop(&mut self, x: usize) -> Option<usize> {
        if self.drop_zones[x] > 0 {
            let drop_spot = self.drop_zones[x];
            self.drop_zones[x] -= 1;
            self.set(x, drop_spot, self.next_move);
            self.next_move = match self.next_move {
                Piece::Yellow => Piece::Red,
                Piece::Red => Piece::Yellow,
                a => a
            };
            Some(drop_spot)
        } else {
            None
        }
    }

    fn update(&mut self, x: usize, y: usize) {
        let pos_sets = vec![(-1, -1), (-1, 0), (-1, 1), (0, 1)]
            .iter()
            .map(|(dx, dy)| (-3..=3)
                .map(|d| ((x as i32) + d * dx, (y as i32) + d * dy))
                .filter(|(px, py)| *px >= 0 && *py >= 0 && *px < self.grid.width as i32 && *py < self.grid.height as i32)
                .map(|(px, py)| (px as usize, py as usize))
                .map(|(px, py)| ((px, py), self.grid.get(px, py)))
                .collect::<Vec<_>>())
            .filter(|poses| poses.len() >= 4)
            .collect::<Vec<_>>();

        // let all_poses = pos_sets.iter().flatten().map(|(pos, _)| pos).collect::<HashSet<_>>();
        // self.print_with_pos_set(&all_poses);

        for poses in pos_sets {
            // self.print_with_pos_set(&poses.iter().map(|(pos, _)| pos).collect());
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

    fn print_with_pos_set(&self, pos_set: &HashSet<&Position>) {
        for sy in 0..self.grid.height {
            print!("| ");
            for sx in 0..self.grid.width {
                let piece = self.grid.get(sx, sy);
                let char = if pos_set.contains(&(sx, sy)) {
                    "*"
                } else {
                    match piece {
                        Piece::Red => "0",
                        Piece::Yellow => "O",
                        Piece::Empty => {
                            let red_threat = self.threats.get(&Piece::Red).unwrap().contains(&(sx, sy));
                            let yellow_threat = self.threats.get(&Piece::Yellow).unwrap().contains(&(sx, sy));
                            match (red_threat, yellow_threat) {
                                (true, true) => "B",
                                (true, false) => "R",
                                (false, true) => "Y",
                                (false, false) => " "
                            }
                        }
                    }
                };
                print!("{} ", char);
            }
            println!("|");
        }
    }
}

// impl Search for Board {
//     fn score<T: Ord, Eq>(&self) -> T {
//         todo!()
//     }

//     fn moves(&self) -> HashSet<Self> where Self: Sized {
//         todo!()
//     }
// }

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.grid.height {
            write!(f, "| ")?;
            for x in 0..self.grid.width {
                let piece = self.grid.get(x, y);
                let char = match piece {
                    Piece::Red => "0",
                    Piece::Yellow => "O",
                    Piece::Empty => {
                        let red_threat = self.threats.get(&Piece::Red).unwrap().contains(&(x, y));
                        let yellow_threat = self.threats.get(&Piece::Yellow).unwrap().contains(&(x, y));
                        match (red_threat, yellow_threat) {
                            (true, true) => "B",
                            (true, false) => "R",
                            (false, true) => "Y",
                            (false, false) => " "
                        }
                    }
                };
                write!(f, "{} ", char)?;
            }
            writeln!(f, "|")?;
        }
        Ok(())
    }
}

fn main() {
    let mut board = Board::new();

    // let moves = vec![0, 1, 2, 2, 1, 3, 2, 3, 4, 3];
    let moves = vec![0, 1, 1, 3, 6, 3, 6, 3];
    for mv in moves {
        board.drop(mv);
    }

    loop {
        print!("{}", board);
        println!("{} move:", board.next_move);
        let mut ply_move = String::new();
        stdin().read_line(&mut ply_move).unwrap();
        match ply_move.trim().parse::<usize>() {
            Err(_) => {
                println!("Type a number");
                continue;
            },
            Ok(move_x) => {
                if move_x >= board.grid.width {
                    println!("Type a number from 0 - {}", board.grid.width-1);
                    continue;
                } else if board.drop_zones[move_x] <= 0 {
                    println!("Can't move there");
                    continue;
                } else {
                    board.drop(move_x);
                }
            }
        }
        match board.winner {
            Some(winner) => {
                println!("{} wins!", winner);
                break;
            },
            None => ()
        };
        if board.drop_zones.iter().all(|x| *x == 0usize) {
            println!("Tie, nobody wins");
            break;
        }
    }
}