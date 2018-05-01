use ez_io::{ReadE, WriteE};
use magic_number::check_magic_number;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use {Frame, PCMError, PCMParameters, Result, Sample, PCM};

impl PCM {
    /// Imports a Wave file and returns a corresponding PCM
    pub fn wave_import_file<R: Read + Seek>(reader: &mut R) -> Result<PCM> {
        check_magic_number(reader, vec![b'R', b'I', b'F', b'F'])?;
        let _chunk_size = reader.read_le_to_u32()?;
        check_magic_number(reader, vec![b'W', b'A', b'V', b'E'])?;
        check_magic_number(reader, vec![b'f', b'm', b't', b' '])?;
        let _sub_chunk_1_size = reader.read_le_to_u32()?;
        let audio_format = reader.read_le_to_u16()?;
        if audio_format != 1 {
            unimplemented!("Cannot work with wave files not using format 1 for now");
        }
        let nb_channels = reader.read_le_to_u16()?;
        let sample_rate = reader.read_le_to_u32()?;
        let _byte_rate = reader.read_le_to_u32()?;
        let _block_align = reader.read_le_to_u16()?;
        let bits_per_sample = reader.read_le_to_u16()?;
        let sample_type = Sample::wave_from_format_bps(&audio_format, &bits_per_sample)?;
        let parameters = PCMParameters {
            sample_rate,
            nb_channels,
            sample_type: sample_type.clone(),
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
                match sample_type {
                    Sample::Unsigned8bits(_) => {
                        samples.push(Sample::Unsigned8bits(pcm_raw.read_to_u8()?))
                    }
                    Sample::Signed16bits(_) => {
                        samples.push(Sample::Signed16bits(pcm_raw.read_le_to_i16()?))
                    }
                    _ => unimplemented!("Cannot read anything else than u8 and i16 for now"),
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
    pub fn wave_export_file<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        // Check if the audio size can fit into a Wave file
        if self.get_audio_size() > (<u32>::max_value() as usize) {
            return Err(PCMError::TooMuchData(self.get_audio_size()));
        }
        if self.parameters
            .sample_type
            .wave_get_format_chunk_extra_size() != 0
        {
            unimplemented!(
                "Cannot work with sample types that requires extra info in format chunk for now"
            );
        }
        // Calculate sizes of all chunks beforehand
        let format_chunk_size_interior = 16 + self.parameters
            .sample_type
            .wave_get_format_chunk_extra_size();
        let format_chunk_size_total = format_chunk_size_interior + 8;
        let (fact_chunk_size_interior, fact_chunk_size_total) =
            if self.parameters.sample_type.wave_get_best_format() == 1 {
                (0, 0)
            } else {
                (4, 12)
            };
        let data_chunk_size_interior = self.get_audio_size() as u32;
        let data_chunk_size_total = data_chunk_size_interior + 8;
        let riff_chunk_size_interior =
            format_chunk_size_total + fact_chunk_size_total + data_chunk_size_total;
        // Write the header
        writer.write_all(&[b'R', b'I', b'F', b'F'])?; // RIFF Chunk
        writer.write_le_to_u32(riff_chunk_size_interior)?; // Interior Size of RIFF Chunk
        writer.write_all(&[b'W', b'A', b'V', b'E'])?; // WAVE Format
        writer.write_all(&[b'f', b'm', b't', b' '])?; // Format Chunk
        writer.write_le_to_u32(format_chunk_size_interior)?; // Format Chunk interior size
        writer.write_le_to_u16(self.parameters.sample_type.wave_get_best_format())?; // Audio Format
        writer.write_le_to_u16(self.parameters.nb_channels)?; // Number of Channels
        writer.write_le_to_u32(self.parameters.sample_rate)?; // Sample Rate
        writer.write_le_to_u32(
            self.parameters.sample_rate * u32::from(self.parameters.nb_channels)
                * (u32::from(self.parameters.sample_type.get_binary_size() / 8)),
        )?; // Byte Rate
        writer.write_le_to_u16(
            self.parameters.nb_channels * (self.parameters.sample_type.get_binary_size() / 8),
        )?; // Block Align
        writer.write_le_to_u16(self.parameters.sample_type.get_binary_size())?; // Bits per Sample
        if self.parameters.sample_type.wave_get_best_format() != 1 {
            writer.write_all(&[b'f', b'a', b'c', b't'])?; // Fact chunk
            writer.write_le_to_u32(fact_chunk_size_interior)?; // Fixed size of 4 bytes
            if self.frames.len() > (<u32>::max_value() as usize) {
                return Err(PCMError::TooManyFrames(self.frames.len()));
            }
            writer.write_le_to_u32(self.frames.len() as u32)?; // Number of frames
        }
        writer.write_all(&[b'd', b'a', b't', b'a'])?; // Sub-chunk 2 ID
        writer.write_le_to_u32(data_chunk_size_interior)?; // Sub-chunk 2 size
        self.export_raw_file(writer)?; // PCM data
        Ok(())
    }
}

impl Sample {
    /// Returns a Sample from a format and a number of bits per sample found in a Wave header
    pub fn wave_from_format_bps(format: &u16, bits_per_sample: &u16) -> Result<Sample> {
        Ok(match format {
            1 => {
                // Standard Integer PCM
                match bits_per_sample {
                    8 => Sample::Unsigned8bits(0u8),
                    16 => Sample::Signed16bits(0i16),
                    // 24 => Sample::Signed24bits(I24 {}), Unusable for now
                    32 => Sample::Signed32bits(0i32),
                    x => return Err(PCMError::UnknownBitsPerSample(*x)),
                }
            }
            2 => {
                // Microsoft ADPCM
                match bits_per_sample {
                    // 4 => Sample::MicrosoftADPCM(MicrosoftADPCM {}), Unusable for now
                    x => return Err(PCMError::UnknownBitsPerSample(*x)),
                }
            }
            3 => {
                // Float PCM
                match bits_per_sample {
                    32 => Sample::Float(0f32),
                    64 => Sample::DoubleFloat(0f64),
                    x => return Err(PCMError::UnknownBitsPerSample(*x)),
                }
            }
            17 => {
                // IMA ADPCM
                match bits_per_sample {
                    // 4 => Sample::ImaADPCM(ImaAdpcm {}), Unusable for now
                    x => return Err(PCMError::UnknownBitsPerSample(*x)),
                }
            }
            x => return Err(PCMError::UnknownFormat(*x)),
        })
    }
    /// Returns how many extra bytes needs to be added at the end of the format chunk
    pub fn wave_get_format_chunk_extra_size(&self) -> u32 {
        match self {
            Sample::Unsigned8bits(_) => 0,
            Sample::Signed16bits(_) => 0,
            Sample::Signed24bits(_) => 0,
            Sample::Signed32bits(_) => 0,
            Sample::MicrosoftADPCM(_) => 34,
            Sample::Float(_) => 0,
            Sample::DoubleFloat(_) => 0,
            Sample::ImaADPCM(_) => 4,
        }
    }
    /// Returns best format to use when writing this type to a Wave file
    pub fn wave_get_best_format(&self) -> u16 {
        match self {
            Sample::Unsigned8bits(_) => 1,
            Sample::Signed16bits(_) => 1,
            Sample::Signed24bits(_) => 1,
            Sample::Signed32bits(_) => 1,
            Sample::MicrosoftADPCM(_) => 2,
            Sample::Float(_) => 3,
            Sample::DoubleFloat(_) => 3,
            Sample::ImaADPCM(_) => 17,
        }
    }
}
