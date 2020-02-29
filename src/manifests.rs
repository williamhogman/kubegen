use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::iter::FromIterator;

#[derive(Serialize)]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub labels: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

impl Metadata {
    pub fn name_only(name: &str) -> Metadata {
        Metadata {
            namespace: None,
            name: name.to_owned(),
            labels: BTreeMap::new(),
            metadata: BTreeMap::new(),
        }
    }
}

fn kube_manifest(
    api_version: &str,
    kind: &str,
    metadata: Metadata,
    other: &serde_json::Map<String, Value>,
) -> Value {
    let mut j = json!({ "api_version": api_version, "kind": kind, "metadata": metadata});
    let target = j.as_object_mut().unwrap();
    target.extend(other.to_owned());
    return Value::Object(target.to_owned());
}

pub fn config_map(metadata: Metadata, vals: impl Iterator<Item = (String, String)>) -> Value {
    let pairs = vals.map({ |(k, v)| (k, Value::String(v)) });
    let rsx = json!({ "data": serde_json::Map::from_iter(pairs) });
    kube_manifest("v1", "ConfigMap", metadata, rsx.as_object().unwrap())
}
