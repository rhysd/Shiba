#[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
pub fn modified_offset(left: &[u8], right: &[u8]) -> Option<usize> {
    let min_len = left.len().min(right.len());
    if min_len == 0 {
        return (left.len() != right.len()).then_some(0);
    }
    modified_offset_scalar(left, right)
}

#[inline]
fn modified_offset_from_index(mut index: usize, left: &[u8], right: &[u8]) -> Option<usize> {
    const WORD_SIZE: usize = 8;

    let min_len = left.len().min(right.len());

    while index + WORD_SIZE <= min_len {
        let left_word = u64::from_ne_bytes(left[index..index + WORD_SIZE].try_into().unwrap());
        let right_word = u64::from_ne_bytes(right[index..index + WORD_SIZE].try_into().unwrap());
        if left_word != right_word {
            break;
        }
        index += WORD_SIZE;
    }

    while index < min_len && left[index] == right[index] {
        index += 1;
    }

    if index == min_len {
        return (left.len() != right.len()).then_some(min_len);
    }
    Some(index)
}

pub fn modified_offset_scalar(left: &[u8], right: &[u8]) -> Option<usize> {
    // Note: Iterating UTF-8 character indices with `str::char_indices` is slower than iterating bytes and adjusting
    // the byte offset to the UTF-8 character boundary. In addition, it is 8~10x faster to search 32 bytes chunk
    // index at first then search the byte index within the chunk rather than searching the index byte-by-byte.
    // - Benchmark:  https://github.com/rhysd/misc/tree/master/rust_bench/str_utf8_aware_offset
    // - Discussion: https://users.rust-lang.org/t/how-to-find-common-prefix-of-two-byte-slices-effectively/25815
    const CHUNK_SIZE: usize = 32;

    let min_len = left.len().min(right.len());
    let mut offset = 0;

    while offset + CHUNK_SIZE <= min_len {
        let end = offset + CHUNK_SIZE;
        if left[offset..end] != right[offset..end] {
            break;
        }
        offset = end;
    }

    modified_offset_from_index(offset, left, right)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
pub fn modified_offset(left: &[u8], right: &[u8]) -> Option<usize> {
    let min_len = left.len().min(right.len());
    if min_len == 0 {
        return (left.len() != right.len()).then_some(0);
    }
    if min_len < 64 {
        return modified_offset_scalar(left, right);
    }
    simd_dispatch(left, right)
}

#[cfg(target_arch = "aarch64")]
fn simd_dispatch(left: &[u8], right: &[u8]) -> Option<usize> {
    // SAFETY: Advanced SIMD (NEON) is part of the baseline ISA on aarch64.
    unsafe { aarch64::modified_offset_neon(left, right) }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn simd_dispatch(left: &[u8], right: &[u8]) -> Option<usize> {
    use std::arch;
    use std::sync::OnceLock;

    static DISPATCH: OnceLock<x86::DispatchFn> = OnceLock::new();

    let dispatch = *DISPATCH.get_or_init(|| {
        if arch::is_x86_feature_detected!("avx2") {
            x86::modified_offset_avx2
        } else if arch::is_x86_feature_detected!("sse2") {
            x86::modified_offset_sse2
        } else {
            modified_offset_scalar
        }
    });

    // SAFETY: The stored function pointer is selected based on runtime CPU feature checks,
    // or falls back to the scalar implementation.
    unsafe { dispatch(left, right) }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86 {
    use super::modified_offset_from_index;
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    pub(super) type DispatchFn = unsafe fn(&[u8], &[u8]) -> Option<usize>;

    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn modified_offset_avx2(left: &[u8], right: &[u8]) -> Option<usize> {
        const CHUNK_SIZE: usize = 64;
        let min_len = left.len().min(right.len());
        let mut offset = 0;

        while offset + CHUNK_SIZE <= min_len {
            macro_rules! unroll_mismatch_checks {
                ($($offset:expr),+) => {$(
                    // SAFETY: The loop bound guarantees that loading this 32-byte lane from both pointers is in-bounds.
                    let lhs = unsafe { _mm256_loadu_si256(left.as_ptr().add(offset + $offset).cast::<__m256i>()) };
                    // SAFETY: Same as above for the right slice.
                    let rhs = unsafe { _mm256_loadu_si256(right.as_ptr().add(offset + $offset).cast::<__m256i>()) };
                    let mask = _mm256_movemask_epi8(_mm256_cmpeq_epi8(lhs, rhs)) as u32;
                    if mask != u32::MAX {
                        return Some(offset + $offset + (!mask).trailing_zeros() as usize);
                    }
                )+}
            }

            unroll_mismatch_checks!(0, 32);
            offset += CHUNK_SIZE;
        }

        modified_offset_from_index(offset, left, right)
    }

    #[target_feature(enable = "sse2")]
    pub(super) unsafe fn modified_offset_sse2(left: &[u8], right: &[u8]) -> Option<usize> {
        const CHUNK_SIZE: usize = 16;
        let min_len = left.len().min(right.len());
        let mut offset = 0;

        while offset + CHUNK_SIZE <= min_len {
            // SAFETY: The loop bound guarantees that loading 16 bytes from both pointers is in-bounds.
            let lhs = unsafe { _mm_loadu_si128(left.as_ptr().add(offset).cast::<__m128i>()) };
            // SAFETY: Same as above for the right slice.
            let rhs = unsafe { _mm_loadu_si128(right.as_ptr().add(offset).cast::<__m128i>()) };
            let eq = _mm_cmpeq_epi8(lhs, rhs);
            let mask = _mm_movemask_epi8(eq) as u32;
            if mask != 0xffff {
                return Some(offset + (!mask).trailing_zeros() as usize);
            }
            offset += CHUNK_SIZE;
        }

        modified_offset_from_index(offset, left, right)
    }
}

#[cfg(target_arch = "aarch64")]
mod aarch64 {
    use super::modified_offset_from_index;
    use std::arch::aarch64::*;

    pub(super) unsafe fn modified_offset_neon(left: &[u8], right: &[u8]) -> Option<usize> {
        const CHUNK_SIZE: usize = 64;
        let min_len = left.len().min(right.len());
        let mut offset = 0;

        while offset + CHUNK_SIZE <= min_len {
            macro_rules! unroll_mismatch_checks {
                ($($offset:expr),+) => {$(
                    // SAFETY: The loop bound guarantees that loading 16 bytes from both pointers is in-bounds.
                    let lhs = unsafe { vld1q_u8(left.as_ptr().add(offset + $offset)) };
                    // SAFETY: Same as above for the right slice.
                    let rhs = unsafe { vld1q_u8(right.as_ptr().add(offset + $offset)) };
                    if let Some(i) = neon_mismatch_index(lhs, rhs) {
                        return Some(offset + $offset + i);
                    }
                )+}
            }

            unroll_mismatch_checks!(0, 16, 32, 48);
            offset += CHUNK_SIZE;
        }

        modified_offset_from_index(offset, left, right)
    }

    #[inline]
    fn neon_mismatch_index(lhs: uint8x16_t, rhs: uint8x16_t) -> Option<usize> {
        // SAFETY: `vceqq_u8` is available on all aarch64 targets as part of the baseline NEON ISA.
        let eq = unsafe { vceqq_u8(lhs, rhs) };
        // SAFETY: Bitwise NOT preserves the vector shape and turns equal bytes into 0x00 and mismatches into 0xff.
        let neq = unsafe { vmvnq_u8(eq) };
        // SAFETY: Reinterpreting the vector keeps the same bits and does not change size or alignment.
        let lanes = unsafe { vreinterpretq_u64_u8(neq) };

        // SAFETY: Accessing lane 0 is in-bounds for a 2-lane `uint64x2_t`.
        let lane = unsafe { vgetq_lane_u64(lanes, 0) };
        if lane != 0 {
            return Some((lane.trailing_zeros() / 8) as usize);
        }
        // SAFETY: Accessing lane 1 is in-bounds for a 2-lane `uint64x2_t`.
        let lane = unsafe { vgetq_lane_u64(lanes, 1) };
        if lane != 0 {
            return Some(8 + (lane.trailing_zeros() / 8) as usize);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::{modified_offset, modified_offset_scalar};

    fn cases() -> Vec<(Vec<u8>, Vec<u8>, Option<usize>)> {
        let mut cases = vec![
            (b"".into(), b"".into(), None),
            (b"".into(), b"a".into(), Some(0)),
            (b"a".into(), b"".into(), Some(0)),
            (b"a".into(), b"a".into(), None),
            (b"a".into(), b"b".into(), Some(0)),
            (b"abcdef".into(), b"abcxef".into(), Some(3)),
            (b"abcdef".into(), b"abcdefg".into(), Some(6)),
            (b"abcdefg".into(), b"abcdef".into(), Some(6)),
            (b"abcdefghijklmnopqrstuvwxyz".into(), b"abcdefghijklmnopqrstuvwxyz".into(), None),
            (b"abcdefghijklmnopqrstuvwxyz".into(), b"abcdefghijklmnopqrstuvwxzz".into(), Some(24)),
        ];

        for (size, pos) in [
            (32, 31),
            (33, 32),
            (34, 33),
            (63, 62),
            (64, 63),
            (65, 64),
            (127, 126),
            (128, 127),
            (129, 128),
            (255, 254),
            (256, 255),
            (257, 256),
            (256, 0),
            (256, 50),
            (256, 100),
            (256, 150),
            (256, 200),
            (256, 250),
        ] {
            let lhs = vec![b'a'; size];
            let mut rhs = lhs.clone();
            rhs[pos] = b'b';
            cases.push((lhs, rhs, Some(pos)));
        }

        for (len1, len2) in [
            (31, 31),
            (32, 32),
            (63, 63),
            (64, 64),
            (127, 127),
            (128, 128),
            (31, 32),
            (32, 31),
            (63, 64),
            (64, 63),
            (127, 128),
            (128, 127),
        ] {
            cases.push((
                vec![b'a'; len1],
                vec![b'a'; len2],
                (len1 != len2).then_some(len1.min(len2)),
            ));
        }

        for len in [30, 31, 32, 62, 63, 64, 126, 127, 128] {
            let lhs = vec![b'a'; len];
            let mut rhs = lhs.clone();
            rhs.extend_from_slice(b"tail");
            cases.push((lhs, rhs, Some(len)));
        }

        cases
    }

    #[test]
    fn scalar_modified_offset() {
        for (left, right, expected) in cases() {
            assert_eq!(
                expected,
                modified_offset_scalar(&left, &right),
                "left={left:?}, right={right:?}"
            );
        }
    }

    #[test]
    fn simd_modified_offset() {
        for (left, right, expected) in cases() {
            assert_eq!(expected, modified_offset(&left, &right), "left={left:?}, right={right:?}");
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn sse2_modified_offset() {
        for (left, right, expected) in cases() {
            assert_eq!(
                expected,
                // SAFETY: This function is only called on target x86 or x86_64.
                unsafe { super::x86::modified_offset_sse2(&left, &right) },
                "left={left:?}, right={right:?}"
            );
        }
    }
}
