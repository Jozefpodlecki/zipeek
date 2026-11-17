use serde_json::{Value, json};

pub fn json_schema() -> Value {
    let string = || json!({"type": "string"});
    let string_array = || json!({"type":"array", "items":{"type":"string"}});
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["translation", "alternatives", "features", "nuance", "difficulty", "estimated_hsk", "rationale", "vocabulary", "grammar", "notes"],
        "properties": {
            "translation": string(),
            "alternatives": string_array(),
            "features": string_array(),
            "nuance": string_array(),
            "difficulty": {"type": "string", "enum": ["beginner", "elementary", "intermediate", "advanced"]},
            "estimated_hsk": {"type": ["integer", "null"]},
            "rationale": string(),
            "vocabulary": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["hanzi", "meaning", "pinyin"],
                    "properties": {
                        "hanzi": string(),
                        "meaning": string(),
                        "pinyin": string()
                    }
                }
            },
            "grammar": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["pattern", "explanation", "example_fragment"],
                    "properties": {
                        "pattern": string(),
                        "explanation": string(),
                        "example_fragment": string()
                    }
                }
            },
            "notes": string_array()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_requires_all_ten_properties_and_disallows_extras() {
        let schema = json_schema();
        let properties = schema["properties"].as_object().unwrap();
        let required = schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 10);
        assert_eq!(properties.len(), required.len());
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(
            schema["properties"]["vocabulary"]["items"]["additionalProperties"],
            false
        );
        assert_eq!(
            schema["properties"]["grammar"]["items"]["additionalProperties"],
            false
        );
        assert_eq!(
            schema["properties"]["estimated_hsk"]["type"],
            json!(["integer", "null"])
        );
    }
}
