use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseFormat {
    JsonSchema { json_schema: JsonSchemaFormat },
}

#[derive(Clone, Debug, Serialize)]
pub struct JsonSchemaFormat {
    pub name: String,
    pub strict: bool,
    pub schema: Value,
}

impl ResponseFormat {
    pub fn strict_schema(name: impl Into<String>, schema: Value) -> Self {
        Self::JsonSchema {
            json_schema: JsonSchemaFormat {
                name: name.into(),
                strict: true,
                schema,
            },
        }
    }
}
