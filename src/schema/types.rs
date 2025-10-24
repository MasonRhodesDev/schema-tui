use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
    pub version: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub sections: Vec<SchemaSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaSection {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    pub fields: Vec<SchemaField>,
    #[serde(default)]
    pub visible_when: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    pub id: String,
    pub label: String,
    pub description: String,

    #[serde(flatten)]
    pub field_type: FieldType,

    #[serde(default)]
    pub optional: bool,

    #[serde(default)]
    pub env_expand: bool,

    #[serde(default)]
    pub ui_widget: UIWidget,

    #[serde(default)]
    pub keybind: Option<String>,

    #[serde(default)]
    pub subsection: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FieldType {
    #[serde(rename = "string")]
    String {
        #[serde(default)]
        default: Option<String>,
        #[serde(default)]
        max_length: Option<usize>,
    },

    #[serde(rename = "number")]
    Number {
        #[serde(default)]
        default: Option<i64>,
        #[serde(default)]
        min: Option<i64>,
        #[serde(default)]
        max: Option<i64>,
    },

    #[serde(rename = "float")]
    Float {
        #[serde(default)]
        default: Option<f64>,
        #[serde(default)]
        min: Option<f64>,
        #[serde(default)]
        max: Option<f64>,
        #[serde(default)]
        step: Option<f64>,
    },

    #[serde(rename = "boolean")]
    Boolean { default: bool },

    #[serde(rename = "enum")]
    Enum {
        options_source: OptionSource,
        #[serde(default)]
        default: Option<String>,
    },

    #[serde(rename = "path")]
    Path {
        #[serde(default)]
        default: Option<String>,
        #[serde(default)]
        file_type: Option<FileTypeFilter>,
        #[serde(default)]
        must_exist: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OptionSource {
    #[serde(rename = "static")]
    Static { values: Vec<String> },

    #[serde(rename = "script")]
    Script {
        command: String,
        #[serde(default)]
        cache_duration: Option<u64>,
        #[serde(default)]
        depends_on: Vec<String>,
    },

    #[serde(rename = "function")]
    Function { name: String },

    #[serde(rename = "provider")]
    Provider { provider: String },

    #[serde(rename = "file_list")]
    FileList {
        directory: String,
        pattern: String,
        #[serde(default)]
        extract: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UIWidget {
    #[default]
    TextInput,
    NumberInput,
    Toggle,
    Dropdown,
    DropdownSearchable,
    FilePicker,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileTypeFilter {
    Image,
    Json,
    Any,
}
