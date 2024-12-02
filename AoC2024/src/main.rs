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

fn part2(sorted1: &[Num], mut sorted2: &[Num]) {
    fn skip_values_lower_than(to: Num, slice: &[Num]) -> &[u32] {
        &slice[slice.iter().position(|id| *id >= to).unwrap_or(slice.len())..]
    }
    fn count_repetitions_of(val: Num, slice: &[u32]) -> usize {
        slice.iter().take_while(|id| **id == val).count()
    }
    sorted2 = skip_values_lower_than(sorted1[0], sorted2);
    let mut last_id_and_count = (sorted1[0], count_repetitions_of(sorted1[0], sorted2));
    let similarity: usize = sorted1
        .iter()
        .map(|id1| {
            if *id1 != last_id_and_count.0 {
                sorted2 = skip_values_lower_than(*id1, sorted2);
                last_id_and_count = (*id1, count_repetitions_of(*id1, sorted2));
            }
            last_id_and_count.0 as usize * last_id_and_count.1
        })
        .sum();

    println!("total similarity score is {similarity}");
}

fn main() {
    let (mut list1, mut list2) = parsed();
    list1.sort_unstable();
    list2.sort_unstable();
    let (sorted1, sorted2) = (list1, list2);

    part1(&sorted1, &sorted2);
    part2(&sorted1, &sorted2);
}
