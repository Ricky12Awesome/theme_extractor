use indexmap::IndexMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Mappings<'a> {
  #[serde(borrow, deserialize_with = "crate::serde_map::skip_nulls_map")]
  pub colors: IndexMap<&'a str, &'a str>,

  #[serde(borrow, deserialize_with = "crate::serde_map::skip_nulls_map")]
  pub attributes: IndexMap<&'a str, &'a str>,
}

const VSCODE_MAPPINGS: &str = include_str!("../mappings/vscode.json");

pub fn from_json_str(str: &str) -> serde_json::Result<Mappings<'_>> {
  serde_json::from_str(str)
}

pub fn vscode_mappings() -> Mappings<'static> {
  from_json_str(VSCODE_MAPPINGS).unwrap()
}
