use super::{frontend::CHUNK_FRAMES, stem::projected_token_count};

pub const TOKENS_PER_CHUNK: usize = 13;
pub const CHUNKS_PER_ATTENTION_WINDOW: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncoderWindow {
    pub mel_frame_start: usize,
    pub mel_frame_end: usize,
    pub token_start: usize,
    pub token_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    Empty,
    TooLong,
}

pub fn encoder_attention_windows(valid_frames: usize) -> Result<Vec<EncoderWindow>, LayoutError> {
    if valid_frames == 0 {
        return Err(LayoutError::Empty);
    }
    if valid_frames > super::frontend::MAX_FRAMES {
        return Err(LayoutError::TooLong);
    }
    let frames_per_window = CHUNK_FRAMES * CHUNKS_PER_ATTENTION_WINDOW;
    let mut windows = Vec::with_capacity(valid_frames.div_ceil(frames_per_window));
    let mut token_start = 0;
    for frame_start in (0..valid_frames).step_by(frames_per_window) {
        let frame_end = (frame_start + frames_per_window).min(valid_frames);
        let token_end = token_start + projected_token_count(frame_end - frame_start);
        windows.push(EncoderWindow {
            mel_frame_start: frame_start,
            mel_frame_end: frame_end,
            token_start,
            token_end,
        });
        token_start = token_end;
    }
    Ok(windows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_second_tail_uses_seven_encoder_tokens() {
        assert_eq!(
            encoder_attention_windows(50).unwrap(),
            vec![EncoderWindow {
                mel_frame_start: 0,
                mel_frame_end: 50,
                token_start: 0,
                token_end: 7,
            }]
        );
    }

    #[test]
    fn one_second_of_audio_generates_one_attention_window() {
        assert_eq!(
            encoder_attention_windows(100).unwrap(),
            vec![EncoderWindow {
                mel_frame_start: 0,
                mel_frame_end: 100,
                token_start: 0,
                token_end: 13,
            }]
        );
    }

    #[test]
    fn longer_audio_splits_at_eight_second_boundaries() {
        let windows = encoder_attention_windows(1850).unwrap();
        assert_eq!(windows.len(), 3);
        assert_eq!(windows[0].mel_frame_start, 0);
        assert_eq!(windows[0].mel_frame_end, 800);
        assert_eq!(windows[0].token_end, 104);
        assert_eq!(windows[1].mel_frame_start, 800);
        assert_eq!(windows[1].mel_frame_end, 1600);
        assert_eq!(windows[1].token_end, 208);
        assert_eq!(windows[2].mel_frame_start, 1600);
        assert_eq!(windows[2].mel_frame_end, 1850);
        assert_eq!(windows[2].token_start, 208);
        assert_eq!(windows[2].token_end, 241);
    }

    #[test]
    fn rejects_empty_or_oversized_audio() {
        assert_eq!(encoder_attention_windows(0), Err(LayoutError::Empty));
        assert_eq!(encoder_attention_windows(3001), Err(LayoutError::TooLong));
    }

    #[test]
    fn one_frame_over_chunk_boundary_produces_fourteen_tokens() {
        assert_eq!(encoder_attention_windows(101).unwrap()[0].token_end, 14);
    }
}
