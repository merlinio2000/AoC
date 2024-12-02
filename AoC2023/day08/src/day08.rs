use std::{fmt::Display, str::Lines};

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Left,
    Right,
}
impl TryFrom<char> for Direction {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'L' => Ok(Direction::Left),
            'R' => Ok(Direction::Right),
            other => Err(anyhow!("invalid direction {other}")),
        }
    }
}

struct Path(Vec<Direction>);
impl IntoIterator for Path {
    type Item = Direction;
    type IntoIter = PathIter;

    fn into_iter(self) -> Self::IntoIter {
        PathIter {
            path: self.0,
            idx: 0,
        }
    }
}

fn parse_path(first_line: &str) -> Result<Path> {
    first_line
        .chars()
        .map(Direction::try_from)
        .try_collect()
        .map(Path)
}

struct PathIter {
    path: Vec<Direction>,
    idx: usize,
}
impl Iterator for PathIter {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        let res = Some(self.path[self.idx]);
        self.idx = (self.idx + 1) % self.path.len();
        res
    }
}

/// since its only 3 letters it can A-Z can be optimized
/// 26^3 possibilities fit into a u16, see [Location::try_from]
/// Goal destinations (**Z) additinoally have their MSB set to one
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Location(u16);

impl Location {
    const Z_FLAG: u16 = 1 << 15;
    /// ends with 'Z'
    fn is_goal(&self) -> bool {
        self.0 & Self::Z_FLAG != 0
    }

    /// ends with 'A'
    fn is_start(&self) -> bool {
        self.0 / 26u16.pow(2) == 0
    }

    fn as_chars(&self) -> [char; 3] {
        let mut val = self.0 & !Self::Z_FLAG;
        let first = val % 26;
        val -= first;
        let second = (val % 26u16.pow(2)) / 26;
        val -= val % 26u16.pow(2);
        let third = val / 26u16.pow(2);

        [first, second, third].map(|u: u16| char::from_u32((u + b'A' as u16).into()).unwrap())
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_chars().into_iter().join(""))
    }
}

impl TryFrom<&str> for Location {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() == 3 && value.chars().all(|c| c.is_ascii_uppercase()) {
            let bytes = value.as_bytes();
            let mut res = 0u16;
            res += (bytes[0] - b'A') as u16;
            res += ((bytes[1] - b'A') as u16) * 26u16; // ^1
            res += ((bytes[2] - b'A') as u16) * 26u16.pow(2);

            if bytes[2] == b'Z' {
                res |= Self::Z_FLAG;
            }

            Ok(Self(res))
        } else {
            Err(anyhow!("invalid location {value}"))
        }
    }
}

struct Crossing {
    src: Location,
    left: Location,
    right: Location,
}
impl Crossing {
    fn go(&self, dir: Direction) -> Location {
        match dir {
            Direction::Left => self.left,
            Direction::Right => self.right,
        }
    }
}

struct Crossings(Vec<Crossing>);

impl Crossings {
    fn new(mut v: Vec<Crossing>) -> Self {
        v.sort_by_key(|c| c.src);
        Self(v)
    }

    fn starts(&self) -> Vec<Location> {
        self.0
            .iter()
            .filter_map(|c| c.src.is_start().then(|| c.src))
            .collect()
    }

    fn go(&self, from: Location, dir: Direction) -> Result<Location> {
        let crossing_idx = self
            .0
            .binary_search_by_key(&from, |c| c.src)
            .map_err(|_| anyhow!("unable to find {from:?} in the crossings"))?;

        Ok(self.0[crossing_idx].go(dir))
    }

    fn paths_to_goals(&self, from) {
        
    }
}

fn parse_crossings(lines: Lines<'_>) -> Result<Crossings> {
    lines
        .map(|line| {
            let (start, lr) = line
                .split_once(" = ")
                .with_context(|| format!("missing ' = ' in  '{line}'"))?;
            let (l, r) = lr[1..lr.len() - 1]
                .split_once(", ")
                .with_context(|| format!("missing ', ' in  '{lr}'"))?;

            Ok(Crossing {
                src: start.try_into()?,
                left: l.try_into()?,
                right: r.try_into()?,
            })
        })
        .try_collect()
        .map(|it| Crossings::new(it))
}

const INPUT: &'static str = include_str!("input.txt");
pub fn main() -> Result<()> {
    let mut lines = INPUT.lines();

    let path = lines.next().context("missing first input line")?;
    let path = parse_path(path)?;

    assert_eq!(lines.next(), Some(""));

    let crossings = parse_crossings(lines)?;

    let mut curr_locs = crossings.starts();

    debug_assert!(curr_locs.iter().all(|l| !l.is_goal()));

    let mut all_finished = true;
    let count = path
        .into_iter()
        .take_while_inclusive(|dir| {
            // println!(
            //     "{}",
            //     curr_locs
            //         .iter()
            //         .map(|l| l.as_chars().iter().join(""))
            //         .join(", ")
            // );
            all_finished = true;
            for curr_loc in &mut curr_locs {
                *curr_loc = crossings.go(*curr_loc, *dir).unwrap();
                all_finished &= curr_loc.is_goal();
            }
            !all_finished
        })
        .count();

    println!("steps: {count}");

    Ok(())
}

mod test {}
