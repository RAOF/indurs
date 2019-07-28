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

pub fn factorise<T: AsMut<[u32]> + AsRef<[u32]>>(source : &[u8], suffix_array : T) -> Box<[Factor]> {
    let mut phi = vec![0; source.len() + 2];

    let mut sa = 
        std::iter::once(0)
        .chain(suffix_array.as_ref().iter().map(|a| *a + 1))
        .chain(std::iter::once(0))
        .collect::<std::vec::Vec<u32>>().into_boxed_slice();

    let mut top = 0;

    println!("SA is {:?}", sa);

    for i in 1..=(source.len() + 1) {
        while sa[top] > sa[i] {
            phi[sa[top] as usize] = sa[i];
            top -= 1;
        }
        top += 1;
        sa[top] = sa[i];
    }

    println!("Phi is {:?}", phi);
    println!("Overwritten SA is {:?}", sa);
    let mut next = 1u32;

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

        assert_eq!(*factorise(&data, suffix_array), expected_factorisation);
    }
}
