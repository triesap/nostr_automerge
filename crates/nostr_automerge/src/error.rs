use core::fmt;

/// Why a canonical 32-byte lowercase hexadecimal value was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HexError {
    /// The text was not exactly 64 bytes.
    InvalidLength,
    /// A byte was not an ASCII lowercase hexadecimal digit.
    InvalidDigit,
}

impl fmt::Display for HexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidLength => "hexadecimal identifier must contain exactly 64 bytes",
            Self::InvalidDigit => "hexadecimal identifier must use lowercase ASCII digits",
        })
    }
}

impl std::error::Error for HexError {}
