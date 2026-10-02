use alloc::string::String;
use core::fmt;

/// Error code shared by every port (SPEC section 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Empty,
    InvalidSubtag,
    DuplicateVariant,
    DuplicateExtension,
    BadNumber,
    BadOption,
    BadDate,
    Syntax,
    MissingOther,
    MissingArgument,
    BadArgument,
    TooDeep,
    BadJson,
    NotObject,
    NonStringValue,
    DuplicateKey,
}

impl ErrorCode {
    /// The code as written in the SPEC, for example `"INVALID_SUBTAG"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "EMPTY",
            Self::InvalidSubtag => "INVALID_SUBTAG",
            Self::DuplicateVariant => "DUPLICATE_VARIANT",
            Self::DuplicateExtension => "DUPLICATE_EXTENSION",
            Self::BadNumber => "BAD_NUMBER",
            Self::BadOption => "BAD_OPTION",
            Self::BadDate => "BAD_DATE",
            Self::Syntax => "SYNTAX",
            Self::MissingOther => "MISSING_OTHER",
            Self::MissingArgument => "MISSING_ARGUMENT",
            Self::BadArgument => "BAD_ARGUMENT",
            Self::TooDeep => "TOO_DEEP",
            Self::BadJson => "BAD_JSON",
            Self::NotObject => "NOT_OBJECT",
            Self::NonStringValue => "NON_STRING_VALUE",
            Self::DuplicateKey => "DUPLICATE_KEY",
        }
    }
}

/// An error with its code and the offending input (subtag, argument name,
/// option name, or position), possibly empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub code: ErrorCode,
    pub detail: String,
}

impl Error {
    pub(crate) fn new(code: ErrorCode, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.detail.is_empty() {
            f.write_str(self.code.as_str())
        } else {
            write!(f, "{}: {}", self.code.as_str(), self.detail)
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
