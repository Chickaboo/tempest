//! A minimal Standard MIDI File writer.

use std::io::{self, Write};

pub const TICKS_PER_QUARTER: u32 = 480;

/// Events as (absolute tick, raw message), in any order.
#[derive(Default)]
pub struct Track(Vec<(u32, Vec<u8>)>);

impl Track {
    pub fn named(name: &str) -> Self {
        let mut track = Track::default();
        track.meta(0, 0x03, name.as_bytes());
        track
    }

    pub fn meta(&mut self, tick: u32, kind: u8, data: &[u8]) {
        self.0.push((tick, [&[0xFF, kind, data.len() as u8], data].concat()));
    }

    pub fn tempo(&mut self, tick: u32, quarters_per_minute: f64) {
        let micros = (60e6 / quarters_per_minute) as u32;
        self.meta(tick, 0x51, &micros.to_be_bytes()[1..]);
    }

    pub fn note(&mut self, start: u32, end: u32, pitch: u8, velocity: u8) {
        self.0.push((start, vec![0x90, pitch, velocity]));
        self.0.push((end, vec![0x80, pitch, 0]));
    }

    pub fn pedal(&mut self, tick: u32, down: bool) {
        self.0.push((tick, vec![0xB0, 64, if down { 127 } else { 0 }]));
    }

    fn chunk(mut self) -> Vec<u8> {
        self.0.sort_by_key(|(tick, message)| (*tick, message[0] & 0xF0 != 0x80)); // releases first
        let end = self.0.last().map_or(0, |(tick, _)| *tick);
        self.meta(end, 0x2F, &[]);
        let (mut bytes, mut now) = (vec![], 0);
        for (tick, message) in self.0 {
            let delta = tick - now;
            let septets = (1..4).rev().map(|i| (delta >> (7 * i)) as u8 & 0x7F | 0x80);
            bytes.extend(septets.skip_while(|byte| *byte == 0x80).chain([delta as u8 & 0x7F]));
            bytes.extend(message);
            now = tick;
        }
        bytes
    }
}

pub fn write(mut out: impl Write, tracks: Vec<Track>) -> io::Result<()> {
    out.write_all(b"MThd")?;
    out.write_all(&[0, 0, 0, 6, 0, 1, 0, tracks.len() as u8])?;
    out.write_all(&(TICKS_PER_QUARTER as u16).to_be_bytes())?;
    for track in tracks {
        let bytes = track.chunk();
        out.write_all(b"MTrk")?;
        out.write_all(&(bytes.len() as u32).to_be_bytes())?;
        out.write_all(&bytes)?;
    }
    Ok(())
}
