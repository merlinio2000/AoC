use core::panic;
use itertools::Itertools;
use nom::{
    IResult,
    character::complete::{digit1, line_ending, space1},
    combinator::map_res,
    multi::separated_list1,
};
use std::str::FromStr;

const INPUT: &str = include_str!("input.txt").trim_ascii_end();
type Num = i32;

fn decimal_number(input: &str) -> IResult<&str, Num> {
    map_res(digit1, Num::from_str)(input)
}
fn level_list(input: &str) -> IResult<&str, Vec<Num>> {
    separated_list1(space1, decimal_number)(input)
}
fn parse(input: &str) -> Vec<Vec<Num>> {
    let (remainder, parsed) =
        separated_list1(line_ending, level_list)(input).expect("failed to parse input");
    debug_assert_eq!(remainder, "");
    parsed
}

fn list_is_monotonic_and_no_repetitions(list: &[Num]) -> bool {
    let [first, second, ..] = list else {
        panic!("at least two elements in list");
    };
    let initial_difference = first - second;
    match initial_difference.abs() {
        0 => false,
        1..=3 => {
            let initial_is_desc = initial_difference > 0;
            list[1..]
                .iter()
                .tuple_windows()
                .map(|(left, right)| left - right)
                .all(|difference| {
                    initial_is_desc == (difference > 0) && (1..=3).contains(&difference.abs())
                })
        }
        _ => false,
    }
}

fn part1(lists: &[Vec<Num>]) -> usize {
    let safe_count = lists
        .iter()
        .filter(|list| list_is_monotonic_and_no_repetitions(list))
        .count();

    println!("total safe count is {safe_count}");
    safe_count
}

fn main() {
    let lists = parse(INPUT);

    part1(&lists);
}

#[cfg(test)]
mod test {
    use crate::list_is_monotonic_and_no_repetitions;

    use super::{parse, part1};

    const INPUT: &str = "\
7 6 4 2 1
1 2 7 8 9
9 7 6 2 1
1 3 2 4 5
8 6 4 4 1
1 3 6 7 9\
";

    #[test]
    fn example() {
        let lists = parse(INPUT);
        itertools::assert_equal(
            [true, false, false, false, false, true],
            lists
                .iter()
                .map(|list| list_is_monotonic_and_no_repetitions(list)),
        );
        let got = part1(&lists);
        assert_eq!(got, 2);
    }
}
