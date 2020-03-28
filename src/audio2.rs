use gm8exe::deps::minio::ReadPrimitives;
use std::{
    io::{self, Cursor, Read as _},
    sync::Mutex,
};

const WAVE_FORMAT_PCM: u16 = 0x0001;
const WAVE_FORMAT_IEEE_FLOAT: u16 = 0x0003;
const WAVE_FORMAT_ALAW: u16 = 0x0006;
const WAVE_FORMAT_MULAW: u16 = 0x0007;
const WAVE_FORMAT_EXTENSIBLE: u16 = 0xFFFE;

pub struct SoundSystem {
    sounds: Mutex<Vec<Sound>>,
}

pub struct Sound {
    data: Box<[u8]>,
    file_format: SoundFileFormat,
}

pub enum SoundFileFormat {
    MP3 { channels: u32, length: f32, sample_rate: u32 },
    Wav {},
    Mid {}, // unsupported
}

// handle to pass in when playing and such
pub struct SoundHandle(usize);

impl SoundSystem {
    pub fn new() -> Self {
        SoundSystem { sounds: Mutex::new(Vec::new()) }
    }

    pub fn add_sound(&mut self, data: Box<[u8]>) -> Option<SoundHandle> {
        let file_format = match data.get(..2)? {
            [0xFF, 0xFF] => {
                // TODO: reuse "global" decoder
                // TODO 2: calculate length like GM8 does
                let mut decoder = rmp3::Decoder::new(&data);

                // find first frame with samples - that's our point of reference (what GM8 does)
                let (channels, sample_rate, f1_sample_count) = {
                    let mut result: Option<(u32, u32, u32)> = None;
                    while let Some(rmp3::Frame { channels, sample_rate, sample_count, .. }) = decoder.peek_frame() {
                        if sample_count != 0 {
                            result = Some((channels, sample_rate, sample_count));
                            break;
                        }
                    }
                    decoder.skip_frame();
                    result?
                };

                // precalculate length - this is used for looping
                // even frames skipped over (unable to play) count for the length
                // this also means that our first-frame-point-of-reference here means nothing
                let mut length = f1_sample_count as f32 / sample_rate as f32;
                while let Some(rmp3::Frame { sample_rate, sample_count, .. }) = decoder.peek_frame() {
                    if sample_count != 0 {
                        length += sample_count as f32 / sample_rate as f32;
                    }
                    decoder.skip_frame();
                }

                SoundFileFormat::MP3 { channels, length, sample_rate }
            },
            [b'R', b'I'] => {
                let mut rdr = Cursor::new(&data);

                // RIFF header
                let riff = rdr.read_u32_be().ok()?;
                let _filesize = rdr.read_u32_le().ok()? as usize + 8;
                let wave = rdr.read_u32_be().ok()?;
                if riff != u32::from_be_bytes(*b"RIFF") || wave != u32::from_be_bytes(*b"WAVE") {
                    return None;
                }

                // data chunks
                loop {
                    let id = rdr.read_u32_be().ok()?.to_be_bytes();
                    let mut chunk = {
                        let size = rdr.read_u32_le().ok()? as usize;
                        let rdr_pos = rdr.position() as usize;
                        Cursor::new(data.get(rdr_pos..rdr_pos + size)?)
                    };
                    match &id {
                        // audio format
                        b"fmt " => {
                            let audio_format = chunk.read_u16_le().ok()?;
                            let channels = chunk.read_u16_le().ok()?;
                            let sample_rate = chunk.read_u32_le().ok()?;
                            let byte_rate = chunk.read_u32_le().ok()?;
                            let block_align = chunk.read_u16_le().ok()?;
                            let bits_per_sample = chunk.read_u16_le().ok()?;
                            match audio_format {
                                WAVE_FORMAT_PCM | WAVE_FORMAT_IEEE_FLOAT => (),

                                // unsupported (TODO: remove this later, testing-thing for now)
                                WAVE_FORMAT_ALAW | WAVE_FORMAT_MULAW | WAVE_FORMAT_EXTENSIBLE => {
                                    panic!("unsupported wave format {} ({0:#X})", audio_format)
                                },

                                _ => return None, // invalid
                            }
                        },

                        // data chunk
                        b"data" => {},

                        _ => (), // don't care
                    }
                }

                todo!()
            },
            [b'M', b'T'] => SoundFileFormat::Mid {},
            _ => return None,
        };
        let sound = Sound::new(data, file_format);
        let mut sounds = self.sounds.lock().unwrap();
        sounds.push(sound);
        Some(SoundHandle(sounds.len() - 1))
    }
}

impl Sound {
    pub fn new(data: Box<[u8]>, file_format: SoundFileFormat) -> Self {
        Self { data, file_format }
    }
}
