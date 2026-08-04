use core::fmt;

pub(crate) const FIXED_32_LENGTH: usize = 32;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Fixed32([u8; FIXED_32_LENGTH]);

impl Fixed32 {
    pub(crate) const fn new(bytes: [u8; FIXED_32_LENGTH]) -> Self {
        Self(bytes)
    }

    pub(crate) const fn as_bytes(&self) -> &[u8; FIXED_32_LENGTH] {
        &self.0
    }
}

impl fmt::Debug for Fixed32 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Fixed32([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::{FIXED_32_LENGTH, Fixed32};

    #[test]
    fn ordering_is_unsigned_byte_order() {
        let low = Fixed32::new([0; FIXED_32_LENGTH]);
        let mut high_bytes = [0; FIXED_32_LENGTH];
        high_bytes[31] = 1;
        let high = Fixed32::new(high_bytes);
        assert!(low < high);
        assert_eq!(high.as_bytes(), &high_bytes);
    }

    #[test]
    fn debug_is_redacted() {
        let value = Fixed32::new([0x42; FIXED_32_LENGTH]);
        assert_eq!(format!("{value:?}"), "Fixed32([REDACTED])");
        assert_eq!(value.as_bytes(), &[0x42; FIXED_32_LENGTH]);
    }
}
