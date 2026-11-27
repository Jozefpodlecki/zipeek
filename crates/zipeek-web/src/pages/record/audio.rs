use std::rc::Rc;
use zipeek_browser::local_speech;

use super::state::RecordingId;

pub(super) struct RecordedAudio {
    id: RecordingId,
    wav: Rc<Vec<u8>>,
    waveform: Vec<f32>,
}

#[derive(Default)]
pub(super) struct AudioResources {
    recording: Option<RecordedAudio>,
    reference: Option<local_speech::MatchingAudio>,
}

impl Drop for AudioResources {
    fn drop(&mut self) {
        self.clear();
    }
}

impl AudioResources {
    pub(super) fn clear(&mut self) {
        self.recording = None;
        self.clear_reference();
    }

    pub(super) fn clear_reference(&mut self) {
        if let Some(reference) = self.reference.take() {
            local_speech::dispose_matching_audio(&reference);
        }
    }

    pub(super) fn clear_recording(&mut self) {
        self.recording = None;
    }

    pub(super) fn store_reference(&mut self, reference: local_speech::MatchingAudio) {
        if let Some(previous) = self.reference.replace(reference) {
            local_speech::dispose_matching_audio(&previous);
        }
    }

    pub(super) fn reference(&self) -> Option<local_speech::MatchingAudio> {
        self.reference.clone()
    }

    pub(super) fn store(&mut self, id: RecordingId, wav: Vec<u8>) {
        let waveform = local_speech::recorded_wav_waveform(&wav);
        self.recording = Some(RecordedAudio {
            id,
            wav: Rc::new(wav),
            waveform,
        });
    }

    pub(super) fn replay(&self, id: Option<RecordingId>) -> Option<Rc<Vec<u8>>> {
        self.recording
            .as_ref()
            .and_then(|recording| (id == Some(recording.id)).then(|| recording.wav.clone()))
    }

    pub(super) fn waveform(&self, id: Option<RecordingId>) -> Vec<f32> {
        self.recording
            .as_ref()
            .and_then(|recording| (id == Some(recording.id)).then(|| recording.waveform.clone()))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_audio_cannot_be_replayed() {
        let mut resources = AudioResources::default();
        let id = RecordingId(1);
        assert!(resources.replay(Some(id)).is_none());
        resources.store(id, vec![1, 2, 3]);
        assert!(resources.replay(Some(RecordingId(2))).is_none());
        assert_eq!(
            resources.replay(Some(id)).as_deref().map(Vec::as_slice),
            Some([1, 2, 3].as_slice())
        );
        resources.clear();
        assert!(resources.replay(Some(id)).is_none());
    }
}
