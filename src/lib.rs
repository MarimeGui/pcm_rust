//! A crate for manipulating PCM-related data in Rust.
//!
//! This crate currently allows for Importing and Writing Wave files with limited support for types.

extern crate ez_io;
extern crate magic_number;

/// Contains the errors for this library
pub mod error;

use error::{PCMError, UndeterminableDataFormat, UnknownFormat, UnimplementedSampleType};
use ez_io::{ReadE, WriteE};
use magic_number::check_magic_number;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::time::Duration;
use std::fmt;

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
    /// Number of bits per sample
    pub bits_per_sample: u16,
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
    /// Four bytes float
    Float(f32),
    /// Eight bytes float
    DoubleFloat(f64)
}

impl PCM {
    /// Imports a Wave file and returns a corresponding PCM
    pub fn import_wave_file<R: Read + Seek>(reader: &mut R) -> Result<PCM, PCMError> {
        check_magic_number(reader, vec![b'R', b'I', b'F', b'F'])?;
        let _chunk_size = reader.read_le_to_u32()?;
        check_magic_number(reader, vec![b'W', b'A', b'V', b'E'])?;
        check_magic_number(reader, vec![b'f', b'm', b't', b' '])?;
        let _sub_chunk_1_size = reader.read_le_to_u32()?;
        let audio_format = reader.read_le_to_u16()?;
        if audio_format != 1 {
            return Err(PCMError::UnknownFormat(UnknownFormat {
                value: audio_format,
            }));
        }
        let nb_channels = reader.read_le_to_u16()?;
        let sample_rate = reader.read_le_to_u32()?;
        let _byte_rate = reader.read_le_to_u32()?;
        let _block_align = reader.read_le_to_u16()?;
        let bits_per_sample = reader.read_le_to_u16()?;
        if !((bits_per_sample == 8) | (bits_per_sample == 16)) {
            return Err(PCMError::UndeterminableDataFormat(
                UndeterminableDataFormat { bits_per_sample },
            ));
        }
        let parameters = PCMParameters {
            sample_rate,
            nb_channels,
            bits_per_sample,
        };
        check_magic_number(reader, vec![b'd', b'a', b't', b'a'])?;
        let sub_chunk_2_size = reader.read_le_to_u32()?;
        let mut data = vec![0u8; sub_chunk_2_size as usize];
        reader.read_exact(&mut data)?;
        let mut pcm_raw = Cursor::new(data);
        let mut frames = Vec::with_capacity(
            (sub_chunk_2_size as usize / (bits_per_sample as usize / 8)) / nb_channels as usize,
        );
        let data_end = u64::from(sub_chunk_2_size);
        while pcm_raw.seek(SeekFrom::Current(0))? < data_end {
            let mut samples = Vec::with_capacity(nb_channels as usize);
            for _ in 0..nb_channels {
                match bits_per_sample {
                    8 => samples.push(Sample::Unsigned8bits(pcm_raw.read_to_u8()?)),
                    16 => samples.push(Sample::Signed16bits(pcm_raw.read_le_to_i16()?)),
                    _ => panic!(),
                }
            }
            frames.push(Frame { samples });
        }
        Ok(PCM {
            parameters,
            loop_info: None,
            frames,
        })
    }
    /// Exports a Wave file from a PCM
    pub fn export_wave_file<W: Write + Seek>(&self, writer: &mut W) -> Result<(), PCMError> {
        let sub_chunk_2_size = self.get_audio_size() as u32;
        writer.write_all(&[b'R', b'I', b'F', b'F'])?; // Chunk ID
        writer.write_le_to_u32(36 + sub_chunk_2_size)?; // Chunk Size
        writer.write_all(&[b'W', b'A', b'V', b'E'])?; // Format
        writer.write_all(&[b'f', b'm', b't', b' '])?; // Sub-chunk 1 ID
        writer.write_le_to_u32(16)?; // Sub-chunk 1 Size
        writer.write_le_to_u16(1)?; // Audio Format
        writer.write_le_to_u16(self.parameters.nb_channels)?; // Number of Channels
        writer.write_le_to_u32(self.parameters.sample_rate)?; // Sample Rate
        writer.write_le_to_u32(
            self.parameters.sample_rate * u32::from(self.parameters.nb_channels)
                * (u32::from(self.parameters.bits_per_sample) / 8),
        )?; // Byte Rate
        writer
            .write_le_to_u16(self.parameters.nb_channels * (self.parameters.bits_per_sample / 8))?; // Block Align
        writer.write_le_to_u16(self.parameters.bits_per_sample)?; // Bits per Sample
        writer.write_all(&[b'd', b'a', b't', b'a'])?; // Sub-chunk 2 ID
        writer.write_le_to_u32(sub_chunk_2_size)?; // Sub-chunk 2 size
        self.export_raw_file(writer)?; // PCM data
        Ok(())
    }
    /// Writes all samples directly to a writer
    pub fn export_raw_file<W: Write + Seek>(&self, writer: &mut W) -> Result<(), PCMError> {
        for frame in &self.frames {
            for sample in &frame.samples {
                match sample {
                    Sample::Unsigned8bits(s) => writer.write_to_u8(s.clone())?,
                    Sample::Signed16bits(s) => writer.write_le_to_i16(s.clone())?, // Todo: Allow for choosing endianness
                    x => return Err(PCMError::UnimplementedSampleType(UnimplementedSampleType {sample_type: x.clone()}))
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
            Sample::Float(_) => 32,
            Sample::DoubleFloat(_) => 64
        }
    }
}

impl fmt::Display for Sample {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Sample::Unsigned8bits(_) => "Unsigned 8 bits",
            Sample::Signed16bits(_) => "Signed 16 bits",
            Sample::Float(_) => "Float",
            Sample::DoubleFloat(_) => "Double-precision Float"
        };
        write!(f, "{}", text)
    }
}

#[cfg(test)]
mod tests {
    use super::PCM;
    use std::fs::File;
    use std::io::{BufReader, BufWriter};
    use std::time::Instant;
    #[test]
    fn read_and_write() {
        let ref mut input_wave_reader = BufReader::new(File::open("test_files/input.wav").unwrap());
        println!("Importing Wave File...");
        let import_start = Instant::now();
        let input_pcm = PCM::import_wave_file(input_wave_reader).unwrap();
        println!(
            "Import took {}.{} seconds",
            import_start.elapsed().as_secs(),
            import_start.elapsed().subsec_nanos()
        );
        let ref mut output_wave_writer =
            BufWriter::new(File::create("test_files/output.wav").unwrap());
        println!("Writing Wave File");
        let output_pcm = Instant::now();
        input_pcm.export_wave_file(output_wave_writer).unwrap();
        println!(
            "Export took {}.{} seconds",
            output_pcm.elapsed().as_secs(),
            output_pcm.elapsed().subsec_nanos()
        );
    }
}
