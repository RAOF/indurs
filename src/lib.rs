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

struct State<T : AsRef<[u8]>> {
    source_indices : std::collections::HashMap<[u8 ; 3], Vec<usize>>,
    source_data : T,
    target_indices : std::collections::HashMap<[u8 ; 3], Vec<usize>>
}

impl<T : AsRef<[u8]> + Default> Default for State<T> {
    fn default() -> State<T> {
        State {
            source_indices : std::collections::HashMap::new(),
            source_data : T::default(),
            target_indices : std::collections::HashMap::new()
        }
    }
}

impl<T : AsRef<[u8]>> State<T> {
    fn process_source(&mut self, data : T) {
        self.source_data = data;
        for (index, str) in self.source_data.as_ref().windows(3).enumerate() {
            self.source_indices.entry(*array_ref![str,0,3]).or_insert(Vec::new()).push(index);
        }
    }

    fn encode(&mut self, target : &[u8]) -> Vec<OutputSymbol> {
        target.into_iter().map(|byte| { OutputSymbol::Literal(*byte) }).collect()
    }

    fn decode(&mut self, encoded_data : &[OutputSymbol]) -> Vec<u8> {
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
}


#[cfg(test)]
mod tests {
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
    }
}
