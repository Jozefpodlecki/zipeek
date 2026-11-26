use std::{fs, path::Path};

use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecoderConfig {
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub layers: usize,
    pub query_heads: usize,
    pub kv_heads: usize,
    pub head_dim: usize,
    pub vocab_size: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomVoiceConfig {
    pub talker: DecoderConfig,
    pub predictor: DecoderConfig,
    pub text_vocab_size: usize,
    pub text_hidden_size: usize,
    pub codebooks: usize,
    pub chinese_token: u32,
    pub vivian_token: u32,
    pub bos_token: u32,
    pub eos_token: u32,
    pub pad_token: u32,
    pub codec_think_token: u32,
    pub codec_think_bos_token: u32,
    pub codec_think_eos_token: u32,
    pub codec_pad_token: u32,
    pub codec_bos_token: u32,
    pub codec_eos_token: u32,
}

fn get_usize(value: &Value, key: &str) -> Result<usize, String> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|number| usize::try_from(number).ok())
        .ok_or_else(|| format!("invalid Qwen3-TTS config field {key}"))
}

fn token(value: &Value, key: &str) -> Result<u32, String> {
    u32::try_from(get_usize(value, key)?).map_err(|_| format!("Qwen3-TTS token {key} exceeds u32"))
}

fn decoder(value: &Value) -> Result<DecoderConfig, String> {
    let configuration = DecoderConfig {
        hidden_size: get_usize(value, "hidden_size")?,
        intermediate_size: get_usize(value, "intermediate_size")?,
        layers: get_usize(value, "num_hidden_layers")?,
        query_heads: get_usize(value, "num_attention_heads")?,
        kv_heads: get_usize(value, "num_key_value_heads")?,
        head_dim: get_usize(value, "head_dim")?,
        vocab_size: get_usize(value, "vocab_size")?,
    };
    if configuration.layers == 0
        || configuration.layers > 64
        || configuration.query_heads == 0
        || configuration.kv_heads == 0
        || configuration.query_heads % configuration.kv_heads != 0
        || configuration.head_dim == 0
        || configuration.head_dim > 256
        || configuration.hidden_size == 0
        || configuration.hidden_size > 8192
        || configuration.intermediate_size == 0
        || configuration.intermediate_size > 32768
    {
        return Err("unsupported Qwen3-TTS decoder dimensions".into());
    }
    Ok(configuration)
}

impl CustomVoiceConfig {
    pub fn from_directory(root: &Path) -> Result<Self, String> {
        let source = fs::read_to_string(root.join("config.json"))
            .map_err(|error| format!("Qwen3-TTS config.json: {error}"))?;
        Self::from_json(&source)
    }

    pub fn from_json(source: &str) -> Result<Self, String> {
        let value: Value = serde_json::from_str(source)
            .map_err(|error| format!("invalid Qwen3-TTS config JSON: {error}"))?;
        if value.get("tts_model_type").and_then(Value::as_str) != Some("custom_voice")
            || value.get("tokenizer_type").and_then(Value::as_str)
                != Some("qwen3_tts_tokenizer_12hz")
        {
            return Err("only Qwen3-TTS 12 Hz CustomVoice is supported".into());
        }
        let talker = value
            .get("talker_config")
            .ok_or("missing Qwen3-TTS talker_config")?;
        let predictor = talker
            .get("code_predictor_config")
            .ok_or("missing code_predictor_config")?;
        let languages = talker
            .get("codec_language_id")
            .ok_or("missing codec_language_id")?;
        let speakers = talker.get("spk_id").ok_or("missing spk_id")?;
        let configuration = Self {
            talker: decoder(talker)?,
            predictor: decoder(predictor)?,
            text_vocab_size: get_usize(talker, "text_vocab_size")?,
            text_hidden_size: get_usize(talker, "text_hidden_size")?,
            codebooks: get_usize(talker, "num_code_groups")?,
            chinese_token: token(languages, "chinese")?,
            vivian_token: token(speakers, "vivian")?,
            bos_token: token(&value, "tts_bos_token_id")?,
            eos_token: token(&value, "tts_eos_token_id")?,
            pad_token: token(&value, "tts_pad_token_id")?,
            codec_think_token: token(talker, "codec_think_id")?,
            codec_think_bos_token: token(talker, "codec_think_bos_id")?,
            codec_think_eos_token: token(talker, "codec_think_eos_id")?,
            codec_pad_token: token(talker, "codec_pad_id")?,
            codec_bos_token: token(talker, "codec_bos_id")?,
            codec_eos_token: token(talker, "codec_eos_token_id")?,
        };
        if configuration.talker.hidden_size != 1024
            || configuration.talker.layers != 28
            || configuration.predictor.layers != 5
            || configuration.codebooks != 16
            || configuration.predictor.hidden_size != configuration.talker.hidden_size
            || configuration.text_vocab_size != 151_936
            || configuration.text_hidden_size != 2048
            || configuration.talker.head_dim != 128
        {
            return Err("Qwen3-TTS config does not match approved 0.6B CustomVoice model".into());
        }
        Ok(configuration)
    }

    pub fn mandarin_vivian_codec_prefix(&self) -> Vec<u32> {
        vec![
            self.codec_think_token,
            self.codec_think_bos_token,
            self.chinese_token,
            self.codec_think_eos_token,
            self.vivian_token,
            self.codec_pad_token,
            self.codec_bos_token,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_published_customvoice_dimensions_and_prompt() {
        let source = r#"{
            "tts_model_type":"custom_voice",
            "tokenizer_type":"qwen3_tts_tokenizer_12hz",
            "tts_bos_token_id":151672,"tts_eos_token_id":151673,"tts_pad_token_id":151671,
            "talker_config":{
                "hidden_size":1024,"intermediate_size":3072,"num_hidden_layers":28,
                "num_attention_heads":16,"num_key_value_heads":8,"head_dim":128,"vocab_size":3072,
                "text_vocab_size":151936,"text_hidden_size":2048,"num_code_groups":16,
                "codec_think_id":2154,"codec_think_bos_id":2156,"codec_think_eos_id":2157,
                "codec_pad_id":2148,"codec_bos_id":2149,"codec_eos_token_id":2150,
                "codec_language_id":{"chinese":2055},"spk_id":{"vivian":3065},
                "code_predictor_config":{
                    "hidden_size":1024,"intermediate_size":3072,"num_hidden_layers":5,
                    "num_attention_heads":16,"num_key_value_heads":8,"head_dim":128,"vocab_size":2048
                }
            }
        }"#;
        let config = CustomVoiceConfig::from_json(source).unwrap();
        assert_eq!(
            config.mandarin_vivian_codec_prefix(),
            [2154, 2156, 2055, 2157, 3065, 2148, 2149]
        );
        assert_eq!(config.talker.layers, 28);
        assert_eq!(config.predictor.layers, 5);
        assert_eq!(config.codebooks, 16);
    }

    #[test]
    fn rejects_other_variants_and_invalid_dimensions() {
        assert!(CustomVoiceConfig::from_json("{}").is_err());
        assert!(
            decoder(&serde_json::json!({
                "hidden_size":1024,"intermediate_size":3072,"num_hidden_layers":5,
                "num_attention_heads":15,"num_key_value_heads":8,"head_dim":128,"vocab_size":2048
            }))
            .is_err()
        );
    }
}
