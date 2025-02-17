use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum NodeTestingConfig {
    Rust(RustNodeTestingConfig),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RustNodeTestingConfig {
    pub chain_id: String,
    pub initial_time: redux::Timestamp,
}

impl RustNodeTestingConfig {
    pub fn berkeley_default() -> Self {
        Self {
            chain_id: "3c41383994b87449625df91769dff7b507825c064287d30fada9286f3f1cb15e".to_owned(),
            initial_time: redux::Timestamp::ZERO,
        }
    }
}
