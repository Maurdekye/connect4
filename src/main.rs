use rand::seq::SliceRandom;
use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
    io::stdin,
};

trait Search {
    type Score: Ord + Eq + Copy;
    type Iter: Iterator<Item = Self>;

    fn score(&self) -> Self::Score;
    fn moves(&self) -> Self::Iter;
    fn game_over(&self) -> bool;
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct Grid<T> {
    _data: Vec<T>,
    width: usize,
    height: usize,
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
    Empty,
}

impl Piece {
    fn opponent(&self) -> Self {
        match self {
            Piece::Red => Piece::Yellow,
            Piece::Yellow => Piece::Red,
            Piece::Empty => Piece::Empty,
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Piece::Red => write!(f, "Red")?,
            Piece::Yellow => write!(f, "Yellow")?,
            Piece::Empty => write!(f, "Empty")?,
        };
        Ok(())
    }
}

type Position = (usize, usize);

#[derive(Clone, Eq)]
struct Board {
    grid: Grid<Piece>,
    drop_zones: Vec<usize>,
    threats: Vec<(Position, Piece)>,
    winner: Option<Piece>,
    next_move: Piece,
    show_threats: bool,
}

struct Tallies {
    red: Vec<Position>,
    yellow: Vec<Position>,
    empty: Vec<Position>,
}

impl Tallies {
    fn get(&mut self, piece: &Piece) -> &mut Vec<Position> {
        match piece {
            Piece::Red => &mut self.red,
            Piece::Yellow => &mut self.yellow,
            Piece::Empty => &mut self.empty,
        }
    }
}

impl Board {
    fn new_with_size(width: usize, height: usize) -> Board {
        Board {
            grid: Grid::new(width, height, Piece::Empty),
            drop_zones: vec![height; width],
            threats: Vec::new(),
            winner: None,
            next_move: Piece::Yellow,
            show_threats: false,
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
        if self.threats.contains(&((x, y), piece)) {
            self.winner = Some(piece);
        }
        self.threats.retain(|&(pos, _)| pos != (x, y));
        self.update(x, y);
    }

    fn drop(&mut self, x: usize) -> Option<usize> {
        if self.drop_zones[x] > 0 {
            self.drop_zones[x] -= 1;
            let drop_spot = self.drop_zones[x];
            self.set(x, drop_spot, self.next_move);
            self.next_move = self.next_move.opponent();
            Some(drop_spot)
        } else {
            None
        }
    }

    fn update(&mut self, x: usize, y: usize) {
        // let pos_sets = vec![(-1, -1), (-1, 0), (-1, 1), (0, 1)]
        //     .iter()
        //     .map(|(dx, dy)| {
        //         (-3..=3)
        //             .map(|d| ((x as i32) + d * dx, (y as i32) + d * dy))
        //             .filter(|(px, py)| {
        //                 *px >= 0
        //                     && *py >= 0
        //                     && *px < self.grid.width as i32
        //                     && *py < self.grid.height as i32
        //             })
        //             .map(|(px, py)| (px as usize, py as usize))
        //             .map(|(px, py)| ((px, py), self.grid.get(px, py)))
        //             .collect::<Vec<_>>()
        //     })
        //     .filter(|poses| poses.len() >= 4);
        // .collect::<Vec<_>>();

        // let all_poses = pos_sets.iter().flatten().map(|(pos, _)| pos).collect::<HashSet<_>>();
        // self.print_with_pos_set(&all_poses);

        for poses in vec![(-1, -1), (-1, 0), (-1, 1), (0, 1)]
            .iter()
            .map(|(dx, dy)| {
                (-3..=3)
                    .map(|d| ((x as i32) + d * dx, (y as i32) + d * dy))
                    .filter(|(px, py)| {
                        *px >= 0
                            && *py >= 0
                            && *px < self.grid.width as i32
                            && *py < self.grid.height as i32
                    })
                    .map(|(px, py)| (px as usize, py as usize))
                    .map(|(px, py)| ((px, py), self.grid.get(px, py)))
                    .collect::<Vec<_>>()
            })
            .filter(|poses| poses.len() >= 4)
        {
            // self.print_with_pos_set(&poses.iter().map(|(pos, _)| pos).collect());
            let mut tallies = Tallies {
                red: Vec::new(),
                yellow: Vec::new(),
                empty: Vec::new(),
            };

            for i in 0..poses.len() {
                let (pos, piece) = poses[i];
                tallies.get(piece).push(pos);
                if i >= 4 {
                    let (early_pos, early_piece) = poses[i - 4];
                    tallies.get(early_piece).retain(|&pos| pos != early_pos);
                }
                if i >= 3 {
                    if tallies.empty.len() == 1 {
                        let gap = tallies.empty.first().unwrap().clone();
                        for piece in vec![Piece::Red, Piece::Yellow] {
                            if tallies.get(&piece).len() == 3 {
                                self.threats.push((gap, piece));
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
                            let red_threat = self.threats.contains(&((sx, sy), Piece::Red));
                            let yellow_threat = self.threats.contains(&((sx, sy), Piece::Yellow));
                            match (red_threat, yellow_threat) {
                                (true, true) => "B",
                                (true, false) => "R",
                                (false, true) => "Y",
                                (false, false) => " ",
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

impl Search for Board {
    type Score = i32;
    type Iter = BoardMoveIterator;

    fn score(&self) -> i32 {
        match self.winner {
            Some(piece) => {
                if piece == Piece::Yellow {
                    return i32::MIN;
                } else {
                    return i32::MAX;
                }
            }
            None => (),
        }
        self.threats
            .iter()
            .map(|&((_, y), piece)| {
                if y == self.grid.height - 1 {
                    0
                } else {
                    match piece {
                        Piece::Red => 2i32.pow(y as u32),
                        Piece::Yellow => -2i32.pow(y as u32),
                        _ => 0,
                    }
                }
            })
            .sum()
    }

    fn moves(&self) -> BoardMoveIterator {
        BoardMoveIterator {
            board: self.clone(),
            move_index: 0,
        }
    }

    fn game_over(&self) -> bool {
        self.winner.is_some() || self.drop_zones.iter().all(|x| *x == 0)
    }
}

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
                        if self.show_threats {
                            let red_threat = self.threats.contains(&((x, y), Piece::Red));
                            let yellow_threat = self.threats.contains(&((x, y), Piece::Yellow));
                            match (red_threat, yellow_threat) {
                                (true, true) => "B",
                                (true, false) => "R",
                                (false, true) => "Y",
                                (false, false) => " ",
                            }
                        } else {
                            " "
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

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.grid == other.grid && self.next_move == other.next_move
    }
}

impl std::hash::Hash for Board {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.grid.hash(state);
        self.next_move.hash(state);
    }
}

struct BoardMoveIterator {
    board: Board,
    move_index: usize,
}

impl Iterator for BoardMoveIterator {
    type Item = Board;

    fn next(&mut self) -> Option<Self::Item> {
        while self.move_index < self.board.drop_zones.len()
            && self.board.drop_zones[self.move_index] == 0
        {
            self.move_index += 1;
        }
        if self.move_index < self.board.drop_zones.len() {
            let mut new_board = self.board.clone();
            new_board.drop(self.move_index);
            self.move_index += 1;
            Some(new_board)
        } else {
            None
        }
    }
}

fn minimax<T: Search>(
    state: &T,
    depth: usize,
    alpha: Option<T::Score>,
    beta: Option<T::Score>,
    maximizing: bool,
) -> T::Score {
    let omax = |a: Option<T::Score>, b: T::Score| a.map_or(Some(b), |v| Some(v.max(b)));
    let omin = |a: Option<T::Score>, b: T::Score| a.map_or(Some(b), |v| Some(v.min(b)));
    // let mut hasher = DefaultHasher::new();
    // state.hash(&mut hasher);
    // let hash = hasher.finish();
    // println!("Parent {} at depth {} with score {}:", hash, depth, state.score());
    // println!("{}", state);
    if depth == 0 || state.game_over() {
        state.score()
    } else if maximizing {
        let mut max_eval: Option<T::Score> = None;
        let mut new_alpha: Option<T::Score> = alpha;
        for child in state.moves() {
            // println!("Max child of {} with score {}:", hash, child.score());
            // println!("{}", child);
            let child_score = minimax(&child, depth - 1, new_alpha, beta, false);
            max_eval = omax(max_eval, child_score);
            new_alpha = omax(new_alpha, child_score);
            // if beta.map_or(true, |b| new_alpha.map_or(true, |a| b <= a)) {
            //     break;
            // }
        }
        max_eval.unwrap_or_else(|| state.score())
    } else {
        let mut min_eval: Option<T::Score> = None;
        let mut new_beta = beta;
        for child in state.moves() {
            // println!("Min child of {} with score {}:", hash, child.score());
            // println!("{}", child);
            let child_score = minimax(&child, depth - 1, alpha, new_beta, true);
            min_eval = omin(min_eval, child_score);
            new_beta = omin(new_beta, child_score);
            // if new_beta.map_or(true, |b| alpha.map_or(true, |a| b <= a)) {
            //     break;
            // }
        }
        min_eval.unwrap_or_else(|| state.score())
    }
}

fn main() {
    let mut board = Board::new();
    board.show_threats = true;

    // let moves = vec![0, 1, 2, 2, 1, 3, 2, 3, 4, 3];
    // let moves = vec![0, 1, 1, 3, 6, 3, 6, 3];
    // let moves = vec![3, 3, 3, 6, 5, 5, 2, 6, 2];
    // for mv in moves {
    //     board.drop(mv);
    // }

    loop {
        print!("{}", board);
        if board.next_move == Piece::Yellow {
            println!("{} move:", board.next_move);
            let mut ply_move = String::new();
            stdin().read_line(&mut ply_move).unwrap();
            match ply_move.trim().parse::<usize>() {
                Err(_) => {
                    println!("Type a number");
                    continue;
                }
                Ok(move_x) => {
                    if move_x >= board.grid.width {
                        println!("Type a number from 0 - {}", board.grid.width - 1);
                        continue;
                    } else if board.drop_zones[move_x] <= 0 {
                        println!("Can't move there");
                        continue;
                    } else {
                        board.drop(move_x);
                    }
                }
            }
        } else {
            let moves_scores = board
                .moves()
                .into_iter()
                .map(|b| (minimax(&b, 5, None, None, false), b))
                .collect::<Vec<_>>();
            for (score, _) in moves_scores.iter() {
                print!("{}, ", score);
            }
            println!();
            if moves_scores.is_empty() {
                println!("Tie, nobody wins (this should never occur)");
                break;
            }
            let max_score = moves_scores.iter().map(|(s, _)| s).max().unwrap();
            let max_scores = moves_scores
                .iter()
                .filter(|(s, _)| s == max_score)
                .map(|(_, b)| b)
                .collect::<Vec<_>>();
            let new_board = *max_scores.choose(&mut rand::thread_rng()).unwrap();
            board = new_board.clone();
        }
        match board.winner {
            Some(winner) => {
                println!("{} wins!", winner);
                break;
            }
            None => (),
        };
        if board.drop_zones.iter().all(|x| *x == 0usize) {
            println!("Tie, nobody wins");
            break;
        }
    }
}
