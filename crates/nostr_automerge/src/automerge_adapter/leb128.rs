pub(crate) fn decode_u64(input: &[u8]) -> Result<(u64, usize), Leb128Error> {
    let mut value = 0_u64;
    for (index, byte) in input.iter().copied().take(10).enumerate() {
        let payload = byte & 0x7f;
        if index == 9 && payload > 1 {
            return Err(Leb128Error::Overflow);
        }
        value |= u64::from(payload) << (index * 7);
        if byte & 0x80 == 0 {
            if index != 0 && payload == 0 {
                return Err(Leb128Error::NonShortest);
            }
            return Ok((value, index + 1));
        }
        if index == 9 {
            return Err(Leb128Error::Overflow);
        }
    }
    Err(if input.len() < 10 {
        Leb128Error::Truncated
    } else {
        Leb128Error::Overflow
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Leb128Error {
    Truncated,
    Overflow,
    NonShortest,
}

#[cfg(test)]
mod tests {
    use super::{Leb128Error, decode_u64};

    #[test]
    fn decodes_boundaries_and_consumed_length() {
        assert_eq!(decode_u64(&[0]), Ok((0, 1)));
        assert_eq!(decode_u64(&[0x80, 1, 99]), Ok((128, 2)));
        assert_eq!(
            decode_u64(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 1]),
            Ok((u64::MAX, 10))
        );
    }

    #[test]
    fn rejects_truncation_overflow_and_alternate_forms() {
        assert_eq!(decode_u64(&[0x80]), Err(Leb128Error::Truncated));
        assert_eq!(decode_u64(&[0x80, 0]), Err(Leb128Error::NonShortest));
        assert_eq!(decode_u64(&[0xff; 10]), Err(Leb128Error::Overflow));
    }
}
