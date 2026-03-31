pub fn modified_offset(left: &[u8], right: &[u8]) -> Option<usize> {
    modified_offset_simd(left, right)
}

pub fn modified_offset_scalar(left: &[u8], right: &[u8]) -> Option<usize> {
    // Note: Iterating UTF-8 character indices with `str::char_indices` is slower than iterating bytes and adjusting
    // the byte offset to the UTF-8 character boundary. In addition, it is 8~10x faster to search 32 bytes chunk
    // index at first then search the byte index within the chunk rather than searching the index byte-by-byte.
    // - Benchmark:  https://github.com/rhysd/misc/tree/master/rust_bench/str_utf8_aware_offset
    // - Discussion: https://users.rust-lang.org/t/how-to-find-common-prefix-of-two-byte-slices-effectively/25815
    const CHUNK_SIZE: usize = 32;
    const WORD_SIZE: usize = 8;

    let min_len = left.len().min(right.len());
    let mut offset = 0;

    while offset + CHUNK_SIZE <= min_len {
        let end = offset + CHUNK_SIZE;
        if left[offset..end] != right[offset..end] {
            break;
        }
        offset = end;
    }

    while offset + WORD_SIZE <= min_len {
        let left_word = u64::from_ne_bytes(left[offset..offset + WORD_SIZE].try_into().unwrap());
        let right_word = u64::from_ne_bytes(right[offset..offset + WORD_SIZE].try_into().unwrap());
        if left_word != right_word {
            break;
        }
        offset += WORD_SIZE;
    }

    while offset < min_len && left[offset] == right[offset] {
        offset += 1;
    }

    if offset == min_len {
        return (left.len() != right.len()).then_some(min_len);
    }
    Some(offset)
}

pub fn modified_offset_simd(left: &[u8], right: &[u8]) -> Option<usize> {
    let min_len = left.len().min(right.len());
    if min_len == 0 {
        return (left.len() != right.len()).then_some(0);
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
    if min_len >= 64 {
        // SAFETY: The caller already checked that both slices have at least 64 bytes in the hot path,
        // and the implementation uses only in-bounds unaligned loads.
        return unsafe { wide_simd::modified_offset_wide(left, right) };
    }

    modified_offset_scalar(left, right)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
mod wide_simd {
    use wide::{CmpEq, u8x32};

    pub(super) unsafe fn modified_offset_wide(left: &[u8], right: &[u8]) -> Option<usize> {
        const CHUNK_SIZE: usize = 128;
        let min_len = left.len().min(right.len());
        let mut offset = 0;

        while offset + CHUNK_SIZE <= min_len {
            macro_rules! unroll_checks {
                ($($offset:expr),+) => {
                    $(
                        // SAFETY: The loop bound guarantees that each 32-byte unaligned load is in-bounds.
                        let lhs = unsafe { load_u8x32(left.as_ptr().add(offset + $offset)) };
                        // SAFETY: Same as above for the right slice.
                        let rhs = unsafe { load_u8x32(right.as_ptr().add(offset + $offset)) };
                        if let Some(i) = mismatch_index(lhs, rhs) {
                            return Some(offset + $offset + i);
                        }
                    )+
                }
            }
            unroll_checks!(0, 32, 64, 96);
            offset += CHUNK_SIZE;
        }

        let index = offset
            + left[offset..].iter().zip(&right[offset..]).take_while(|(x, y)| x == y).count();
        if index == min_len {
            return (left.len() != right.len()).then_some(min_len);
        }
        Some(index)
    }

    #[inline]
    unsafe fn load_u8x32(ptr: *const u8) -> u8x32 {
        // SAFETY: The caller guarantees that reading 32 bytes from `ptr` is in-bounds.
        unsafe { ptr.cast::<u8x32>().read_unaligned() }
    }

    #[inline]
    fn mismatch_index(lhs: u8x32, rhs: u8x32) -> Option<usize> {
        let mask = lhs.simd_eq(rhs).to_bitmask();
        if mask == u32::MAX {
            return None;
        }
        Some((!mask).trailing_zeros() as usize)
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
            (127, 128),
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
}
