pub mod manifest;
pub mod paths;
pub mod payload;
pub mod script;
pub mod step;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputFile {
    pub path: String,
    pub contents: String,
    pub executable: bool,
}
