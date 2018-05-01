use magic_number::MagicNumberCheckError;
use super::Sample;
use std::error::Error;
use std::fmt;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum PCMError {
    IoError(IoError),
    UnknownFormat(UnknownFormat),
    WrongMagicNumber(MagicNumberCheckError),
    UndeterminableDataFormat(UndeterminableDataFormat),
    UnimplementedSampleType(UnimplementedSampleType)
}

impl Error for PCMError {
    fn description(&self) -> &str {
        match self {
            PCMError::IoError(e) => e.description(),
            PCMError::UnknownFormat(e) => e.description(),
            PCMError::WrongMagicNumber(e) => e.description(),
            PCMError::UndeterminableDataFormat(e) => e.description(),
            PCMError::UnimplementedSampleType(e) => e.description(),
        }
    }
}

impl fmt::Display for PCMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PCMError::IoError(e) => e.fmt(f),
            PCMError::UnknownFormat(e) => e.fmt(f),
            PCMError::WrongMagicNumber(e) => e.fmt(f),
            PCMError::UndeterminableDataFormat(e) => e.fmt(f),
            PCMError::UnimplementedSampleType(e) => e.fmt(f)
        }
    }
}

impl From<IoError> for PCMError {
    fn from(e: IoError) -> PCMError {
        PCMError::IoError(e)
    }
}

impl From<UnknownFormat> for PCMError {
    fn from(e: UnknownFormat) -> PCMError {
        PCMError::UnknownFormat(e)
    }
}

impl From<MagicNumberCheckError> for PCMError {
    fn from(e: MagicNumberCheckError) -> PCMError {
        PCMError::WrongMagicNumber(e)
    }
}

impl From<UndeterminableDataFormat> for PCMError {
    fn from(e: UndeterminableDataFormat) -> PCMError {
        PCMError::UndeterminableDataFormat(e)
    }
}

impl From<UnimplementedSampleType> for PCMError {
    fn from(e: UnimplementedSampleType) -> PCMError {
        PCMError::UnimplementedSampleType(e)
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

#[derive(Debug)]
pub struct UnimplementedSampleType {
    pub sample_type: Sample,
}

impl Error for UnimplementedSampleType {
    fn description(&self) -> &str {
        "Unimplemented sample type for writing to a Wave file"
    }
}

impl fmt::Display for UnimplementedSampleType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Sample type: {}", self.sample_type)
    }
}
