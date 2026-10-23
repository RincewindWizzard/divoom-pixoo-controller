//! Message framing and command constants.
//!
//! Every message has the shape
//!
//! ```text
//! 01 | LL LL | CMD | ARGS... | CS CS | 02
//! ```
//!
//! * `LL LL` is `1 + ARGS.len() + 2` as little-endian u16
//! * `CS CS` is the sum of all bytes from `LL` up to the last argument,
//!   little-endian (a 4 byte sum is used once it no longer fits into 16 bit,
//!   which is what the community implementations do)

use anyhow::{bail, Result};

pub const START: u8 = 0x01;
pub const END: u8 = 0x02;

/// Command bytes seen in community reverse engineering. Not all are
/// implemented by this crate, see `docs/protocol/commands.md`.
pub mod cmd {
    pub const SET_JSON: u8 = 0x01;
    pub const SET_RADIO: u8 = 0x05;
    pub const SET_VOLUME: u8 = 0x08;
    pub const SET_PLAYSTATE: u8 = 0x0a;
    pub const SET_PLAYTRACK: u8 = 0x12;
    pub const SET_GAME_KEYDOWN: u8 = 0x17;
    pub const SET_DATE_TIME: u8 = 0x18;
    pub const SET_GAME_KEYUP: u8 = 0x21;
    pub const SET_KEYBOARD: u8 = 0x23;
    pub const SET_HOT: u8 = 0x26;
    pub const SET_TEMP_TYPE: u8 = 0x2b;
    pub const SET_TIME_TYPE: u8 = 0x2d;
    pub const SET_LIGHTNESS: u8 = 0x32;
    pub const SET_SLEEPTIME: u8 = 0x40;
    pub const SET_ALARM: u8 = 0x43;
    pub const SET_IMAGE: u8 = 0x44;
    pub const SET_VIEW: u8 = 0x45;
    pub const GET_VIEW: u8 = 0x46;
    pub const SET_LIGHT_COLOR: u8 = 0x47;
    pub const SET_ANIMATION_FRAME: u8 = 0x49;
    pub const SET_TEMP_UNIT: u8 = 0x4c;
    pub const SET_MEMORIAL: u8 = 0x54;
    pub const SET_TEMP: u8 = 0x5f;
    pub const SET_RADIO_FREQUENCY: u8 = 0x61;
    pub const SET_TOOL: u8 = 0x72;
    pub const SET_BRIGHTNESS: u8 = 0x74;
    pub const SET_GAME_KEYPRESS: u8 = 0x88;
    pub const SET_GIF: u8 = 0x8b;
    pub const SET_GAME: u8 = 0xa0;
    pub const SET_DESIGN: u8 = 0xbd;
}

/// Checksum of `bytes` (length field + command + arguments), little-endian.
pub fn checksum(bytes: &[u8]) -> Vec<u8> {
    let sum: u32 = bytes.iter().map(|&b| u32::from(b)).sum();
    if sum >= 65535 {
        sum.to_le_bytes().to_vec()
    } else {
        (sum as u16).to_le_bytes().to_vec()
    }
}

/// Frame a command with its arguments into a complete wire message.
pub fn build_message(command: u8, args: &[u8]) -> Result<Vec<u8>> {
    let length = args.len() + 3;
    if length > usize::from(u16::MAX) {
        bail!("payload too large: {} argument bytes", args.len());
    }
    let mut body = Vec::with_capacity(length + 2);
    body.extend_from_slice(&(length as u16).to_le_bytes());
    body.push(command);
    body.extend_from_slice(args);

    let mut msg = Vec::with_capacity(body.len() + 4);
    msg.push(START);
    msg.extend_from_slice(&body);
    msg.extend_from_slice(&checksum(&body));
    msg.push(END);
    Ok(msg)
}

/// A message received from the device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// Everything between the length field and the checksum.
    pub payload: Vec<u8>,
}

impl Message {
    /// Replies usually look like `04 CMD 55 DATA...`; returns `CMD` then.
    pub fn command(&self) -> Option<u8> {
        match self.payload.as_slice() {
            [0x04, cmd, 0x55, ..] => Some(*cmd),
            _ => None,
        }
    }

    /// The data after the `04 CMD 55` prefix (or the whole payload if the
    /// reply does not have that shape).
    pub fn data(&self) -> &[u8] {
        match self.payload.as_slice() {
            [0x04, _, 0x55, rest @ ..] => rest,
            other => other,
        }
    }
}

/// Extract all well-formed messages from a receive buffer. Bytes that do not
/// belong to a valid message (wrong checksum, garbage) are skipped.
pub fn parse_messages(buf: &[u8]) -> Vec<Message> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < buf.len() {
        if buf[i] != START {
            i += 1;
            continue;
        }
        match parse_one(&buf[i..]) {
            Some((msg, used)) => {
                out.push(msg);
                i += used;
            }
            None => i += 1,
        }
    }
    out
}

fn parse_one(buf: &[u8]) -> Option<(Message, usize)> {
    if buf.len() < 7 {
        return None;
    }
    let length = usize::from(u16::from_le_bytes([buf[1], buf[2]]));
    if length < 2 {
        return None;
    }
    let total = length + 4;
    if buf.len() < total || buf[total - 1] != END {
        return None;
    }
    let body = &buf[1..total - 3];
    let given = u16::from_le_bytes([buf[total - 3], buf[total - 2]]);
    let sum: u32 = body.iter().map(|&b| u32::from(b)).sum();
    if u32::from(given) != sum & 0xffff {
        return None;
    }
    Some((
        Message {
            payload: body[2..].to_vec(),
        },
        total,
    ))
}

/// Format bytes as space separated lowercase hex.
pub fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parse a hex string like `"74 64"`, `"0x74,0x64"` or `"7464"`.
pub fn parse_hex(s: &str) -> Result<Vec<u8>> {
    let cleaned: String = s
        .replace("0x", "")
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ',' && *c != ':')
        .collect();
    if cleaned.len() & 1 == 1 {
        bail!("odd number of hex digits");
    }
    (0..cleaned.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&cleaned[i..i + 2], 16).map_err(Into::into))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_view_message() {
        assert_eq!(
            build_message(cmd::GET_VIEW, &[]).unwrap(),
            [0x01, 0x03, 0x00, 0x46, 0x49, 0x00, 0x02]
        );
    }

    #[test]
    fn brightness_message() {
        assert_eq!(
            build_message(cmd::SET_BRIGHTNESS, &[100]).unwrap(),
            [0x01, 0x04, 0x00, 0x74, 0x64, 0xdc, 0x00, 0x02]
        );
    }

    #[test]
    fn parse_reply_roundtrip() {
        // 04 46 55 01 -> length 4 + 2
        let payload = [0x04, 0x46, 0x55, 0x01];
        let mut body = vec![(payload.len() + 2) as u8, 0x00];
        body.extend_from_slice(&payload);
        let mut raw = vec![START];
        raw.extend_from_slice(&body);
        raw.extend_from_slice(&checksum(&body));
        raw.push(END);

        let msgs = parse_messages(&[&[0xff][..], &raw, &raw].concat());
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].command(), Some(0x46));
        assert_eq!(msgs[0].data(), &[0x01]);
    }

    #[test]
    fn bad_checksum_is_skipped() {
        let mut raw = build_message(cmd::GET_VIEW, &[]).unwrap();
        raw[4] ^= 0xff;
        assert!(parse_messages(&raw).is_empty());
    }

    #[test]
    fn hex_parsing() {
        assert_eq!(parse_hex("0x74, 0x64").unwrap(), [0x74, 0x64]);
        assert_eq!(parse_hex("7464").unwrap(), [0x74, 0x64]);
        assert!(parse_hex("746").is_err());
    }
}
