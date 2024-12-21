use std::simd::prelude::*;

pub const fn shiftr_filter(character: u8, shift: u8) -> Simd<u8, 32> {
    let mut filter = [character; 32];
    let mut i = 0;

    while i < shift as usize {
        filter[i] = 0;
        i += 1;
    }

    return Simd::from_array(filter);
}

pub const fn shiftl_filter(character: u8, shift: u8) -> Simd<u8, 32> {
    let mut filter = [character; 32];

    let mut i = 0;

    while i < shift as usize {
        filter[32 - i - 1] = 0;
        i += 1;
    }

    return Simd::from_array(filter);
}
