use core::panic;
use itertools::Itertools;
use nom::{
    IResult,
    character::complete::{digit1, line_ending, space1},
    combinator::map_res,
    multi::separated_list1,
};
use std::{
    cmp::{Ordering, min},
    str::FromStr,
};

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

    println!("PART1: total safe count is {safe_count}");
    safe_count
}

fn part2_simple(list: &[Num]) -> bool {
    if list_is_monotonic_and_no_repetitions(list) {
        return true;
    }
    let cleaned_len = list.len() - 1;
    let mut cleaned_vec = vec![0; cleaned_len];
    for index_to_remove in 0..list.len() {
        for (index, value) in list.iter().enumerate() {
            match index.cmp(&index_to_remove) {
                Ordering::Greater => cleaned_vec[index - 1] = *value,
                Ordering::Equal => { /* ignore this element */ }
                Ordering::Less => cleaned_vec[index] = *value,
            }
        }
        if list_is_monotonic_and_no_repetitions(cleaned_vec.as_slice()) {
            return true;
        }
    }
    false
}

fn compare_solutions(list: &[Num]) -> bool {
    let probably_correct = part2_simple(list);
    let trying_to_be_smart = list_is_monotonic_and_no_repetitions_with_one_removal(list);
    if probably_correct == trying_to_be_smart {
        trying_to_be_smart
    } else {
        println!(
            "INPUT: {list:?}
mismatch between the simple={probably_correct} and fancy={trying_to_be_smart} result"
        );
        probably_correct
    }
}

fn list_is_monotonic_and_no_repetitions_with_one_removal(list: &[Num]) -> bool {
    debug_assert!(list.len() > 3);

    let differences = list
        .iter()
        .tuple_windows()
        .map(|(left, right)| left - right)
        .collect_vec();
    // TODO: is this beeing auto-vectorized? could be fun to manually do
    let (num_neg, num_0, num_pos) = (
        differences.iter().filter(|diff| **diff < 0).count(),
        differences.iter().filter(|diff| **diff == 0).count(),
        differences.iter().filter(|diff| **diff > 0).count(),
    );
    if num_0 > 1 || min(num_neg, num_pos) > 1 {
        return false;
    }

    match num_neg.cmp(&num_pos) {
        Ordering::Less => {
            // input: [8, 4, 2, 1]
            // diffs: [4, 2, 1]
            // -------------------
            // input: [4, 8, 2, 1]
            // diffs: [-4, 6, 1]
            // -------------------
            // input: [4, 2, 8, 1]
            // diffs: [2, -6, 7]
            // -------------------
            // input: [4, 2, 1, 8]
            // diffs: [2, 1, -7]
            // -------------------
            // input: [87, 86, 87, 86, 83]
            // diffs: [1, -1, 1, 3]

            // println!(
            //     "DESC -
            //     input: {list:?}
            //     diffs: {differences:?}",
            // );
            let expected_range = 1..=3;
            let mut carry = 0;
            let mut had_error = false;
            for diff in differences {
                // println!("diff={diff}, carry={carry}");
                if diff < 0 {
                    carry = -diff;
                    if had_error {
                        return false;
                    } else {
                        had_error = true;
                    }
                } else if !expected_range.contains(&(diff - carry)) {
                    carry = diff;
                    if had_error {
                        return false;
                    } else {
                        had_error = true;
                    }
                } else {
                    carry = 0;
                }
            }
            true
        }
        Ordering::Greater => {
            // input: [8, 1, 2, 4]
            // diffs: [7, -1, -2]
            // -------------------
            // input: [1, 8, 2, 4]
            // diffs: [-7, 6, -2]
            // -------------------
            // input: [1, 2, 8, 4]
            // diffs: [-1, -6, 4]
            // -------------------
            // input: [1, 2, 4, 8]
            // diffs: [-1, -2, -4]
            // -------------------
            // input: [1, 2, 8, 9]
            // diffs: [-1, -6, -1]
            // -------------------
            // INPUT: [78, 81, 83, 84, 83, 84]
            // diffs: [-3, -2, -1, 1, -1]

            // Buggy - we dont correctly handle mismatches in the first
            // value (there should be no carry then since there is no gap to be
            // bridged)
            // INPUT: [7, 4, 6, 7, 8, 10, 13, 15]
            // diffs: [3, -2, -1, -1, -2, -3, -2]

            // println!(
            //     "ASC -
            //     input: {list:?}
            //     diffs: {differences:?}",
            // );
            let expected_range = -3..=-1;
            let mut carry = 0;
            let mut had_error = false;
            for diff in differences {
                // println!("diff={diff}, carry={carry}");
                if !expected_range.contains(&(diff + carry)) {
                    carry = diff;
                    if had_error {
                        return false;
                    } else {
                        had_error = true;
                    }
                } else {
                    carry = 0;
                }
            }
            true
        }
        Ordering::Equal => panic!("this should not be possible"),
    }
}

fn part2(lists: &[Vec<Num>]) -> usize {
    let safe_count = lists.iter().filter(|list| compare_solutions(list)).count();

    println!("PART2: total safe count is {safe_count}");
    safe_count
}

fn main() {
    let lists = parse(INPUT);

    part1(&lists);
    part2(&lists);
}

#[cfg(test)]
mod test {
    use crate::{
        list_is_monotonic_and_no_repetitions,
        list_is_monotonic_and_no_repetitions_with_one_removal, part2,
    };

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
    fn example1() {
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

    #[test]
    fn example2() {
        let lists = parse(INPUT);
        itertools::assert_equal(
            [true, false, false, true, true, true],
            lists
                .iter()
                .map(|list| list_is_monotonic_and_no_repetitions_with_one_removal(list)),
        );
        let got = part2(&lists);
        assert_eq!(got, 4);
    }
}
