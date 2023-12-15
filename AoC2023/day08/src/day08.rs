use std::str::Lines;

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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Location(u16);

impl Location {
    fn is_goal(&self) -> bool {
        self.0 == 26u16.pow(3) - 1
    }
}

impl TryFrom<&str> for Location {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() == 3 && value.chars().all(|c| c.is_ascii_uppercase()) {
            let bytes = value.as_bytes();
            let mut res = 0u16;
            res += (bytes[0] - b'A') as u16; // * 26^0
            res += ((bytes[1] - b'A') as u16) * 26u16; // ^1
            res += ((bytes[2] - b'A') as u16) * 26u16.pow(2);
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
    fn go(&self, dir: Direction) -> &Location {
        match dir {
            Direction::Left => &self.left,
            Direction::Right => &self.right,
        }
    }
}

struct Crossings(Vec<Crossing>);

impl Crossings {
    fn new(mut v: Vec<Crossing>) -> Self {
        v.sort_by_key(|c| c.src);
        Self(v)
    }

    fn start(&self) -> Option<&Location> {
        self.0.first().map(|c| &c.src)
    }

    fn go(&self, from: &Location, dir: Direction) -> Result<&Location> {
        let crossing_idx = self
            .0
            .binary_search_by_key(from, |c| c.src)
            .map_err(|_| anyhow!("unable to find {from:?} in the crossings"))?;

        Ok(self.0[crossing_idx].go(dir))
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

    debug_assert_eq!(lines.next(), Some(""));

    let crossings = parse_crossings(lines)?;

    let mut curr_loc = crossings.start().context("crossings have no start")?;
    assert_eq!(curr_loc.0, 0);

    let steps = path
        .into_iter()
        .take_while_inclusive(|dir| {
            curr_loc = crossings.go(curr_loc, *dir).unwrap();
            !curr_loc.is_goal()
        })
        .count();

    println!("steps: {steps}");

    Ok(())
}

mod test {}
