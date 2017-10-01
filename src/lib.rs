#[macro_use]
extern crate arrayref;

#[macro_use]
extern crate proptest;
use proptest::prelude::*;

use std::vec::Vec;

enum OutputSymbol {
    Literal(u8),
    Copy(u8, isize, usize)
}

struct State {
    source_indices : std::collections::HashMap<[u8 ; 3], Vec<usize>>,
    target_indices : std::collections::HashMap<[u8 ; 3], Vec<usize>>
}

impl Default for State {
    fn default() -> State {
        State {
            source_indices : std::collections::HashMap::new(),
            target_indices : std::collections::HashMap::new()
        }
    }
}

fn populate_source(state : &mut State, data : &[u8]) {
    for (index, str) in data.windows(3).enumerate() {
        state.source_indices.entry(*array_ref![str,0,3]).or_insert(Vec::new()).push(index);
    }
}

fn encode_target(state : &mut State, target : &[u8]) -> std::vec::Vec<OutputSymbol> {
    target.into_iter().map(|byte| { OutputSymbol::Literal(*byte) }).collect()
}

fn decode_target(state : &mut State, encoded_data : &[OutputSymbol]) -> std::vec::Vec<u8> {
    encoded_data
        .into_iter()
        .map(|symbol|
            {
                match *symbol {
                    OutputSymbol::Literal(a) => a,
                    OutputSymbol::Copy(_,_,_) => 0
                }
            })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn duplicate_substrings_result_in_multiple_indicies() {
        let mut state = State::default();
        let data = [1u8, 2u8, 3u8, 1u8, 2u8, 3u8];

        populate_source(&mut state, &data);
        assert_eq!(state.source_indices[&[1u8, 2u8, 3u8]], vec![0, 3]);
    }

    use super::*;
    proptest! {
        #[test]
        fn source_extraction_doesnt_crash(ref data in ".*") {
            let mut state = State::default();
            populate_source(&mut state, data.as_bytes());
        }

        #[test]
        fn source_extraction_calculates_one_index_per_window(ref data in ".{3,}") {
            let mut state = State::default();
            populate_source(&mut state, data.as_bytes());
            let index_count = state.source_indices.values().into_iter().fold(0, |acc, ref v| { acc + v.len() });
            prop_assert_eq!(index_count, data.len() - 2);
        }

        #[test]
        fn roundtrip_is_noop(ref source in ".{3,}", ref target in ".{3,}") {
            let mut state = State::default();
            populate_source(&mut state, source.as_bytes());
            let encoded_data = encode_target(&mut state, target.as_bytes());
            let decoded_data = decode_target(&mut state, &encoded_data);
            prop_assert_eq!(target.as_bytes(), &*decoded_data);
        }
    }
}
