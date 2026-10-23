//! High level Pixoo commands on top of any byte stream.

use crate::canvas::{Canvas, Rgb, SIZE};
use crate::protocol::{self, cmd, hex, Message};
use anyhow::{bail, ensure, Result};
use std::io::{ErrorKind, Read, Write};

/// A transport the device can be driven over (RFCOMM socket, dry-run sink, ...).
pub trait Transport: Read + Write {}
impl<T: Read + Write> Transport for T {}

/// Top level "channels" selectable with `SET_VIEW`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    Clock = 0,
    Light = 1,
    Cloud = 2,
    Effects = 3,
    Visualization = 4,
    Design = 5,
    Scoreboard = 6,
}

#[derive(Clone, Debug)]
pub struct ClockOptions {
    pub style: u8,
    pub twenty_four_hour: bool,
    pub weather: bool,
    pub temperature: bool,
    pub calendar: bool,
    pub color: Option<Rgb>,
}

/// Max animation chunk payload, and limits of the 16 bit / 8 bit header fields.
const CHUNK_SIZE: usize = 200;
const MAX_FRAMES: usize = 60;

pub struct Pixoo<T: Transport> {
    io: T,
    /// Print every message as hex to stderr.
    pub verbose: bool,
}

impl<T: Transport> Pixoo<T> {
    pub fn new(io: T) -> Self {
        Self { io, verbose: false }
    }

    pub fn into_inner(self) -> T {
        self.io
    }

    /// Send a raw command byte with arguments.
    pub fn send(&mut self, command: u8, args: &[u8]) -> Result<()> {
        let msg = protocol::build_message(command, args)?;
        if self.verbose {
            eprintln!("-> {}", hex(&msg));
        }
        self.io.write_all(&msg)?;
        self.io.flush()?;
        Ok(())
    }

    /// Read whatever the device sends (until the transport times out or hits
    /// EOF) and return the well-formed messages in it.
    pub fn read_messages(&mut self) -> Result<Vec<Message>> {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 1024];
        loop {
            match self.io.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    buf.extend_from_slice(&chunk[..n]);
                    if n < chunk.len() {
                        break;
                    }
                }
                Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => break,
                Err(e) => return Err(e.into()),
            }
        }
        if self.verbose && !buf.is_empty() {
            eprintln!("<- {}", hex(&buf));
        }
        Ok(protocol::parse_messages(&buf))
    }

    /// Show a canvas (replaces whatever is displayed).
    pub fn set_image(&mut self, canvas: &Canvas) -> Result<()> {
        self.send(cmd::SET_IMAGE, &canvas.image_args())
    }

    /// Play an animation of `(canvas, duration_ms)` frames in a loop.
    pub fn set_animation(&mut self, frames: &[(Canvas, u16)]) -> Result<()> {
        ensure!(!frames.is_empty(), "animation needs at least one frame");
        if frames.len() == 1 {
            return self.set_image(&frames[0].0);
        }
        let frames = thin_frames(frames, MAX_FRAMES);
        let encoded: Vec<Vec<u8>> = frames.iter().map(|(c, t)| c.encode_frame(*t)).collect();
        let total: usize = encoded.iter().map(Vec::len).sum();
        let all: Vec<u8> = encoded.concat();
        let chunks = all.chunks(CHUNK_SIZE).count();
        if total > usize::from(u16::MAX) || chunks > 256 {
            bail!(
                "animation too large ({total} bytes, {chunks} chunks); use fewer frames or colors"
            );
        }
        for (index, chunk) in all.chunks(CHUNK_SIZE).enumerate() {
            let mut args = (total as u16).to_le_bytes().to_vec();
            args.push(index as u8);
            args.extend_from_slice(chunk);
            self.send(cmd::SET_ANIMATION_FRAME, &args)?;
        }
        Ok(())
    }

    /// 0..=100
    pub fn set_brightness(&mut self, percent: u8) -> Result<()> {
        ensure!(percent <= 100, "brightness must be 0-100");
        self.send(cmd::SET_BRIGHTNESS, &[percent])
    }

    /// Switch to a channel without extra options.
    pub fn set_channel(&mut self, channel: Channel) -> Result<()> {
        self.send(cmd::SET_VIEW, &[channel as u8])
    }

    pub fn show_clock(&mut self, o: &ClockOptions) -> Result<()> {
        ensure!(o.style <= 15, "clock style must be 0-15");
        // Separate command for the 12/24h setting, as the app does.
        self.send(cmd::SET_TIME_TYPE, &[u8::from(o.twenty_four_hour)])?;
        let mut args = vec![
            Channel::Clock as u8,
            u8::from(o.twenty_four_hour),
            o.style,
            0x01,
            u8::from(o.weather),
            u8::from(o.temperature),
            u8::from(o.calendar),
        ];
        if let Some(c) = o.color {
            args.extend_from_slice(&c);
        }
        self.send(cmd::SET_VIEW, &args)
    }

    /// Light channel. `effect` 0 = fixed color, 1-4 = animated.
    pub fn show_light(
        &mut self,
        color: Rgb,
        brightness: u8,
        effect: u8,
        power: bool,
    ) -> Result<()> {
        ensure!(brightness <= 100, "brightness must be 0-100");
        let mut args = vec![Channel::Light as u8];
        args.extend_from_slice(&color);
        args.extend_from_slice(&[brightness, effect, u8::from(power), 0, 0, 0]);
        self.send(cmd::SET_VIEW, &args)
    }

    pub fn show_effect(&mut self, number: u8) -> Result<()> {
        self.send(cmd::SET_VIEW, &[Channel::Effects as u8, number])
    }

    pub fn show_visualization(&mut self, number: u8) -> Result<()> {
        self.send(cmd::SET_VIEW, &[Channel::Visualization as u8, number])
    }

    pub fn show_design(&mut self, number: Option<u8>) -> Result<()> {
        self.set_channel(Channel::Design)?;
        if let Some(n) = number {
            self.send(cmd::SET_DESIGN, &[0x17, n])?;
        }
        Ok(())
    }

    pub fn show_scoreboard(&mut self, red: u16, blue: u16) -> Result<()> {
        ensure!(red <= 999 && blue <= 999, "scores are limited to 0-999");
        let mut args = vec![Channel::Scoreboard as u8, 0x00];
        args.extend_from_slice(&red.to_le_bytes());
        args.extend_from_slice(&blue.to_le_bytes());
        self.send(cmd::SET_VIEW, &args)
    }

    /// Set the device clock. Fields: year (e.g. 2026), month, day, hour, minute, second,
    /// weekday (0 = Sunday).
    #[allow(clippy::too_many_arguments)]
    pub fn set_date_time(
        &mut self,
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        weekday: u8,
    ) -> Result<()> {
        self.send(
            cmd::SET_DATE_TIME,
            &[
                (year % 100) as u8,
                (year / 100) as u8,
                month,
                day,
                hour,
                minute,
                second,
                weekday,
            ],
        )
    }

    /// Weather shown on the clock channel. `weather` code, see docs.
    pub fn set_weather(&mut self, celsius: i8, weather: u8) -> Result<()> {
        self.send(cmd::SET_TEMP, &[celsius as u8, weather])
    }

    /// Timer tool: 0 = stop, 1 = start; 2 is the community client's default when none is given (meaning unverified).
    pub fn timer(&mut self, state: u8) -> Result<()> {
        self.send(cmd::SET_TOOL, &[0x00, state])
    }

    pub fn countdown(&mut self, start: bool, minutes: u8, seconds: u8) -> Result<()> {
        self.send(cmd::SET_TOOL, &[0x03, u8::from(start), minutes, seconds])
    }

    pub fn noise_meter(&mut self, start: bool) -> Result<()> {
        self.send(cmd::SET_TOOL, &[0x02, if start { 1 } else { 2 }])
    }

    /// Ask for the current view. Replies are returned as parsed messages.
    pub fn get_view(&mut self) -> Result<Vec<Message>> {
        self.send(cmd::GET_VIEW, &[])?;
        self.read_messages()
    }
}

/// Reduce `frames` to at most `max` frames, spreading the kept frames evenly
/// and extending their duration to cover the dropped ones.
fn thin_frames(frames: &[(Canvas, u16)], max: usize) -> Vec<(Canvas, u16)> {
    let count = frames.len();
    if count <= max {
        return frames.to_vec();
    }
    let picks: Vec<usize> = (0..max).map(|i| i * count / max).collect();
    let mut out = Vec::with_capacity(max);
    for (n, &start) in picks.iter().enumerate() {
        let end = picks.get(n + 1).copied().unwrap_or(count);
        let time: u32 = frames[start..end].iter().map(|f| u32::from(f.1)).sum();
        out.push((
            frames[start].0.clone(),
            time.min(u32::from(u16::MAX)) as u16,
        ));
    }
    out
}

/// Number of pixels on one axis; re-exported for convenience.
pub const DISPLAY_SIZE: usize = SIZE;

/// A transport that only records what would be sent (for `--dry-run` and tests).
#[derive(Default)]
pub struct DryRun {
    pub written: Vec<u8>,
}

impl Read for DryRun {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(0)
    }
}

impl Write for DryRun {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.written.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sent(f: impl FnOnce(&mut Pixoo<DryRun>) -> Result<()>) -> Vec<u8> {
        let mut p = Pixoo::new(DryRun::default());
        f(&mut p).unwrap();
        p.into_inner().written
    }

    #[test]
    fn brightness() {
        assert_eq!(
            sent(|p| p.set_brightness(100)),
            [0x01, 0x04, 0x00, 0x74, 0x64, 0xdc, 0x00, 0x02]
        );
        assert!(Pixoo::new(DryRun::default()).set_brightness(101).is_err());
    }

    #[test]
    fn image_message_structure() {
        let bytes = sent(|p| p.set_image(&Canvas::filled([255, 0, 0])));
        assert_eq!(bytes[0], 0x01);
        assert_eq!(*bytes.last().unwrap(), 0x02);
        assert_eq!(bytes[3], cmd::SET_IMAGE);
        assert_eq!(&bytes[4..9], &[0x00, 0x0a, 0x0a, 0x04, 0xaa]);
        let len = usize::from(u16::from_le_bytes([bytes[1], bytes[2]]));
        assert_eq!(bytes.len(), len + 4);
    }

    #[test]
    fn scoreboard() {
        let b = sent(|p| p.show_scoreboard(2, 300));
        assert_eq!(&b[3..10], &[0x45, 0x06, 0x00, 0x02, 0x00, 0x2c, 0x01]);
    }

    #[test]
    fn animation_is_chunked() {
        let mut frames = Vec::new();
        for i in 0..10u8 {
            let mut c = Canvas::default();
            for x in 0..16 {
                c.set(x, 0, [i * 20, x as u8 * 10, 3]).unwrap();
            }
            frames.push((c, 100));
        }
        let bytes = sent(|p| p.set_animation(&frames));
        // every message starts with 01 .. cmd 49
        let msgs: Vec<&[u8]> = bytes.split_inclusive(|&b| b == 0x02).collect();
        assert!(msgs.len() > 1);
        assert_eq!(bytes[3], cmd::SET_ANIMATION_FRAME);
        // chunk index of the first message is 0
        assert_eq!(bytes[6], 0);
    }

    #[test]
    fn thinning_keeps_total_time() {
        let frames: Vec<_> = (0..100).map(|_| (Canvas::default(), 10u16)).collect();
        let thin = thin_frames(&frames, 60);
        assert_eq!(thin.len(), 60);
        let total: u32 = thin.iter().map(|f| u32::from(f.1)).sum();
        assert_eq!(total, 1000);
    }
}
