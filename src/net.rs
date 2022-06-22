use crate::config::{JobConfig, RepositoryConfig};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[cfg(test)]
mod tests {}

/// Represents a message to chat with the core
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ExecutorMessage {
    BuildRequest(BuildRequestFromRepo),
    JobResult(JobResult),
    ExecutorRegister,
    ExecutorRegisterResponse { id: String },
    ExecutorStatusQuery,
    ExecutorStatusResponse(ExecutorStatus),
    CloseConnection(Option<String>),
}

impl ExecutorMessage {
    pub fn from<T: Read>(reader: T) -> anyhow::Result<Self> {
        Ok(ExecutorMessage::deserialize(
            &mut rmp_serde::Deserializer::new(reader),
        )?)
    }
    pub fn write<T: Write>(&self, writer: T) -> anyhow::Result<()> {
        self.serialize(&mut rmp_serde::Serializer::new(writer))?;
        Ok(())
    }
}

#[derive(PartialOrd, PartialEq, Debug, Clone, Deserialize, Serialize)]
pub enum ExecutorStatus {
    YetToRegister,
    Available,
    Running,
}

/// Represents the data contained in a build request.
/// the repo url, the job to run, eventual volume data…
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobBuildRequestMessage {
    pub repo_url: String,
    pub reference: String,
    pub job: JobConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
pub enum IncomingMessage {
    BuildRequestFromRepo(BuildRequestFromRepo),
}

impl IncomingMessage {
    pub fn from<T: Read>(reader: T) -> anyhow::Result<Self> {
        Ok(IncomingMessage::deserialize(
            serde_yaml::Deserializer::from_reader(reader),
        )?)
    }
    pub fn write<T: Write>(&self, writer: T) -> anyhow::Result<()> {
        self.serialize(&mut serde_yaml::Serializer::new(writer))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct BuildRequestFromRepo {
    pub repo_url: String,
    pub reference: String,
    pub repo_config: RepositoryConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobResult {
    pub original_request: JobBuildRequestMessage,
    pub logs: Vec<StepLog>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StepLog {
    pub cmd: String,
    pub log: String,
}
