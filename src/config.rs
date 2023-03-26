use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
  pub(crate) host: String,
  pub(crate) path: String,
  pub(crate) directory: String,
  pub(crate) icons: Option<HashMap<String, String>>,
}
