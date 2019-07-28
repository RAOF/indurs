use std::convert::AsMut;

fn lcp(i : u32, j : u32, source : &[u8]) -> u32 {
    if i == 0 || j == 0 {
        return 0;
    }
    let first_suffix = &source[i as usize - 1..];
    let second_suffix = &source[j as usize - 1..];
    let mut common_prefix_len = 0;
    for (a, b) in first_suffix.into_iter().zip(second_suffix.into_iter()) {
        if *a != *b {
            return common_prefix_len;
        }
        common_prefix_len += 1;
    }
    common_prefix_len
}

#[derive(Debug, PartialEq, Eq)]
pub enum Factor {
    Normal(u32, u32),
    Special(u8)
}

pub fn factorise<T: AsMut<[u32]> + AsRef<[u32]>>(source : &[u8], start : usize, suffix_array : T) -> Box<[Factor]> {
    let mut phi = vec![0; source.len() + 2];

    let mut sa = 
        std::iter::once(0)
        .chain(suffix_array.as_ref().iter().map(|a| *a + 1))
        .chain(std::iter::once(0))
        .collect::<std::vec::Vec<u32>>().into_boxed_slice();

    let mut top = 0;

    for i in 1..=(source.len() + 1) {
        while sa[top] > sa[i] {
            phi[sa[top] as usize] = sa[i];
            top -= 1;
        }
        top += 1;
        sa[top] = sa[i];
    }

    let mut next = (start + 1) as u32;

    let mut factorised = std::vec::Vec::new();
    for t in 1..=source.len() {
        let nsv = phi[t];
        let psv = phi[nsv as usize];
        if t as u32 == next {
            let (inner_next, factor)  = next_factor(t as u32, &source, psv, nsv);
            factorised.push(factor);
            next = inner_next;
        }
        phi[t] = psv;
        phi[nsv as usize] = t as u32;
    }
    factorised.into_boxed_slice()
}

fn next_factor(i : u32, source : &[u8], psv : u32, nsv : u32) -> (u32, Factor) {
    let psv_len = lcp(i, psv, source);
    let nsv_len = lcp(i, nsv, source);
    let (p, l) = {
        if psv_len > nsv_len {
            (psv, psv_len)
        } else {
            (nsv, nsv_len)
        }
    };
    if l > 0 {
        (i + l, Factor::Normal(p - 1, l))
    } else {
        (i + 1, Factor::Special(source[i as usize - 1]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn lz_expand(source : impl AsRef<[u8]>, factorised : impl AsRef<[Factor]>) -> Box<[u8]> {
        let src = source.as_ref();
        let mut output = std::vec::Vec::new();
        for factor in factorised.as_ref() {
            match *factor {
                Factor::Special(data) => output.push(data),
                Factor::Normal(idx, len) => {
                    for i in idx..(idx + len) {
                        if i < src.len() as u32 {
                            output.push(src[i as usize]);
                        }
                        else {
                            output.push(output[(i - src.len() as u32) as usize]);
                        }
                    }
                }
            }
        }
        output.into_boxed_slice()
    }

    #[test]
    fn simple_lcp() {
        let data = [ 1, 2, 3, 1, 2, 3, 4, 5, 2, 3, 4, 1];

        assert_eq!(lcp(1, 1, &data), data.len() as u32);
        assert_eq!(lcp(1, 4, &data), 3);
        assert_eq!(lcp(1, 2, &data), 0);
        assert_eq!(lcp(5, 9, &data), 3);
        assert_eq!(lcp(0, 1, &data), 0);
    }

    #[test]
    fn check_suffix_array() {
        let data = [ 1, 2, 3, 1, 2, 3, 4, 5, 2, 3, 4, 1];
        let expected_sufficies = 
            [ 12, 11, 0, 3, 1, 8, 4, 2, 9, 5, 10, 6, 7 ]; 

        let (_, suffix_array) = suffix_array::SuffixArray::new(&data).into_parts();

        assert_eq!(suffix_array.len(), data.len() + 1);
        assert_eq!(suffix_array, expected_sufficies);
    }

    #[test]
    fn simple_lz_factorisation() {
        let data = [ 1, 2, 3, 1, 2, 3, 4, 5, 2, 3, 4, 1];
        let expected_factorisation = [
            Factor::Special(1),
            Factor::Special(2),
            Factor::Special(3),
            Factor::Normal(0, 3),
            Factor::Special(4),
            Factor::Special(5),
            Factor::Normal(4, 3),
            Factor::Normal(0, 1)
        ];

        let (_, suffix_array) = suffix_array::SuffixArray::new(&data).into_parts();

        assert_eq!(*factorise(&data, 0, suffix_array), expected_factorisation);
    }

    #[test]
    fn lz_factorisation_of_substring() {
        let data = [ 1, 2, 3, 1, 2, 3, 4, 5, 2, 3, 4, 1];
        let expected_factorisation = [
            Factor::Normal(0, 3),
            Factor::Special(4),
            Factor::Special(5),
            Factor::Normal(4, 3),
            Factor::Normal(0, 1)
        ];

        let (_, suffix_array) = suffix_array::SuffixArray::new(&data).into_parts();

        assert_eq!(*factorise(&data, 3, suffix_array), expected_factorisation);
    }

    #[test]
    fn simple_lz_factorisation_of_substring_roundtrips() {
        let data = [ 1, 2, 3, 1, 2, 3, 4, 5, 2, 3, 4, 1];

        let (_, suffix_array) = suffix_array::SuffixArray::new(&data).into_parts();

        let factorised = factorise(&data, 3, suffix_array);
        let expanded = lz_expand(&data[..3], factorised);

        assert_eq!(&*expanded, &[ 1, 2, 3, 4, 5, 2, 3, 4, 1 ]);
    }

    use proptest::prelude::*;

        prop_compose! {
            fn data_and_index()(data in proptest::collection::vec(proptest::num::u8::ANY, 2..100))
                (index in 1..data.len(), data in Just(data)) -> (Vec<u8>, usize) {
                (data, index)
            }
        }
    use proptest::string::bytes_regex;
    proptest! {
        #[test]
        fn lz_factorisation_roundtrip(ref data in bytes_regex(".*").unwrap()) {
            let (_, suffix_array) =
                suffix_array::SuffixArray::new(&data).into_parts();
            
            let factorised = factorise(&data, 0, suffix_array);
            let no_data = [0u8; 0];
            let ref_data : &[u8] = &data;
            prop_assert_eq!(&*lz_expand(no_data, factorised), ref_data);
        }

        #[test]
        fn lz_substring_factorisation_roundtrips((ref data, idx) in data_and_index()) {
            let (_, sa) = suffix_array::SuffixArray::new(&data).into_parts();
            
            let factorised = factorise(&data, idx, sa);
            let source_data = &data[0..idx];
            prop_assert_eq!(&*lz_expand(source_data, factorised), &data[idx..]);
        }
    }
}
