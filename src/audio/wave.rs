//! Waveform file parser.

use gm8exe::deps::minio::ReadPrimitives;
use std::{convert::TryFrom, io};

// wFormatTag constants
const FORMAT_PCM: u16 = 0x0001;
const FORMAT_IEEE_FLOAT: u16 = 0x0003;
const FORMAT_ALAW: u16 = 0x0006;
const FORMAT_MULAW: u16 = 0x0007;
const FORMAT_EXTENSIBLE: u16 = 0xFFFE;

/// Header data contained in the format RIFF chunk.
#[derive(Debug)]
pub struct WaveHeader {
    audio_format: WaveFormat,
    channels: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
}

/// Format of the bytes in the data RIFF chunk.
#[derive(Debug)]
pub enum WaveFormat {
    PCM,
    Float,
    G711A, // A-law
    G711U, // μ-law
}

impl TryFrom<u16> for WaveFormat {
    type Error = ();

    fn try_from(x: u16) -> Result<Self, Self::Error> {
        Ok(match x {
            FORMAT_PCM => Self::PCM,
            FORMAT_IEEE_FLOAT => Self::Float,
            FORMAT_ALAW => Self::G711A,
            FORMAT_MULAW => Self::G711U,
            _ => return Err(()),
        })
    }
}

/// Parses file header and returns that & how many bytes to read from the data chunk.
/// The reader's position will be at the start of the data chunk's bytes.
pub fn parse(rdr: &mut (impl io::Read + io::Seek)) -> io::Result<(WaveHeader, usize)> {
    // generic bad data return type
    let invalid_data = || io::ErrorKind::InvalidData.into();

    let riff = rdr.read_u32_be()?.to_be_bytes();
    let _filesize = rdr.read_u32_le()? as usize + 8; // TODO: maybe verify/return this...
    let wave = rdr.read_u32_be()?.to_be_bytes();
    if &riff != b"RIFF" || &wave != b"WAVE" {
        return Err(invalid_data())
    }

    let mut fmt: Option<WaveHeader> = None;
    loop {
        let id = rdr.read_u32_be()?.to_be_bytes();
        let size = rdr.read_u32_le()?;
        match &id {
            // audio format
            b"fmt " => {
                if size < 16 {
                    return Err(invalid_data())
                }
                let audio_format = WaveFormat::try_from(rdr.read_u16_le()?).map_err(|_| invalid_data())?;
                let channels = rdr.read_u16_le()?;
                let sample_rate = rdr.read_u32_le()?;
                let byte_rate = rdr.read_u32_le()?;
                let block_align = rdr.read_u16_le()?;
                let bits_per_sample = rdr.read_u16_le()?;
                rdr.seek(io::SeekFrom::Current(size as i64 - 16))?;
                fmt = Some(WaveHeader { audio_format, channels, sample_rate, byte_rate, block_align, bits_per_sample });
            },
            b"data" => {
                break match fmt {
                    Some(header) => Ok((header, size as usize)),
                    None => Err(invalid_data()),
                }
            },
            _ => {
                rdr.seek(io::SeekFrom::Current(size as i64))?;
            },
        }
    }
}
