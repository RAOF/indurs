// Copyright © 2017 Christopher James Halse Rogers <raof@ubuntu.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate arrayref;

#[macro_use]
extern crate proptest;

#[cfg(test)]
extern crate itertools;

use std::vec::Vec;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ReferenceSource {
    Source,
    Target
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum OutputSymbol {
    Literal(u8),
    Copy(ReferenceSource, isize, usize),
}

pub struct State<T: AsRef<[u8]>> {
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
    pub fn process_source(&mut self, data: T) {
        self.source_data = data;
        for (index, str) in self.source_data.as_ref().windows(3).enumerate() {
            self.source_indices
                .entry(*array_ref![str, 0, 3])
                .or_insert(Vec::new())
                .push(index);
        }
    }

    pub fn encode(&mut self, target: &[u8]) -> Vec<OutputSymbol> {
        let source = self.source_data.as_ref();
        let mut result = Vec::new();
        let mut remaining_target = target;
        let mut target_index = 0usize;

        while remaining_target.len() > 2 {
            let longest_source_match = (&self.source_indices.get(array_ref![remaining_target, 0, 3]))
                .unwrap_or(&Vec::new())
                .into_iter()
                .map(|&index| {
                    let first_difference = remaining_target
                        .into_iter()
                        .zip(&source[index..])
                        .position(|(&source, &target)| source != target);

                    let maximum_possible_match = std::cmp::min(remaining_target.len(), source.len() - index);
                    match first_difference {
                        Some(pos) => (pos - 1, index),
                        None => (maximum_possible_match, index),
                    }
                })
                .max_by_key(|&(length, _)| length)
                .unwrap_or((0, 0));

            let longest_target_match = (&self.target_indices.get(array_ref![remaining_target, 0, 3]))
                .unwrap_or(&Vec::new())
                .into_iter()
                .map(|&index| {
                    let first_difference = remaining_target
                        .into_iter()
                        .zip(&target[index..])
                        .position(|(&source, &target)| source != target);

                    let maximum_possible_match = std::cmp::min(remaining_target.len(), target.len() - index);
                    match first_difference {
                        Some(pos) => (pos - 1, index),
                        None => (maximum_possible_match, index),
                    }
                })
                .max_by_key(|&(length, _)| length)
                .unwrap_or((0, 0));

            match (longest_target_match, longest_source_match) {
                ((target_len, index), (source_len, _)) if target_len >= source_len && target_len >= 3 => {
                    for skipped_data_index in 0..std::cmp::min(target_len, remaining_target.len() - 3) {
                        self.target_indices
                            .entry(*array_ref![remaining_target, skipped_data_index, 3])
                            .or_insert(Vec::new())
                            .push(target_index);
                        target_index = target_index + 1;
                    }
                    result.push(OutputSymbol::Copy(ReferenceSource::Target, index as isize, target_len));
                    remaining_target = &remaining_target[target_len..];
                }
                ((_, _), (source_len, index)) if source_len >= 3 => {
                    for skipped_data_index in 0..std::cmp::min(source_len, remaining_target.len() - 3) {
                        self.target_indices
                            .entry(*array_ref![remaining_target, skipped_data_index, 3])
                            .or_insert(Vec::new())
                            .push(target_index);
                        target_index = target_index + 1;
                    }
                    result.push(OutputSymbol::Copy(ReferenceSource::Source, index as isize, source_len));
                    remaining_target = &remaining_target[source_len..];
                }
                _ => {
                    self.target_indices
                        .entry(*array_ref![remaining_target, 0, 3])
                        .or_insert(Vec::new())
                        .push(target_index);
                    target_index = target_index + 1;

                    result.push(OutputSymbol::Literal(remaining_target[0]));
                    remaining_target = &remaining_target[1..];
                }
            }
        }
        for remainder in remaining_target {
            result.push(OutputSymbol::Literal(*remainder));
        }
        result
    }

    pub fn decode(&mut self, encoded_data: &[OutputSymbol]) -> Vec<u8> {
        let mut result = Vec::new();

        for symbol in encoded_data {
            match *symbol {
                OutputSymbol::Literal(a) => result.push(a),
                OutputSymbol::Copy(ReferenceSource::Source, offset, length) => {
                    result.extend_from_slice(&self.source_data.as_ref()[offset as usize..offset as usize + length])
                },
                OutputSymbol::Copy(ReferenceSource::Target, offset, length) => {
                    for i in 0..length {
                        let copy = result[offset as usize + i];
                        result.push(copy);
                    }
                }
            }
        }
        result
    }
}


#[cfg(test)]
mod tests {
    use itertools::Itertools;

    #[test]
    fn duplicate_substrings_result_in_multiple_indicies() {
        let mut state = State::default();
        let data = [1u8, 2u8, 3u8, 1u8, 2u8, 3u8];

        state.process_source(data);
        assert_eq!(state.source_indices[&[1u8, 2u8, 3u8]], vec![0, 3]);
    }

    use super::*;
    use proptest::string::bytes_regex;
    proptest! {
        #[test]
        fn source_extraction_doesnt_crash(ref data in bytes_regex(".*").unwrap()) {
            let mut state = State::<&[u8]>::default();
            state.process_source(data);
        }

        #[test]
        fn source_extraction_calculates_one_index_per_window(ref data in bytes_regex(".{3,}").unwrap()) {
            let mut state = State::<&[u8]>::default();
            state.process_source(data);
            let index_count = state.source_indices.values().into_iter().fold(0, |acc, ref v| { acc + v.len() });
            prop_assert_eq!(index_count, data.len() - 2);
        }

        #[test]
        fn roundtrip_is_noop(ref source in bytes_regex(".{3,}").unwrap(), ref target in bytes_regex(".{3,}").unwrap()) {
            let mut state = State::<&[u8]>::default();
            state.process_source(source);
            let encoded_data = state.encode(target);
            let decoded_data = state.decode(&encoded_data);
            prop_assert_eq!(target, &decoded_data);
        }

        #[test]
        fn target_identical_to_source_encodes_to_single_copy(ref source in bytes_regex(".{3,}").unwrap()) {
            let mut state = State::<&[u8]>::default();
            state.process_source(&source);
            let encoded_data = state.encode(&source);

            prop_assert_eq!(encoded_data, vec![OutputSymbol::Copy(ReferenceSource::Source, 0, source.len())]);
        }

        #[test]
        fn duplicate_runs_in_destination_encode_to_copies(ref target_fragment in bytes_regex(".{3,}").unwrap(), repeat in 2..10usize) {
            let source = Vec::<u8>::new();
            let mut state = State::<&[u8]>::default();
            state.process_source(&source);

            let dest = itertools::repeat_n(target_fragment, repeat)
                .flatten()
                .map(|a| *a)
                .collect::<Vec<u8>>();

            let encoded_data = state.encode(&dest);

            // target_fragment might, itself, be compressible, so all we can check
            // is that the final symbol is a copy of at least target_fragment * repeats
            // length
            if let Some(final_symbol) = encoded_data.last() {
                match *final_symbol {
                    OutputSymbol::Copy(ReferenceSource::Target, 0, length) => prop_assert!(length >= (target_fragment.len() * (repeat - 1))),
                    symbol => panic!("Final symbol {:?} is not a Copy", symbol)
                }
            } else {
                panic!("Encoded data is empty?!");
            }
        }
    }
}
