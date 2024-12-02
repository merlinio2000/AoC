use itertools::Itertools;
use nom::{
    IResult,
    character::complete::{digit1, line_ending, space1},
    combinator::map_res,
    multi::separated_list1,
    sequence::separated_pair,
};
use std::str::FromStr;

const INPUT: &str = include_str!("week1.input").trim_ascii_end();
type Num = u32;

fn decimal_number(input: &str) -> IResult<&str, Num> {
    map_res(digit1, Num::from_str)(input)
}
fn id_pair(input: &str) -> IResult<&str, (Num, Num)> {
    separated_pair(decimal_number, space1, decimal_number)(input)
}
fn parsed() -> (Vec<Num>, Vec<Num>) {
    let (remainder, parsed) =
        separated_list1(line_ending, id_pair)(INPUT).expect("failed to parse input");
    debug_assert_eq!(remainder, "");
    parsed.into_iter().unzip()
}

fn part1(sorted1: &[Num], sorted2: &[Num]) {
    let distance: Num = sorted1
        .iter()
        .zip_eq(sorted2)
        .map(|(left, right)| left.abs_diff(*right))
        .sum();

    println!("total distance is {distance}");
}

fn part2(sorted1: &[Num], sorted2: &[Num]) {
    let mut iter2 = sorted2.iter().peekable();
    let mut skip_values_lower_than = |to: Num| iter2.by_ref().skip_while(|id2| **id2 < to);
    let mut count_repetitions_of = |val: Num| iter2.peeking_take_while(|id2| **id2 == val).count();
    let similarity: usize = sorted1
        .iter()
        .map(|id1| {
            skip_values_lower_than(*id1);
            *id1 as usize * count_repetitions_of(*id1)
        })
        .sum();
}

fn main() {
    let (mut list1, mut list2) = parsed();
    list1.sort_unstable();
    list2.sort_unstable();
    let (sorted1, sorted2) = (list1, list2);

    part1(&sorted1, &sorted2);
}
