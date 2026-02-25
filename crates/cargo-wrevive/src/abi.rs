use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AbiJson(pub Vec<AbiItem>);

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AbiItem {
    Function {
        name: String,
        inputs: Vec<AbiParam>,
        outputs: Vec<AbiParam>,
        #[serde(rename = "stateMutability")]
        #[serde(skip_serializing_if = "Option::is_none")]
        state_mutability: Option<String>,
    },
    Constructor {
        inputs: Vec<AbiParam>,
        #[serde(rename = "stateMutability")]
        #[serde(skip_serializing_if = "Option::is_none")]
        state_mutability: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AbiParam {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
}

/// Generate ABI for the given binary. Returns Ok(None) if no ABI generation is configured.
pub fn generate_abi_for_bin(_manifest_dir: &Path, _bin_name: &str) -> Result<Option<AbiJson>> {
    Ok(None)
}
