use std::collections::VecDeque;

pub const TTS_CACHE_ENTRIES: usize = 20;
pub const TTS_CACHE_MAX_BYTES: usize = 32 * 1024 * 1024;

struct CachedAudio {
    key: String,
    wav: Vec<u8>,
}

pub struct TtsAudioCache {
    entries: VecDeque<CachedAudio>,
    bytes: usize,
}

impl TtsAudioCache {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            bytes: 0,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<Vec<u8>> {
        let position = self.entries.iter().position(|entry| entry.key == key)?;
        let entry = self.entries.remove(position)?;
        let wav = entry.wav.clone();
        self.entries.push_back(entry);
        Some(wav)
    }

    pub fn insert(&mut self, key: String, wav: Vec<u8>) {
        if wav.len() > TTS_CACHE_MAX_BYTES {
            return;
        }
        if let Some(position) = self.entries.iter().position(|entry| entry.key == key) {
            if let Some(previous) = self.entries.remove(position) {
                self.bytes -= previous.wav.len();
            }
        }
        self.bytes += wav.len();
        self.entries.push_back(CachedAudio { key, wav });
        while self.entries.len() > TTS_CACHE_ENTRIES || self.bytes > TTS_CACHE_MAX_BYTES {
            if let Some(evicted) = self.entries.pop_front() {
                self.bytes -= evicted.wav.len();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_only_twenty_most_recent_entries() {
        let mut cache = TtsAudioCache::new();
        for index in 0..21 {
            cache.insert(index.to_string(), vec![index as u8]);
        }
        assert!(cache.get("0").is_none());
        assert_eq!(cache.get("1"), Some(vec![1]));
        assert_eq!(cache.get("20"), Some(vec![20]));
    }

    #[test]
    fn reads_make_entries_recent_and_updates_replace_audio() {
        let mut cache = TtsAudioCache::new();
        for index in 0..20 {
            cache.insert(index.to_string(), vec![index as u8]);
        }
        assert_eq!(cache.get("0"), Some(vec![0]));
        cache.insert("20".into(), vec![20]);
        assert_eq!(cache.get("0"), Some(vec![0]));
        assert!(cache.get("1").is_none());
        cache.insert("0".into(), vec![99]);
        assert_eq!(cache.get("0"), Some(vec![99]));
    }

    #[test]
    fn cache_instances_do_not_share_speech_results() {
        let mut first = TtsAudioCache::new();
        let mut second = TtsAudioCache::new();
        first.insert("qwen:你好".into(), vec![1, 2, 3]);
        assert_eq!(second.get("qwen:你好"), None);
        second.insert("qwen:你好".into(), vec![4, 5, 6]);
        assert_eq!(first.get("qwen:你好"), Some(vec![1, 2, 3]));
        assert_eq!(second.get("qwen:你好"), Some(vec![4, 5, 6]));
    }

    #[test]
    fn synthesis_backends_have_separate_cache_keys() {
        let mut cache = TtsAudioCache::new();
        cache.insert("qwen:你好".into(), vec![1]);
        cache.insert("espeak:你好".into(), vec![2]);
        assert_eq!(cache.get("qwen:你好"), Some(vec![1]));
        assert_eq!(cache.get("espeak:你好"), Some(vec![2]));
    }

    #[test]
    fn enforces_memory_budget_and_does_not_cache_oversized_audio() {
        let mut cache = TtsAudioCache::new();
        cache.insert("first".into(), vec![1; 20 * 1024 * 1024]);
        cache.insert("second".into(), vec![2; 20 * 1024 * 1024]);
        assert!(cache.get("first").is_none());
        assert_eq!(cache.get("second").map(|wav| wav.len()), Some(20 * 1024 * 1024));
        cache.insert("huge".into(), vec![3; TTS_CACHE_MAX_BYTES + 1]);
        assert!(cache.get("huge").is_none());
    }
}
