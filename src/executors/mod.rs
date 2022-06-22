use crate::net::{JobBuildRequestMessage, JobResult, StepLog};
use rand::Rng;
use std::sync::mpsc::Sender;

const DOCKER_NAME_CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz-_0123456789";
const FORBIDDEN_START_CHARS: &[char] = &['-', '_'];

/// Represents something that will execute a Job.
pub trait Executor {
    /// Execute the given JobConfig, returning a JobResult
    fn execute(&mut self, build_request: JobBuildRequestMessage) -> anyhow::Result<JobResult>;
    /// Execute the given JobConfig, returning a JobResult and writing all logs to the given Sender
    fn execute_with_live_output(
        &mut self,
        build_request: JobBuildRequestMessage,
        sender: Sender<String>,
    ) -> anyhow::Result<JobResult>;
    fn get_logs(&self) -> Vec<StepLog>;
    fn get_log_for_step(&self, step_order: usize) -> Option<String>;
    fn is_busy(&self) -> bool;
    /// Creates the volume to be attached to all containers.
    fn create_job_volume(&self) -> String;
    /// Execution initialization: clone the project on the volume.
    fn init_execution(&self) -> anyhow::Result<bool>;
}

pub fn generate_docker_name(prefix: &str) -> String {
    let mut rng = rand::thread_rng();
    let s = format!(
        "{}{}",
        prefix,
        (0..16)
            .map(|_| { DOCKER_NAME_CHARSET[rng.gen_range(0..DOCKER_NAME_CHARSET.len())] as char })
            .collect::<String>()
    );
    if s.starts_with(FORBIDDEN_START_CHARS) || s.ends_with(FORBIDDEN_START_CHARS) {
        return generate_docker_name(prefix);
    }
    s
}

/// A simple docker executors. Calls `docker` on the calling machine
pub mod simple_docker;
