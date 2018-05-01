//! A crate for manipulating PCM-related data in Rust.
//!
//! This crate currently allows for Importing and Writing Wave files with limited support for types.

extern crate ez_io;
extern crate magic_number;

/// Contains the errors for this library
pub mod error;
/// Contains structs for different types of samples for use in PCM data
pub mod sample_types;
/// Functions for Importing and Exporting Wave files
pub mod wave;

use error::PCMError;
use ez_io::WriteE;
use sample_types::{I24, ImaADPCM, MicrosoftADPCM};
use std::fmt;
use std::io::{Seek, Write};
use std::time::Duration;

/// The main result type used everywhere in this Library
type Result<T> = std::result::Result<T, PCMError>;

/// Represents PCM data.
#[derive(Clone)]
pub struct PCM {
    /// Parameters for this signal
    pub parameters: PCMParameters,
    /// Loop information if any
    pub loop_info: Option<Vec<LoopInfo>>,
    /// Frames that composes the stream
    pub frames: Vec<Frame>,
}

/// Parameters for PCM signal
#[derive(Clone)]
pub struct PCMParameters {
    /// Number of samples per second
    pub sample_rate: u32,
    /// Number of samples per frame
    pub nb_channels: u16,
    /// Sample type to use in frames
    pub sample_type: Sample,
}

/// Information about Looping in PCM data
#[derive(Clone)]
pub struct LoopInfo {
    /// Where does the loop start in frame count
    pub loop_start: u64,
    /// Where does the loop end in frame count
    pub loop_end: u64,
}

/// Contains a sample for each channel in the stream
#[derive(Clone)]
pub struct Frame {
    /// Samples for all the different channels
    pub samples: Vec<Sample>,
}

/// A value representing a level in the signal
#[derive(Clone, Debug)]
pub enum Sample {
    /// One unsigned byte
    Unsigned8bits(u8),
    /// Two bytes signed
    Signed16bits(i16),
    /// Three bytes signed
    Signed24bits(I24),
    /// Four bytes signed
    Signed32bits(i32),
    /// Half byte IMA ADPCM
    ImaADPCM(ImaADPCM),
    /// Half byte Microsoft ADPCM
    MicrosoftADPCM(MicrosoftADPCM),
    /// Four bytes float
    Float(f32),
    /// Eight bytes float
    DoubleFloat(f64),
}

impl PCM {
    /// Writes all samples directly to a writer
    pub fn export_raw_file<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        for frame in &self.frames {
            for sample in &frame.samples {
                match sample {
                    Sample::Unsigned8bits(s) => writer.write_to_u8(s.clone())?,
                    Sample::Signed16bits(s) => writer.write_le_to_i16(s.clone())?, // Todo: Allow for choosing endianness
                    _ => unimplemented!("Can only write u8s and u16s for now"),
                }
            }
        }
        Ok(())
    }
    /// Returns the size of the raw stream in bytes
    pub fn get_audio_size(&self) -> usize {
        self.frames.len() * match self.frames.get(0) {
            Some(f) => f.get_audio_size(),
            None => 0,
        }
    }
    /// Get the duration of the signal
    pub fn get_audio_duration(&self) -> Duration {
        let duration_float = (self.frames.len() as f64) / f64::from(self.parameters.sample_rate);
        Duration::new(
            duration_float.round() as u64,
            (duration_float.fract() * 10f64.powi(9)) as u32,
        )
    }
}

impl Frame {
    /// Returns how big a frame is in bytes
    pub fn get_audio_size(&self) -> usize {
        self.samples.len() * match self.samples.get(0) {
            Some(s) => (s.get_binary_size() / 8) as usize,
            None => 0,
        }
    }
}

impl Sample {
    /// Returns how big a sample is in bits
    pub fn get_binary_size(&self) -> u16 {
        match self {
            Sample::Unsigned8bits(_) => 8,
            Sample::Signed16bits(_) => 16,
            Sample::Signed24bits(_) => 24,
            Sample::Signed32bits(_) => 32,
            Sample::MicrosoftADPCM(_) => 4,
            Sample::Float(_) => 32,
            Sample::DoubleFloat(_) => 64,
            Sample::ImaADPCM(_) => 4,
        }
    }
}

impl fmt::Display for Sample {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Sample::Unsigned8bits(_) => "Unsigned 8 bits",
            Sample::Signed16bits(_) => "Signed 16 bits",
            Sample::Signed24bits(_) => "Signed 24 bits",
            Sample::Signed32bits(_) => "Signed 32 bits",
            Sample::MicrosoftADPCM(_) => "Microsoft ADPCM 4 bits",
            Sample::Float(_) => "Float 32 bits",
            Sample::DoubleFloat(_) => "Double-precision Float 64 bits",
            Sample::ImaADPCM(_) => "IMA ADPCM 4 bits",
        };
        write!(f, "{}", text)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{BufReader, BufWriter};
    use std::time::Instant;
    use PCM;
    #[test]
    fn wave_read_and_write() {
        let ref mut input_wave_reader = BufReader::new(File::open("test_files/input.wav").unwrap());
        println!("Importing Wave File...");
        let import_start = Instant::now();
        let input_pcm = PCM::wave_import_file(input_wave_reader).unwrap();
        println!(
            "Import took {}.{} seconds",
            import_start.elapsed().as_secs(),
            import_start.elapsed().subsec_nanos()
        );
        let ref mut output_wave_writer =
            BufWriter::new(File::create("test_files/output.wav").unwrap());
        println!("Writing Wave File");
        let output_pcm = Instant::now();
        input_pcm.wave_export_file(output_wave_writer).unwrap();
        println!(
            "Export took {}.{} seconds",
            output_pcm.elapsed().as_secs(),
            output_pcm.elapsed().subsec_nanos()
        );
    }
}
