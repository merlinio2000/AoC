use std::{
    cmp::{max, min},
    num::ParseIntError,
    ops::Range,
    str::FromStr,
};

use itertools::Itertools;

type Id = i64;
type SeedRange = Range<Id>;

fn parse_seed_ranges<'a>(section: &str) -> Result<Vec<SeedRange>, String> {
    let mut items = section.trim_start_matches("seeds: ").split_whitespace();

    let mut result = Vec::with_capacity(items.size_hint().0 / 2);

    while let Some(item) = items.next() {
        let other_item = items.next().ok_or(format!(
            "uneven amount of items, expected one more following '{item}'"
        ))?;

        let range_start = item.parse::<Id>().map_err(|e| e.to_string())?;
        let range_len = other_item.parse::<Id>().map_err(|e| e.to_string())?;
        result.push(range_start..(range_start + range_len));
    }

    Ok(result)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct RangeMapping {
    src: Id,
    dest: Id,
    len: Id,
}
impl RangeMapping {
    fn from_len(src: Id, dest: Id, len: Id) -> Self {
        debug_assert!(len > 0);
        Self { src, dest, len }
    }

    /// gaps are mapped to themselves (default)
    fn dflt_from_bounds(src: Id, src_end_excl: Id) -> Self {
        // debug_assert!(src_end_excl > src);
        if src_end_excl <= src {
            dbg!(src);
            dbg!(src_end_excl);
            panic!("invalid range");
        }
        Self {
            src,
            dest: src,
            len: src_end_excl - src,
        }
    }
    /// default ([Self::dflt]) for the upper range (up to [Id::MAX])
    fn upper_dflt(src: Id) -> Self {
        Self::dflt_from_bounds(src, Id::MAX)
    }

    /// upper bounds is exclusive
    fn src_end_excl(&self) -> Id {
        self.src + self.len
    }
    /// upper bounds is exclusive
    fn dest_end_excl(&self) -> Id {
        self.dest + self.len
    }

    /// see excalidraw
    fn self_dest_overlaps_other_src(&self, other: &Self) -> Option<RangeOverlap> {
        // 0,5 : 3
        // 6,12 : 1
        // -> 6 : 1
        // # start = max(5,6) = 6
        // # end = min(5+3,6+1) = 7
        //
        // 0,5 : 3
        // 10,15 : 3
        // -> None
        // # start = max(5, 10) = 10
        // # end = min(5+3,10+3) = 8
        //
        // 10,0 : 5
        // 3,20 : 4
        // -> 3 : 2
        // # start = max(0, 3) = 3
        // # end = min(0+5, 3+4) = 5
        let start = max(self.dest, other.src);
        let end = min(self.dest_end_excl(), other.src_end_excl());

        let len = end - start;

        if len > 0 {
            Some(RangeOverlap { start, len })
        } else {
            None
        }
    }

    /// Creates a new [Self] based on the overlap between `self`'s dest
    /// and `other`'s src
    /// The new instance will have a (sub)range of `self.src` as src
    /// and a (sub)range of `other.dest` as dest
    /// Parameters:
    /// - [self]: provides `dest` for the overlap
    /// - [other]: provides `src` for the overlap
    fn merge_with_overlap(&self, other: &Self) -> Option<Self> {
        if let Some(RangeOverlap {
            start: overlap_start,
            len: overlap_len,
        }) = self.self_dest_overlaps_other_src(other)
        {
            let self_src_to_dest_offset = self.dest - self.src;
            let other_dest_to_src_offset = other.src - other.dest;
            let overlap_start_in_self_src = overlap_start - self_src_to_dest_offset;
            let overlap_start_in_other_dest = overlap_start - other_dest_to_src_offset;

            if overlap_start_in_self_src < 0 || overlap_start_in_other_dest < 0 {
                dbg!(self);
                dbg!(other);
                dbg!(overlap_start);
                dbg!(overlap_len);
                dbg!(overlap_start_in_self_src);
                dbg!(overlap_start_in_other_dest);
                panic!("invalid merged range");
            }

            Some(Self {
                src: overlap_start_in_self_src,
                dest: overlap_start_in_other_dest,
                len: overlap_len,
            })
        } else {
            None
        }
    }
}

impl PartialEq<(Range<Id>, Range<Id>)> for RangeMapping {
    fn eq(&self, other: &(Range<Id>, Range<Id>)) -> bool {
        self.src == other.0.start
            && self.dest == other.1.start
            && self.src_end_excl() == other.0.end
            && self.dest_end_excl() == other.1.end
    }
}

impl FromStr for RangeMapping {
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
                dest: *first,
                src: *second,
                len: *third,
            }),
            Ok(_) => Err(format!("expected 3 uints but got '{}'", s)),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// ASSUMES vec to be sorted by input range start
/// format: `[(in_range, out_range)]`
///
/// **!!ONLY!!** for instances created using [RangeMap::from_fill_gaps]:
/// All values in the range `0..`[Id::MAX] are mapped in this structure
/// see [RangeMap::from_fill_gaps]
#[derive(PartialEq, Eq, Debug)]
struct RangeMap(Vec<RangeMapping>);

impl RangeMap {
    fn from_fill_gaps(value: impl Iterator<Item = RangeMapping>) -> Self {
        let value = value.sorted_by_key(|mapping| mapping.src);
        // *4 guesstimate pulled straight out of my ass
        let mut result = Vec::with_capacity(value.len() * 4);

        // fill gaps
        let mut start = 0;
        for mapping in value {
            if mapping.src != start {
                // gaps are mapped to themselves
                result.push(RangeMapping::dflt_from_bounds(start, mapping.src));
            }
            start = mapping.src_end_excl();
            result.push(mapping);
        }

        if start != Id::MAX {
            // also fill the gap "above" the defined ranges
            result.push(RangeMapping::upper_dflt(start));
        }

        result.shrink_to_fit();
        Self(result)
    }

    /// seeds can be represented as a [RangeMap] that maps the
    /// seed-ranges to themselves
    fn from_seeds(seed_ranges: impl Iterator<Item = SeedRange>) -> Self {
        Self(
            seed_ranges
                .into_iter()
                .sorted_by_key(|e| e.start)
                .map(|e| RangeMapping::dflt_from_bounds(e.start, e.end))
                .collect(),
        )
    }
}

impl RangeMap {
    /// Joins like: self(inner(x)) -> output(x)
    /// This means that the resulting map, maps `src` ranges from inner
    /// to `dest` ranges of self
    fn left_join(&self, inner: &RangeMap) -> RangeMap {
        let outer = self;
        let result = inner
            .0
            .iter()
            .flat_map(|inner_range| {
                outer
                    .0
                    .iter()
                    .filter_map(|outer_range| inner_range.merge_with_overlap(outer_range))
            })
            .collect();
        RangeMap(result)
    }

    // TODO: can be optimized because arrays are sorted by in.start
    fn _find_in_overlapping_with_out<'a, 'b: 'a>(
        &'a self,
        out_overlap_with: &'b RangeMapping,
    ) -> impl Iterator<Item = (&'a RangeMapping, RangeOverlap)> {
        self.0.iter().filter_map(move |range| {
            range
                .self_dest_overlaps_other_src(out_overlap_with)
                .map(|overlap| (range, overlap))
        })
    }
}

struct RangeOverlap {
    start: Id,
    len: Id,
}

struct Map(Vec<RangeMapping>);
impl Map {
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

impl Into<RangeMap> for Map {
    fn into(self) -> RangeMap {
        RangeMap::from_fill_gaps(self.0.into_iter())
    }
}

const INPUT: &'static str = include_str!("./day05_input.txt");
pub fn main() {
    let emptyline_re = regex::Regex::new(r"(?m)^\n").unwrap();
    let mut sections = emptyline_re.split(INPUT);

    let seed_ranges = parse_seed_ranges(sections.next().unwrap()).unwrap();

    let seed_to_soil = Map::parse_section(sections.next().unwrap());
    let soil_to_fertilizer = Map::parse_section(sections.next().unwrap());
    let fertilizer_to_water = Map::parse_section(sections.next().unwrap());
    let water_to_light = Map::parse_section(sections.next().unwrap());
    let light_to_temp = Map::parse_section(sections.next().unwrap());
    let temp_to_humid = Map::parse_section(sections.next().unwrap());
    let humid_to_location = Map::parse_section(sections.next().unwrap());

    let reducer_chain: [RangeMap; 7] = [
        seed_to_soil,
        soil_to_fertilizer,
        fertilizer_to_water,
        water_to_light,
        light_to_temp,
        temp_to_humid,
        humid_to_location,
    ]
    .map(|e| e.into());

    let dummy_seed_map = RangeMap::from_seeds(seed_ranges.into_iter());

    let final_map = reducer_chain
        .iter()
        .fold(dummy_seed_map, |acc, curr_map| curr_map.left_join(&acc));

    // let debug = final_map
    //     .0
    //     .iter()
    //     .sorted_by_key(|e| e.dest)
    //     .take(20)
    //     .collect_vec();
    //
    // dbg!(debug);

    // find the lowest start of any output range
    let best_possible_result = final_map
        .0
        .iter()
        .min_by_key(|e| e.dest)
        .expect("mapping should not be empty")
        .dest;
    dbg!(best_possible_result);
}

#[cfg(test)]
mod test {
    use crate::day05::RangeMapping;

    use super::Id;
    use super::RangeMap;
    #[test]
    fn create_range_map_gaps() {
        let ranges = [
            RangeMapping::from_len(20, 3, 3),
            RangeMapping::from_len(5, 10, 10),
        ];
        let ranges2 = [
            RangeMapping::from_len(0, 2, 2),
            RangeMapping::from_len(2, 100, 2),
            RangeMapping::from_len(4, 10, 2),
        ];
        let created = RangeMap::from_fill_gaps(ranges.into_iter());
        let created2 = RangeMap::from_fill_gaps(ranges2.into_iter());

        assert_eq!(created.0.len(), 5);
        debug_assert_eq!(created.0[0], (0..5, 0..5));
        debug_assert_eq!(created.0[1], (5..15, 10..20));
        debug_assert_eq!(created.0[2], (15..20, 15..20));
        debug_assert_eq!(created.0[3], (20..23, 3..6));
        debug_assert_eq!(created.0[4], (23..Id::MAX, 23..Id::MAX));

        assert_eq!(created2.0.len(), 4);
        debug_assert_eq!(created2.0[0], (0..2, 2..4));
        debug_assert_eq!(created2.0[1], (2..4, 100..102));
        debug_assert_eq!(created2.0[2], (4..6, 10..12));
        debug_assert_eq!(created2.0[3], (6..Id::MAX, 6..Id::MAX));
    }

    #[test]
    fn create_range_map_seeds() {
        let seed_ranges = vec![10..15, 0..2, 5..8];

        let created = RangeMap::from_seeds(seed_ranges.into_iter());

        assert_eq!(created.0.len(), 3);
        debug_assert_eq!(created.0[0], (0..2, 0..2));
        debug_assert_eq!(created.0[1], (5..8, 5..8));
        debug_assert_eq!(created.0[2], (10..15, 10..15));
    }

    #[test]
    fn merge_with_overlap() {
        let r1 = RangeMapping::from_len(0, 5, 3);
        let r2 = RangeMapping::from_len(5, 10, 5);
        let r3 = RangeMapping::from_len(2, 0, 2);

        debug_assert_eq!(r1.merge_with_overlap(&r1), None);
        debug_assert_eq!(
            r1.merge_with_overlap(&r2),
            Some(RangeMapping::from_len(0, 10, 3))
        );
        debug_assert_eq!(r1.merge_with_overlap(&r3), None)
    }

    #[test]
    fn left_join_seeds() {
        let seed_ranges = vec![10..15, 0..2, 5..8];
        let ranges = [
            RangeMapping::from_len(20, 3, 3),
            RangeMapping::from_len(5, 10, 2),
            RangeMapping::from_len(8, 80, 3),
            RangeMapping::from_len(11, 110, 1),
            RangeMapping::from_len(14, 140, 3),
        ];

        let seeds = RangeMap::from_seeds(seed_ranges.into_iter());
        let created = RangeMap::from_fill_gaps(ranges.into_iter());

        let joined = created.left_join(&seeds);
        debug_assert_eq!(
            joined,
            RangeMap(vec![
                RangeMapping::dflt_from_bounds(0, 2),
                RangeMapping::from_len(5, 10, 2),
                RangeMapping::dflt_from_bounds(7, 8),
                RangeMapping::from_len(10, 82, 1),
                RangeMapping::from_len(11, 110, 1),
                RangeMapping::dflt_from_bounds(12, 14),
                RangeMapping::from_len(14, 140, 1),
            ])
        )
    }
}
