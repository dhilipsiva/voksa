//! voksa CLI library: argument parsing, WAV encoding, and native audio
//! playback. The `voksa` binary (src/main.rs) is a thin wrapper over these.
//!
//! Playback keeps synthesis off the audio thread: the whole utterance is
//! rendered up front, streamed into an `rtrb` SPSC ring, and the cpal output
//! callback only pops from the ring (docs/research/02-architecture-v2.md).

pub mod args;
pub mod playback;
pub mod wav;
