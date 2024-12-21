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

const fn shiftr_filter_<const CHAR_: u8, const N_: usize>() -> Simd<u8, 32> {
    let mut filter = [CHAR_; 32];
    let mut i = 0;

    while i < N_ as usize {
        filter[i] = 0;
        i += 1;
    }

    return Simd::from_array(filter);
}

// Does a filter against shifted data. if I want to test for the existence
// of a sequence (e.g. ab), I can do:
//
// let a_mask = text.rotate_elements_right(1).simd_eq([b'a'; 32]);
// let b_mask = text.simd_eq([b'b'; 32]);
// a_mask & b_mask
//
#[derive(Clone, Copy)]
pub struct FilterShiftR<const SHIFT: usize> {
    filter: Simd<u8, 32>,
}

impl<const SHIFT: usize> FilterShiftR<SHIFT> {
    pub const fn new(c: u8) -> Self {
        let mut filter = [c; 32];
        let mut i = 0;

        while i < SHIFT as usize {
            filter[i] = 0;
            i += 1;
        }

        return Self {
            filter: Simd::from_array(filter),
        };
    }

    pub fn check_ne(self, t: Simd<u8, 32>) -> Mask<i8, 32> {
        return self.check::<false>(t);
    }

    pub fn check_eq(self, t: Simd<u8, 32>) -> Mask<i8, 32> {
        return self.check::<true>(t);
    }

    fn check<const EQ: bool>(self, t: Simd<u8, 32>) -> Mask<i8, 32> {
        let rotated = t.rotate_elements_right::<{ SHIFT }>();

        let mut mask = if EQ {
            rotated.simd_eq(self.filter)
        } else {
            rotated.simd_ne(self.filter)
        };
        let mut i = 0;

        while i < SHIFT {
            mask.set(i, !EQ);
            i += 1;
        }

        return mask;
    }
}
