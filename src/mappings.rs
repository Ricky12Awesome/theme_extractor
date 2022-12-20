use std::{collections::HashMap, hash::Hash};

use serde::{Deserialize, Serialize};

pub type Mappings<'a> = HashMap<&'a str, &'a str>;

#[derive(Serialize, Deserialize)]
struct MappingsSerde<'a> {
  #[serde(flatten, borrow, deserialize_with = "crate::serde_map::skip_nulls_map")]
  mappings: Mappings<'a>,
}

const VSCODE_MAPPINGS: &str = include_str!("../mappings/vscode.json");

pub fn from_json_str(str: &str) -> serde_json::Result<Mappings<'_>> {
  serde_json::from_str::<MappingsSerde>(str).map(|parsed| parsed.mappings)
}

pub fn vscode_mappings() -> Mappings<'static> {
  from_json_str(VSCODE_MAPPINGS).unwrap()
}
