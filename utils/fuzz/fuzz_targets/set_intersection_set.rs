#![no_main]

use std::collections::HashSet;

use libfuzzer_sys::fuzz_target;

use tlsn_utils_fuzz::{assert_invariants, SmallSet};

use utils::range::*;

fuzz_target!(|r: (SmallSet, SmallSet)| {
    let s1: RangeSet<u8> = r.0.into();
    let s2: RangeSet<u8> = r.1.into();

    let h1: HashSet<u8> = HashSet::from_iter(s1.iter());
    let h2: HashSet<u8> = HashSet::from_iter(s2.iter());

    let intersection = s1.intersection(&s2);
    let h3: HashSet<u8> = HashSet::from_iter(intersection.iter());

    assert_eq!(h3, h1.intersection(&h2).copied().collect::<HashSet<_>>());

    assert_invariants(intersection);
});
