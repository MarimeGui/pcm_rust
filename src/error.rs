use magic_number::MagicNumberCheckError;
use std::error::Error;
use std::fmt;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum PCMError {
    IoError(IoError),
    WrongMagicNumber(MagicNumberCheckError),
    UnknownFormat(u16),
    UnknownBitsPerSample(u16),
    TooMuchData(usize),
}

impl Error for PCMError {
    fn description(&self) -> &str {
        match self {
            PCMError::IoError(e) => e.description(),
            PCMError::WrongMagicNumber(e) => e.description(),
            PCMError::UnknownFormat(_) => "Unknown format field value in Wave Header",
            PCMError::UnknownBitsPerSample(_) => {
                "Cannot infer information about a Bits per Sample in Wave header"
            }
            PCMError::TooMuchData(_) => "Cannot write this size of data to a Wave file",
        }
    }
}

impl fmt::Display for PCMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PCMError::IoError(e) => e.fmt(f),
            PCMError::WrongMagicNumber(e) => e.fmt(f),
            PCMError::UnknownFormat(v) => write!(f, "Unrecognized {}", v),
            PCMError::UnknownBitsPerSample(b) => write!(f, "Bits per Sample: {}", b),
            PCMError::TooMuchData(s) => write!(f, "Tried to write {} bytes of data", s),
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
