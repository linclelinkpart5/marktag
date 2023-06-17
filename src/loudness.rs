use std::fmt::Display;
use std::path::Path;

use bs1770::{Power, Windows100ms};
use claxon::FlacReader;

use crate::helpers::Track;
pub(crate) struct Loudness(Power);

impl Loudness {
    fn lkfs(&self) -> f32 {
        self.0.loudness_lkfs()
    }
}

impl Display for Loudness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3} LUFS", self.lkfs())
    }
}

pub(crate) struct LoudnessAnalyzer {
    windows: Windows100ms<Vec<Power>>,
}

impl LoudnessAnalyzer {
    pub fn new() -> Self {
        Self {
            windows: Windows100ms::new(),
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn calculate_track_loudness(&mut self, track_path: &Path) -> Loudness {
        let mut reader = FlacReader::open(&track_path).unwrap();

        let streaminfo = reader.streaminfo();

        // The maximum amplitude is 1 << (bits per sample - 1), because one bit
        // is the sign bit.
        let normalizer = 1.0 / (1_u64 << (streaminfo.bits_per_sample - 1)) as f32;

        assert!(streaminfo.channels == 2);

        let mut meters = vec![
            bs1770::ChannelLoudnessMeter::new(streaminfo.sample_rate);
            streaminfo.channels as usize
        ];

        let mut blocks = reader.blocks();
        let mut buffer = Vec::new();

        while let Some(block) = blocks.read_next_or_eof(buffer).unwrap() {
            for (ch, meter) in meters.iter_mut().enumerate() {
                meter.push(
                    block
                        .channel(ch as u32)
                        .iter()
                        .map(|s| *s as f32 * normalizer),
                );
            }
            buffer = block.into_buffer();
        }

        let zipped: Windows100ms<Vec<Power>> =
            bs1770::reduce_stereo(meters[0].as_100ms_windows(), meters[1].as_100ms_windows());
        let gated_power = bs1770::gated_mean(zipped.as_ref()).unwrap_or(Power(0.0));

        // Update the album loudness window.
        self.windows.inner.extend(zipped.inner);

        Loudness(gated_power)
    }

    pub fn calculate_album_loudness(&self) -> Loudness {
        let gated_power = bs1770::gated_mean(self.windows.as_ref()).unwrap_or(Power(0.0));

        Loudness(gated_power)
    }
}

pub(crate) struct ScannedTrack {
    track: Track,
    loudness: Loudness,
}

pub(crate) struct AnalysisOutput {
    scanned_tracks: Vec<ScannedTrack>,
    album_loudness: Loudness,
}

pub(crate) fn analyze_tracks(tracks: Vec<Track>) -> AnalysisOutput {
    // Scan tracks for loudness.
    let mut loudness_analyzer = LoudnessAnalyzer::new();
    let scanned_tracks = tracks
        .into_iter()
        .map(|track| {
            let track_loudness = loudness_analyzer.calculate_track_loudness(&track.path);

            ScannedTrack {
                track,
                loudness: track_loudness,
            }
        })
        .collect::<Vec<_>>();

    let album_loudness = loudness_analyzer.calculate_album_loudness();

    AnalysisOutput {
        scanned_tracks,
        album_loudness,
    }
}
