use std::{cmp::min, num::ParseIntError, ops::Range, str::FromStr};

use itertools::Itertools;

type Id = i64;
type Seeds = Vec<Id>;
fn parse_seeds(section: &str) -> Result<Seeds, ParseIntError> {
    section
        .trim_start_matches("seeds: ")
        .split_whitespace()
        .try_fold(Seeds::new(), |mut acc, numstr| {
            acc.push(numstr.parse()?);
            Ok(acc)
        })
}

type SeedRange = Range<Id>;
fn parse_seed_ranges<'a>(
    section: &'a str,
) -> impl Iterator<Item = Result<SeedRange, String>> + 'a + Send {
    let mut items = section.trim_start_matches("seeds: ").split_whitespace();
    std::iter::from_fn(move || {
        match (
            items.next().map(|it| it.parse::<Id>()),
            items.next().map(|it| it.parse::<Id>()),
        ) {
            (Some(Ok(range_start)), Some(Ok(range_len))) => {
                Some(Ok(range_start..(range_start + range_len)))
            }
            (Some(leftover), None) => Some(Err(format!(
                "uneven amount of items, got leftover {leftover:#?}"
            ))),
            (Some(Err(e)), _) => Some(Err(e.to_string())),
            (_, Some(Err(e))) => Some(Err(e.to_string())),
            _ => None,
        }
    })
}

struct MapEntry {
    dest_range_start: Id,
    src_range_start: Id,
    range_len: Id,
}
impl MapEntry {
    fn contains_src(&self, src: Id) -> bool {
        self.in_range().contains(&src)
    }
    fn map(&self, src: Id) -> Option<Id> {
        if self.contains_src(src) {
            Some(self.dest_range_start + (src - self.src_range_start))
        } else {
            None
        }
    }
    fn in_range(&self) -> Range<Id> {
        self.src_range_start..self.src_range_start + self.range_len
    }
    fn out_range(&self) -> Range<Id> {
        self.dest_range_start..self.dest_range_start + self.range_len
    }
}
impl FromStr for MapEntry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut values = s.split_whitespace();

        match values
            .try_fold(vec![], |mut acc, item| {
                acc.push(item.parse()?);
                Ok::<Vec<Id>, ParseIntError>(acc)
            })
            .as_ref()
            .map(|values| &values[..])
        {
            Ok([first, second, third]) => Ok(Self {
                dest_range_start: *first,
                src_range_start: *second,
                range_len: *third,
            }),
            Ok(_) => Err(format!("expected 3 uints but got '{}'", s)),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// ASSUMES vec to be sorted by input range start
/// format: [(in_range, out_range)]
#[derive(Debug)]
struct RangeMap(Vec<(Range<Id>, Range<Id>)>);

impl<T: Iterator<Item = (Range<Id>, Range<Id>)>> From<T> for RangeMap {
    fn from(value: T) -> Self {
        let value = value.sorted_by_key(|(in_range, _)| in_range.start);
        // *4 guesstime pulled straight out of my ass
        let mut result = Vec::with_capacity(value.len() * 4);

        // fill gaps
        let mut start = 0;
        for (in_range, out_range) in value {
            if in_range.start != start {
                // gaps are mapped to themselves
                result.push((start..in_range.start, start..in_range.start));
            }
            start = in_range.end; // exclusive
            result.push((in_range, out_range));
        }

        result.shrink_to_fit();
        Self(result)
    }
}

impl RangeMap {
    /// TODO: this doesn't work as I thought because we need to consider
    /// the default mapping ABOVE the last defined range as well
    /// other idea: start from the top and work on all the produced ranges
    /// step-by-step
    /// should be called from the bottom up
    /// joins like: self(other(x)) -> output(x)
    /// this means that the resulting map, maps IN ranges from other
    /// to OUT ranges of self
    fn left_join(&self, outer: &RangeMap) -> RangeMap {
        let mut result = Vec::with_capacity(self.0.len() + outer.0.len());

        for (outer_in, outer_out) in &outer.0 {
            // TODO: can be optimized because arrays are sorted by in.start
            let overlapping_self = self.0.iter().filter(|(self_in, _)| {
                self_in.start <= outer_out.end && self_in.end >= outer_out.start
            });

            result.extend(overlapping_self.map(|(self_in, self_out)| {
                let overlap_start = min(outer_out.start, self_in.start);
                let overlap_end = min(outer_out.end, self_in.end);
                let overlap_len = overlap_end - overlap_start;

                let outer_in_to_out_offset = outer_out.start - outer_in.start;
                let new_in = (overlap_start - outer_in_to_out_offset)
                    ..(overlap_end - outer_in_to_out_offset + overlap_len);

                let self_in_to_out_offset = self_out.start - self_in.start;
                let new_out = (overlap_start - self_in_to_out_offset)
                    ..(overlap_end - self_in_to_out_offset + overlap_len);

                (new_in, new_out)
            }));
        }
        result.shrink_to_fit();
        RangeMap(result)
    }
}

struct Map(Vec<MapEntry>);
impl Map {
    fn map(&self, src: Id) -> Id {
        self.0
            .iter()
            .find_map(|entry| entry.map(src))
            .unwrap_or(src)
    }

    fn inout_ranges(&self) -> RangeMap {
        RangeMap::from(self.0.iter().map(|e| (e.in_range(), e.out_range())))
    }

    fn parse_section(s: &str) -> Self {
        let mut lines = s.lines().peekable();

        // take first line as header if it doesnt start with a digit
        if let Some(maybe_header) = lines.peek() {
            if maybe_header
                .chars()
                .next()
                .is_some_and(|first_char| !first_char.is_digit(10))
            {
                lines.next().unwrap();
            }
        }

        let res = lines
            .try_fold(vec![], |mut acc, line| {
                acc.push(line.parse()?);
                Ok::<_, String>(acc)
            })
            .expect("invalid section");

        assert_ne!(res.len(), 0);

        Self(res)
    }
}

const INPUT: &'static str = include_str!("./day05_input.txt");
pub fn main() {
    let emptyline_re = regex::Regex::new(r"(?m)^\n").unwrap();
    let mut sections = emptyline_re.split(INPUT);

    let seed_ranges_iter = parse_seed_ranges(sections.next().unwrap());

    let seed_to_soil = Map::parse_section(sections.next().unwrap());
    let soil_to_fertilizer = Map::parse_section(sections.next().unwrap());
    let fertilizer_to_water = Map::parse_section(sections.next().unwrap());
    let water_to_light = Map::parse_section(sections.next().unwrap());
    let light_to_temp = Map::parse_section(sections.next().unwrap());
    let temp_to_humid = Map::parse_section(sections.next().unwrap());
    let humid_to_location = Map::parse_section(sections.next().unwrap());

    let reducer_chain = [
        &seed_to_soil,
        &soil_to_fertilizer,
        &fertilizer_to_water,
        &water_to_light,
        &light_to_temp,
        &temp_to_humid,
        &humid_to_location,
    ]
    .map(|e| e.inout_ranges());

    let final_map = reducer_chain
        .iter()
        .rev()
        .fold(humid_to_location.inout_ranges(), |acc, curr_map| {
            curr_map.left_join(&acc)
        });

    dbg!(final_map);
}

#[cfg(test)]
mod test {
    use std::ops::Range;

    use super::Id;
    use super::RangeMap;
    #[test]
    fn create_range_map() {
        let ranges: [(Range<Id>, Range<Id>); 2] = [(20..23, 3..6), (5..15, 10..20)];

        let created = RangeMap::from(ranges.into_iter());

        assert_eq!(created.0.len(), 4);
        debug_assert_eq!(created.0[0], (0..5, 0..5));
        debug_assert_eq!(created.0[1], (5..15, 10..20));
        debug_assert_eq!(created.0[2], (15..20, 15..20));
        debug_assert_eq!(created.0[3], (20..23, 3..6));
    }
}
