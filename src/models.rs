use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileContext {
  pub path: String,
  pub icon: Option<String>,
  pub is_directory: bool,
  pub is_file: bool,
  pub last_modified: String,
  pub size: String,
  pub name: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct DirTemplateContext {
  pub files: Vec<FileContext>,
}
