// use crate::net::TCPSerializable;
use crate::utils;
use serde::{Deserialize, Serialize};
use utils::gen_uuid;

/// The file in a repository to configure the different steps needed to build the project
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct RepositoryConfig {
    #[serde(default = "gen_uuid")]
    pub id: String,
    pub jobs: Vec<JobConfig>,
}
/// A single job config
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct JobConfig {
    #[serde(default = "gen_uuid")]
    pub id: String,
    pub name: String,
    pub image: String,
    pub steps: Vec<String>,
}
