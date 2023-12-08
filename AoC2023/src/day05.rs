use std::{num::ParseIntError, str::FromStr};

type Seeds = Vec<u64>;
fn parse_seeds(section: &str) -> Result<Seeds, ParseIntError> {
    section
        .trim_start_matches("seeds: ")
        .split_whitespace()
        .try_fold(Seeds::new(), |mut acc, numstr| {
            acc.push(numstr.parse()?);
            Ok(acc)
        })
}

// TODO: explore
//      this could be optimized using a once computed lookup array
struct MapEntry {
    dest_range_start: u64,
    src_range_start: u64,
    range_len: u64,
}
impl MapEntry {
    fn contains_src(&self, src: u64) -> bool {
        (self.src_range_start..self.src_range_start + self.range_len).contains(&src)
    }
    fn map(&self, src: u64) -> Option<u64> {
        if self.contains_src(src) {
            Some(src + self.range_len)
        } else {
            None
        }
    }
}
impl FromStr for MapEntry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut values = s.split_whitespace();

        match values
            .try_fold(vec![], |mut acc, item| {
                acc.push(item.parse()?);
                Ok::<Vec<u64>, ParseIntError>(acc)
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

struct Map(Vec<MapEntry>);
impl Map {
    fn map(&self, src: u64) -> u64 {
        self.0
            .iter()
            .find_map(|entry| entry.map(src))
            .unwrap_or(src)
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

    let seeds = parse_seeds(sections.next().unwrap());
    let seed_to_soil = Map::parse_section(sections.next().unwrap());
    let soil_to_fertilizer = Map::parse_section(sections.next().unwrap());
    let fertilizer_to_water = Map::parse_section(sections.next().unwrap());
    let water_to_light = Map::parse_section(sections.next().unwrap());
    let light_to_temp = Map::parse_section(sections.next().unwrap());
    let temp_to_humid = Map::parse_section(sections.next().unwrap());
    let humid_to_location = Map::parse_section(sections.next().unwrap());
}
