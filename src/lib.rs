/*!
# Yes, but what is it really ?

Division is a very costly operation for your CPU (probably between 10
and 40 cycles).

You may have noticed that when the divisor is known at compile time,
your compiler transforms the operations into a cryptic combination of
a multiplication and bitshift.

Fastdivide is about doing the same trick your compiler uses but
when the divisor is unknown at compile time.
Of course, it requires preprocessing a datastructure that is
specific to your divisor, and using it only makes sense if
this preprocessing is amortized by a high number of division (with the
same divisor).

# When is it useful ?

You should probably use `fastdivide`, if you do a lot (> 10) of division with the same divisor ;
and these divisions are a bottleneck in your program.

This is for instance useful to compute histograms.



# Example

```rust
use fastdivide::DividerU64;

fn histogram(vals: &[u64], min: u64, interval: u64, output: &mut [usize]) {

    // Preprocessing a datastructure dedicated
    // to dividing `u64` by `interval`.
    //
    // This preprocessing is not cheap.
    let divide = DividerU64::divide_by(interval);

    // We reuse the same `Divider` for all of the
    // values in vals.
    for &val in vals {
        if val < min {
            continue;
        }
        let bucket_id = divide.divide(val - min) as usize;
        if bucket_id < output.len() {
            output[bucket_id as usize] += 1;
        }
    }
}

# let mut output = vec![0; 3];
# histogram(&[0u64, 1u64, 4u64, 36u64, 2u64, 1u64], 1u64, 3u64, &mut output[..]);
# assert_eq!(output[0], 3);
# assert_eq!(output[1], 1);
# assert_eq!(output[2], 0);

```

*/


// ported from  libdivide.h by ridiculous_fish
//
//  This file is not the original library, it is an attempt to port part
//  of it to rust.
//
const LIBDIVIDE_ADD_MARKER: u8 = 0x40;
const LIBDIVIDE_U64_SHIFT_PATH: u8 = 0x80;
const LIBDIVIDE_64_SHIFT_MASK: u8 = 0x3F;


#[derive(Debug)]
pub struct DividerU64 {
    magic: u64,
    more: u8,
}

fn libdivide_mullhi_u64(x: u64, y: u64) -> u64 {
    let xl = x as u128;
    let yl = y as u128;
    ((xl * yl) >> 64) as u64
}


impl DividerU64 {
    pub fn divide_by(divisor: u64) -> DividerU64 {
        assert!(divisor > 0u64);
        let floor_log_2_d: u8 = 63u8 - (divisor.leading_zeros() as u8);
        if divisor & (divisor - 1) == 0 {
            DividerU64 {
                magic: 0u64,
                more: floor_log_2_d | LIBDIVIDE_U64_SHIFT_PATH,
            }
        } else {
            let u = 1u128 << (floor_log_2_d + 64);
            let mut proposed_m: u128 = u / divisor as u128;
            let reminder: u64 = (u - proposed_m * divisor as u128) as u64;
            assert!(reminder > 0 && reminder < divisor);
            let e: u64 = divisor - reminder;
            let more: u8 = if e < (1u64 << floor_log_2_d) {
                floor_log_2_d
            } else {
                proposed_m += proposed_m;
                let twice_rem = reminder * 2;
                if twice_rem >= divisor || twice_rem < reminder {
                    proposed_m += 1;
                }
                floor_log_2_d | LIBDIVIDE_ADD_MARKER
            };
            DividerU64 {
                more: more,
                magic: (proposed_m as u64) + 1u64,
            }
        }
    }

    #[allow(unknown_lints, inline_always)]
    #[inline(always)]
    pub fn divide(&self, n: u64) -> u64 {
        if self.more & LIBDIVIDE_U64_SHIFT_PATH != 0 {
            n >> (self.more & LIBDIVIDE_64_SHIFT_MASK)
        } else {
            let q = libdivide_mullhi_u64(self.magic, n);
            if self.more & LIBDIVIDE_ADD_MARKER != 0 {
                let t = ((n - q) >> 1).wrapping_add(q);
                t >> (self.more & LIBDIVIDE_64_SHIFT_MASK)
            } else {
                q >> self.more
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::DividerU64;

    #[test]
    fn test_libdivide() {
        for d in (1u64..100u64)
            .chain(vec![2048, 234234131223u64].into_iter())
            .chain((5..63).map(|i| 1 << i))
        {
            let divider = DividerU64::divide_by(d);
            for i in (0u64..10_000).chain(vec![2048, 234234131223u64, 1 << 43, 1 << 43 + 1]) {
                assert_eq!(divider.divide(i), i / d);
            }

        }
    }

}
