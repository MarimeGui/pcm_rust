extern crate ez_io;
extern crate magic_number;

pub mod error;

use error::{PcmError, UndeterminableDataFormat, UnknownFormat};
use ez_io::{ReadE, WriteE};
use magic_number::check_magic_number;
use std::io::{Read, Seek, SeekFrom, Write, Cursor};
use std::time::Duration;

#[derive(Clone)]
pub struct Pcm {
    pub sample_rate: u32,
    pub nb_channels: u16,
    pub bits_per_sample: u16,
    pub frames: Vec<Frame>,
}

#[derive(Clone)]
pub struct Frame {
    pub samples: Vec<Sample>,
}

#[derive(Clone)]
pub enum Sample {
    Unsigned8bits(u8),
    Signed16bits(i16),
}

impl Pcm {
    pub fn import_wave_file<R: Read + Seek>(reader: &mut R) -> Result<Pcm, PcmError> {
        check_magic_number(reader, vec![b'R', b'I', b'F', b'F'])?;
        let _chunk_size = reader.read_le_to_u32()?;
        check_magic_number(reader, vec![b'W', b'A', b'V', b'E'])?;
        check_magic_number(reader, vec![b'f', b'm', b't', b' '])?;
        let _sub_chunk_1_size = reader.read_le_to_u32()?;
        let audio_format = reader.read_le_to_u16()?;
        if audio_format != 1 {
            return Err(PcmError::UnknownFormat(UnknownFormat {
                value: audio_format,
            }));
        }
        let nb_channels = reader.read_le_to_u16()?;
        let sample_rate = reader.read_le_to_u32()?;
        let _byte_rate = reader.read_le_to_u32()?;
        let _block_align = reader.read_le_to_u16()?;
        let bits_per_sample = reader.read_le_to_u16()?;
        if !((bits_per_sample == 8) | (bits_per_sample == 16)) {
            return Err(PcmError::UndeterminableDataFormat(
                UndeterminableDataFormat { bits_per_sample },
            ));
        }
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
        Ok(Pcm {
            sample_rate,
            nb_channels,
            bits_per_sample,
            frames,
        })
    }
    pub fn export_wave_file<W: Write + Seek>(&self, writer: &mut W) -> Result<(), PcmError> {
        let sub_chunk_2_size = self.get_audio_binary_size() as u32;
        writer.write_all(&[b'R', b'I', b'F', b'F'])?; // Chunk ID
        writer.write_le_to_u32(36 + sub_chunk_2_size)?; // Chunk Size
        writer.write_all(&[b'W', b'A', b'V', b'E'])?; // Format
        writer.write_all(&[b'f', b'm', b't', b' '])?; // Sub-chunk 1 ID
        writer.write_le_to_u32(16)?; // Sub-chunk 1 Size
        writer.write_le_to_u16(1)?; // Audio Format
        writer.write_le_to_u16(self.nb_channels)?; // Number of Channels
        writer.write_le_to_u32(self.sample_rate)?; // Sample Rate
        writer.write_le_to_u32(
            self.sample_rate * u32::from(self.nb_channels) * (u32::from(self.bits_per_sample) / 8),
        )?; // Byte Rate
        writer.write_le_to_u16(self.nb_channels * (self.bits_per_sample / 8))?; // Block Align
        writer.write_le_to_u16(self.bits_per_sample)?; // Bits per Sample
        writer.write_all(&[b'd', b'a', b't', b'a'])?; // Sub-chunk 2 ID
        writer.write_le_to_u32(sub_chunk_2_size)?; // Sub-chunk 2 size
        self.export_raw_file(writer)?; // PCM data
        Ok(())
    }
    pub fn export_raw_file<W: Write + Seek>(&self, writer: &mut W) -> Result<(), PcmError> {
        for frame in &self.frames {
            for sample in &frame.samples {
                match sample {
                    Sample::Unsigned8bits(s) => writer.write_to_u8(s.clone())?,
                    Sample::Signed16bits(s) => writer.write_le_to_i16(s.clone())?,
                }
            }
        }
        Ok(())
    }
    pub fn get_audio_binary_size(&self) -> usize {
        self.frames.len() * match self.frames.get(0) {
            Some(f) => f.get_binary_size(),
            None => 0,
        }
    }
    pub fn get_audio_duration(&self) -> Duration {
        let duration_float = (self.frames.len() as f64) / f64::from(self.sample_rate);
        Duration::new(
            duration_float.round() as u64,
            (duration_float.fract() * 10f64.powi(9)) as u32,
        )
    }
}

impl Frame {
    pub fn get_binary_size(&self) -> usize {
        self.samples.len() * match self.samples.get(0) {
            Some(s) => s.get_binary_size(),
            None => 0,
        }
    }
}

impl Sample {
    pub fn get_binary_size(&self) -> usize {
        match self {
            Sample::Unsigned8bits(_) => 1,
            Sample::Signed16bits(_) => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Pcm;
    use std::fs::File;
    use std::io::{BufReader, BufWriter};
    use std::time::{Duration, Instant};
    #[test]
    fn read_and_write() {
        let ref mut input_wave_reader = BufReader::new(File::open("test_files/input.wav").unwrap());
        println!("Importing Wave File...");
        let import_start = Instant::now();
        let input_pcm = Pcm::import_wave_file(input_wave_reader).unwrap();
        println!("Import took {}.{} seconds", import_start.elapsed().as_secs(), import_start.elapsed().subsec_nanos());
        let ref mut output_wave_writer =
            BufWriter::new(File::create("test_files/output.wav").unwrap());
        println!("Writing Wave File");
        let output_pcm = Instant::now();
        input_pcm.export_wave_file(output_wave_writer).unwrap();
        println!("Export took {}.{} seconds", output_pcm.elapsed().as_secs(), output_pcm.elapsed().subsec_nanos());
    }
}
