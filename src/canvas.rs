//! A 16x16 RGB pixel buffer and the Pixoo palette image encoding.

use anyhow::{bail, ensure, Result};
use std::collections::HashMap;

pub const SIZE: usize = 16;
pub type Rgb = [u8; 3];

/// Header that precedes a single (non animated) image frame in `SET_IMAGE`.
const IMAGE_HEADER: [u8; 4] = [0x00, 0x0a, 0x0a, 0x04];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Canvas {
    pixels: [Rgb; SIZE * SIZE],
}

impl Default for Canvas {
    fn default() -> Self {
        Self::filled([0, 0, 0])
    }
}

impl Canvas {
    pub fn filled(color: Rgb) -> Self {
        Self {
            pixels: [color; SIZE * SIZE],
        }
    }

    pub fn set(&mut self, x: usize, y: usize, color: Rgb) -> Result<()> {
        ensure!(
            x < SIZE && y < SIZE,
            "pixel ({x},{y}) is outside the {SIZE}x{SIZE} display"
        );
        self.pixels[y * SIZE + x] = color;
        Ok(())
    }

    pub fn get(&self, x: usize, y: usize) -> Rgb {
        self.pixels[y * SIZE + x]
    }

    /// Raw RGB bytes, row by row (768 bytes). Used for the `--state` file.
    pub fn to_rgb_bytes(&self) -> Vec<u8> {
        self.pixels.iter().flatten().copied().collect()
    }

    pub fn from_rgb_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != SIZE * SIZE * 3 {
            bail!(
                "expected {} bytes of RGB data, got {}",
                SIZE * SIZE * 3,
                bytes.len()
            );
        }
        let mut c = Self::default();
        for (px, chunk) in c.pixels.iter_mut().zip(bytes.chunks_exact(3)) {
            *px = [chunk[0], chunk[1], chunk[2]];
        }
        Ok(c)
    }

    /// Palette + packed pixel indices, as used inside a frame.
    fn palette_and_indices(&self) -> (Vec<Rgb>, Vec<u8>) {
        let mut palette: Vec<Rgb> = Vec::new();
        let mut lookup: HashMap<Rgb, u8> = HashMap::new();
        let mut indices = Vec::with_capacity(SIZE * SIZE);
        for &px in &self.pixels {
            let idx = *lookup.entry(px).or_insert_with(|| {
                palette.push(px);
                (palette.len() - 1) as u8
            });
            indices.push(idx);
        }
        (palette, indices)
    }

    /// Encode as a frame: `AA LL LL TT TT 00 NN palette pixels`.
    ///
    /// `time_ms` only matters for animations (it is written as 0 for stills).
    pub fn encode_frame(&self, time_ms: u16) -> Vec<u8> {
        let (palette, indices) = self.palette_and_indices();
        let count = palette.len();
        let bits = bits_per_pixel(count);

        let mut body = Vec::new();
        body.extend_from_slice(&time_ms.to_le_bytes());
        body.push(0x00); // palette reset flag
        body.push(if count >= 256 { 0 } else { count as u8 });
        body.extend(palette.iter().flatten());
        body.extend(pack_indices(&indices, bits));

        let length = (body.len() + 3) as u16;
        let mut frame = vec![0xaa];
        frame.extend_from_slice(&length.to_le_bytes());
        frame.extend(body);
        frame
    }

    /// Arguments of the `SET_IMAGE` command that shows this canvas.
    pub fn image_args(&self) -> Vec<u8> {
        let mut args = IMAGE_HEADER.to_vec();
        args.extend(self.encode_frame(0));
        args
    }
}

/// Bits needed per pixel index: ceil(log2(colors)), at least 1.
pub fn bits_per_pixel(colors: usize) -> u32 {
    let mut bits = 1;
    while (1usize << bits) < colors {
        bits += 1;
    }
    bits
}

/// Pack `bits`-wide values LSB first into bytes.
pub fn pack_indices(indices: &[u8], bits: u32) -> Vec<u8> {
    let mask = (1u32 << bits) - 1;
    let mut out = Vec::new();
    let (mut acc, mut nbits) = (0u32, 0u32);
    for &i in indices {
        acc |= (u32::from(i) & mask) << nbits;
        nbits += bits;
        while nbits >= 8 {
            out.push((acc & 0xff) as u8);
            acc >>= 8;
            nbits -= 8;
        }
    }
    if nbits > 0 {
        out.push((acc & 0xff) as u8);
    }
    out
}

/// Parse `#rrggbb`, `rrggbb`, `#rgb` or a basic color name.
pub fn parse_color(s: &str) -> Result<Rgb> {
    let named = match s.to_ascii_lowercase().as_str() {
        "black" => Some([0, 0, 0]),
        "white" => Some([255, 255, 255]),
        "red" => Some([255, 0, 0]),
        "green" => Some([0, 255, 0]),
        "blue" => Some([0, 0, 255]),
        "yellow" => Some([255, 255, 0]),
        "cyan" => Some([0, 255, 255]),
        "magenta" => Some([255, 0, 255]),
        "orange" => Some([255, 128, 0]),
        "purple" => Some([128, 0, 255]),
        "pink" => Some([255, 105, 180]),
        _ => None,
    };
    if let Some(c) = named {
        return Ok(c);
    }
    let h = s.trim_start_matches('#');
    let h: String = match h.len() {
        3 => h.chars().flat_map(|c| [c, c]).collect(),
        6 => h.to_string(),
        _ => bail!("invalid color '{s}', use #rrggbb or a name like red"),
    };
    let v = |i: usize| u8::from_str_radix(&h[i..i + 2], 16);
    match (v(0), v(2), v(4)) {
        (Ok(r), Ok(g), Ok(b)) => Ok([r, g, b]),
        _ => bail!("invalid color '{s}', use #rrggbb or a name like red"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_widths() {
        assert_eq!(bits_per_pixel(1), 1);
        assert_eq!(bits_per_pixel(2), 1);
        assert_eq!(bits_per_pixel(3), 2);
        assert_eq!(bits_per_pixel(4), 2);
        assert_eq!(bits_per_pixel(5), 3);
        assert_eq!(bits_per_pixel(256), 8);
    }

    #[test]
    fn packing_is_lsb_first() {
        // 2 bit values 1,2,3,0 -> 0b00_11_10_01
        assert_eq!(pack_indices(&[1, 2, 3, 0], 2), [0b0011_1001]);
        assert_eq!(pack_indices(&[1, 0, 1], 1), [0b101]);
    }

    #[test]
    fn solid_color_frame() {
        let c = Canvas::filled([1, 2, 3]);
        let f = c.encode_frame(0);
        // AA, len(2), time(2), flag, count, rgb(3), 256 px * 1 bit = 32 bytes
        assert_eq!(f.len(), 1 + 2 + 2 + 1 + 1 + 3 + 32);
        assert_eq!(&f[..3], &[0xaa, 42, 0]);
        assert_eq!(&f[3..9], &[0, 0, 0, 1, 1, 2]);
        assert_eq!(f[9], 3);
        assert!(f[10..].iter().all(|&b| b == 0));
    }

    #[test]
    fn frame_length_field_matches_size() {
        let mut c = Canvas::default();
        c.set(3, 4, [255, 0, 0]).unwrap();
        c.set(15, 15, [0, 255, 0]).unwrap();
        let f = c.encode_frame(100);
        assert_eq!(usize::from(u16::from_le_bytes([f[1], f[2]])), f.len());
        // two bits would be wrong: 3 colors -> 2 bits -> 64 bytes
        assert_eq!(f.len(), 1 + 2 + 2 + 1 + 1 + 9 + 64);
    }

    #[test]
    fn pixel_position_is_row_major() {
        let mut c = Canvas::default();
        c.set(1, 0, [9, 9, 9]).unwrap();
        let f = c.encode_frame(0);
        // palette: black(0), grey(1); pixel 1 set -> first pixel byte 0b10
        let pixels = &f[7 + 6..];
        assert_eq!(pixels[0], 0b10);
    }

    #[test]
    fn out_of_range_pixel() {
        assert!(Canvas::default().set(16, 0, [0; 3]).is_err());
    }

    #[test]
    fn rgb_roundtrip() {
        let mut c = Canvas::default();
        c.set(2, 7, [10, 20, 30]).unwrap();
        assert_eq!(Canvas::from_rgb_bytes(&c.to_rgb_bytes()).unwrap(), c);
    }

    #[test]
    fn colors() {
        assert_eq!(parse_color("#ff8000").unwrap(), [255, 128, 0]);
        assert_eq!(parse_color("f00").unwrap(), [255, 0, 0]);
        assert_eq!(parse_color("Blue").unwrap(), [0, 0, 255]);
        assert!(parse_color("#12").is_err());
        assert!(parse_color("zzzzzz").is_err());
    }
}
