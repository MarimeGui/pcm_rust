use super::Sample;
use magic_number::MagicNumberCheckError;
use std::error::Error;
use std::fmt;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum PCMError {
    IoError(IoError),
    WrongMagicNumber(MagicNumberCheckError),
    UnknownFormat(u16),
    UndeterminableDataFormat(u16),
    UnimplementedSampleType(Sample),
}

impl Error for PCMError {
    fn description(&self) -> &str {
        match self {
            PCMError::IoError(e) => e.description(),
            PCMError::WrongMagicNumber(e) => e.description(),
            PCMError::UnknownFormat(_) => "Unknown format field value in Wave Header",
            PCMError::UndeterminableDataFormat(_) => {
                "Cannot infer information about a Bits per Sample in Wave header"
            }
            PCMError::UnimplementedSampleType(_) => {
                "Cannot write a sample type to Wave file as it is unimplemented"
            }
        }
    }
}

impl fmt::Display for PCMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PCMError::IoError(e) => e.fmt(f),
            PCMError::WrongMagicNumber(e) => e.fmt(f),
            PCMError::UnknownFormat(v) => write!(f, "Unrecognized 0x{:X}", v),
            PCMError::UndeterminableDataFormat(b) => write!(f, "Bits per Sample: 0x{:X}", b),
            PCMError::UnimplementedSampleType(s) => write!(f, "Sample type: {}", s),
        }
    }
}

impl From<IoError> for PCMError {
    fn from(e: IoError) -> PCMError {
        PCMError::IoError(e)
    }
}

impl From<MagicNumberCheckError> for PCMError {
    fn from(e: MagicNumberCheckError) -> PCMError {
        PCMError::WrongMagicNumber(e)
    }
}
