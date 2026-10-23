//! Client library for the Divoom Pixoo (16x16) over Bluetooth Classic.
//!
//! The device speaks a simple framed binary protocol over an RFCOMM serial
//! channel. See the `docs/` folder (MkDocs) for the full protocol notes.

pub mod canvas;
pub mod device;
pub mod protocol;
pub mod rfcomm;
