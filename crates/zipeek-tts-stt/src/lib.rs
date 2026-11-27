#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unreachable_patterns)]
#![allow(unused_parens)]
#![allow(unused)]
#![allow(unused_imports)]

pub mod audio_codec;
pub mod candle_streaming_stt;
mod candle_stt;
pub mod candle_tts;
pub mod live_stt;
pub mod runtime;
pub mod server;
pub mod speech;
mod speech_cache;
mod speech_workers;
pub mod web;
