use magic_number::MagicNumberCheckError;
use std::error::Error;
use std::fmt;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum PcmError {
    IoError(IoError),
    UnknownFormat(UnknownFormat),
    WrongMagicNumber(MagicNumberCheckError),
    UndeterminableDataFormat(UndeterminableDataFormat),
}

impl Error for PcmError {
    fn description(&self) -> &str {
        match self {
            PcmError::IoError(e) => e.description(),
            PcmError::UnknownFormat(e) => e.description(),
            PcmError::WrongMagicNumber(e) => e.description(),
            PcmError::UndeterminableDataFormat(e) => e.description(),
        }
    }
}

impl fmt::Display for PcmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PcmError::IoError(e) => e.fmt(f),
            PcmError::UnknownFormat(e) => e.fmt(f),
            PcmError::WrongMagicNumber(e) => e.fmt(f),
            PcmError::UndeterminableDataFormat(e) => e.fmt(f),
        }
    }
}

impl From<IoError> for PcmError {
    fn from(e: IoError) -> PcmError {
        PcmError::IoError(e)
    }
}

impl From<UnknownFormat> for PcmError {
    fn from(e: UnknownFormat) -> PcmError {
        PcmError::UnknownFormat(e)
    }
}

impl From<MagicNumberCheckError> for PcmError {
    fn from(e: MagicNumberCheckError) -> PcmError {
        PcmError::WrongMagicNumber(e)
    }
}

impl From<UndeterminableDataFormat> for PcmError {
    fn from(e: UndeterminableDataFormat) -> PcmError {
        PcmError::UndeterminableDataFormat(e)
    }
}

#[derive(Debug)]
pub struct UnknownFormat {
    pub value: u16,
}

impl Error for UnknownFormat {
    fn description(&self) -> &str {
        "Unknown AudioFormat in fmt sub-chunk"
    }
}

impl fmt::Display for UnknownFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unrecognized 0x{:X}", self.value)
    }
}

#[derive(Debug)]
pub struct UndeterminableDataFormat {
    pub bits_per_sample: u16,
}

impl Error for UndeterminableDataFormat {
    fn description(&self) -> &str {
        "Cannot infer information from a Bits per Sample value"
    }
}

impl fmt::Display for UndeterminableDataFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bits per Sample: 0x{:X}", self.bits_per_sample)
    }
}
