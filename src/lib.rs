#![feature(inclusive_range_syntax)]

#[macro_use]
extern crate arrayref;

#[macro_use]
extern crate proptest;

use std::vec::Vec;

#[derive(Debug, PartialEq)]
enum OutputSymbol {
    Literal(u8),
    Copy(u8, isize, usize),
}

struct State<T: AsRef<[u8]>> {
    source_indices: std::collections::HashMap<[u8; 3], Vec<usize>>,
    source_data: T,
    target_indices: std::collections::HashMap<[u8; 3], Vec<usize>>,
}

impl<T: AsRef<[u8]> + Default> Default for State<T> {
    fn default() -> State<T> {
        State {
            source_indices: std::collections::HashMap::new(),
            source_data: T::default(),
            target_indices: std::collections::HashMap::new(),
        }
    }
}

impl<T: AsRef<[u8]>> State<T> {
    fn process_source(&mut self, data: T) {
        self.source_data = data;
        for (index, str) in self.source_data.as_ref().windows(3).enumerate() {
            self.source_indices
                .entry(*array_ref![str, 0, 3])
                .or_insert(Vec::new())
                .push(index);
        }
    }

    fn encode(&mut self, mut target: &[u8]) -> Vec<OutputSymbol> {
        let source = self.source_data.as_ref();
        let mut result = Vec::new();

        while target.len() > 2 {
            let longest_match = (&self.source_indices.get(array_ref![target, 0, 3]))
                .unwrap_or(&Vec::new())
                .into_iter()
                .map(|&index| {
                    let first_difference = target
                        .into_iter()
                        .zip(&source[index..])
                        .position(|(&source, &target)| source != target);

                    let maximum_possible_match = std::cmp::min(target.len(), source.len() - index);
                    match first_difference {
                        Some(pos) => (pos - 1, index),
                        None => (maximum_possible_match, index),
                    }
                })
                .max_by_key(|&(length, _)| length);

            match longest_match {
                Some((length, index)) if length >= 3 => {
                    result.push(OutputSymbol::Copy(0, index as isize, length));
                    target = &target[length..];
                }
                _ => {
                    // No match, or match less than our abitrary 3-byte threshold: emit a literal
                    result.push(OutputSymbol::Literal(target[0]));
                    target = &target[1..];
                }
            }
        }
        for remainder in target {
            result.push(OutputSymbol::Literal(*remainder));
        }
        result
    }

    fn decode(&mut self, encoded_data: &[OutputSymbol]) -> Vec<u8> {
        let data_ref = (0u8..=255).collect::<Vec<u8>>();
        encoded_data
            .into_iter()
            .flat_map(|symbol| match *symbol {
                OutputSymbol::Literal(a) => &data_ref[a as usize..a as usize + 1],
                OutputSymbol::Copy(_, offset, length) => {
                    &self.source_data.as_ref()[offset as usize..offset as usize + length]
                }
            })
            .map(|ptr| *ptr)
            .collect()
    }
}


#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    #[test]
    fn duplicate_substrings_result_in_multiple_indicies() {
        let mut state = State::default();
        let data = [1u8, 2u8, 3u8, 1u8, 2u8, 3u8];

        state.process_source(data);
        assert_eq!(state.source_indices[&[1u8, 2u8, 3u8]], vec![0, 3]);
    }

    use super::*;
    proptest! {
        #[test]
        fn source_extraction_doesnt_crash(ref data in ".*") {
            let mut state = State::default();
            state.process_source(data.as_bytes());
        }

        #[test]
        fn source_extraction_calculates_one_index_per_window(ref data in ".{3,}") {
            let mut state = State::default();
            state.process_source(data.as_bytes());
            let index_count = state.source_indices.values().into_iter().fold(0, |acc, ref v| { acc + v.len() });
            prop_assert_eq!(index_count, data.len() - 2);
        }

        #[test]
        fn roundtrip_is_noop(ref source in ".{3,}", ref target in ".{3,}") {
            let mut state = State::default();
            state.process_source(source.as_bytes());
            let encoded_data = state.encode(target.as_bytes());
            let decoded_data = state.decode(&encoded_data);
            prop_assert_eq!(target.as_bytes(), &*decoded_data);
        }

        #[test]
        fn target_identical_to_source_encodes_to_single_copy(ref source in ".{3,}") {
            let mut state = State::default();
            state.process_source(source.as_bytes());
            let encoded_data = state.encode(source.as_bytes());

            prop_assert_eq!(encoded_data, vec![OutputSymbol::Copy(0, 0, source.len())]);
        }
    }
}
