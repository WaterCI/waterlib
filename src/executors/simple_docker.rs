use crate::executors::{generate_docker_name, Executor, JobResult, StepLog};
use crate::utils::docker::{DockerEnv, DockerVolumes};
use anyhow::{Error, Result};
use std::io::{BufRead, BufReader, Write};

use crate::net::JobBuildRequestMessage;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use tracing::{debug, info, instrument, trace};

#[derive(Debug)]
pub struct SimpleDockerExecutorConfig {}

#[derive(Debug)]
pub struct SimpleDockerExecutor {
    pub config: SimpleDockerExecutorConfig,
    pub current_container: String,
    pub code_volume_name: String,
}

const STEP_CHANNEL_PREFIX: &str = "> executing: ";
const STDOUT_CHANNEL_PREFIX: &str = "<1: ";
const STDERR_CHANNEL_PREFIX: &str = "<2: ";

// TODO: limit job execution's time
impl SimpleDockerExecutor {
    pub fn new(config: SimpleDockerExecutorConfig) -> Self {
        SimpleDockerExecutor {
            config,
            current_container: "".to_string(),
            code_volume_name: "".to_string(),
        }
    }
}

impl Executor for SimpleDockerExecutor {
    #[instrument]
    fn execute(&mut self, build_request: JobBuildRequestMessage) -> Result<JobResult> {
        let mut result = JobResult {
            original_request: build_request.clone(),
            logs: vec![],
        };
        let (tx, rx) = mpsc::channel();
        self.execute_with_live_output(build_request, tx)?;
        let mut current_log = StepLog {
            cmd: "".to_string(),
            log: "".to_string(),
        };
        let mut log = String::new();
        for line in rx {
            trace!("execute: recieved \"{}\" on rx", &line);
            if line.starts_with(STEP_CHANNEL_PREFIX) {
                if !log.is_empty() {
                    current_log.log = log;
                    log = String::new();
                    result.logs.push(current_log);
                }
                current_log = StepLog {
                    cmd: line.strip_prefix(STEP_CHANNEL_PREFIX).unwrap().to_string(),
                    log: "".to_string(),
                }
            } else {
                log.push_str(&format!("{}\n", line));
            }
        }
        result.logs.push(current_log);
        Ok(result)
    }

    #[instrument]
    fn execute_with_live_output(
        &mut self,
        build_request: JobBuildRequestMessage,
        sender: Sender<String>,
    ) -> Result<JobResult> {
        let cname = generate_docker_name(&format!("water-{}-", build_request.job.name));
        self.code_volume_name = self.create_job_volume();
        debug!("Created volume {}", &self.code_volume_name);
        let vols = DockerVolumes::from([(self.code_volume_name.clone(), "/code".to_string())]);
        debug!("Initializing container…");
        info!("Creating container {}…", cname);
        init_container(
            &build_request.job.image,
            &cname,
            vols,
            DockerEnv::new(),
            "/code",
        )?;
        debug!("Container {} created.", cname);

        for step in &build_request.job.steps {
            let args = &["start", "-ai", &cname];
            trace!("running cmd: docker {}", &args.join(" "));
            let mut cmd = Command::new("docker")
                .args(args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            let stdin = cmd
                .stdin
                .as_mut()
                .ok_or(Error::msg("could not obtain handle on stdin"))?;
            //let sender = Arc::new(Mutex::new(sender));
            let step_sender = sender.clone();
            let stdout_sender = sender.clone();
            let stderr_sender = sender.clone();
            let stdout = cmd
                .stdout
                .take()
                .ok_or(Error::msg("Could not obtain handle on stdout"))?;
            let stderr = cmd
                .stderr
                .take()
                .ok_or(Error::msg("Could not obtain handle on stderr"))?;

            trace!("launching stdout log_sender_thread…");
            let log_sender_thread_stdout = thread::spawn(move || {
                let sender = stdout_sender;
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    match line {
                        Ok(l) => {
                            trace!("{}{}", STDOUT_CHANNEL_PREFIX, l);
                            sender
                                .send(format!("{}{}", STDOUT_CHANNEL_PREFIX, l))
                                .expect("Could not send");
                        }
                        // FIXME: handle error properly
                        Err(_) => {}
                    }
                }
                trace!("Finished reading container's stdout");
            });
            trace!("launching stderr log_sender_thread…");
            let log_sender_thread_stderr = thread::spawn(move || {
                let sender = stderr_sender;
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    match line {
                        Ok(l) => {
                            trace!("{}{}", STDERR_CHANNEL_PREFIX, l);
                            sender
                                .send(format!("{}{}", STDERR_CHANNEL_PREFIX, l))
                                .expect("Could not send");
                        }
                        // FIXME: handle error properly
                        Err(_) => {}
                    }
                }
                trace!("Finished reading container's stdout");
            });

            step_sender.send(format!("{}{}", STEP_CHANNEL_PREFIX, step))?;
            trace!("sending step to container's stdin: {}", step);
            stdin.write_all(format!("{}\nexit\n", step).as_bytes())?;
            trace!("waiting on container exit…");
            match cmd.wait() {
                Ok(i) => {
                    info!("container exited with status {}, stop & remove it.", i);
                }
                Err(e) => {
                    return Err(anyhow::Error::new(e));
                }
            }
            // TODO: remove the container
            trace!("waiting on log sender threads");
            let _ = log_sender_thread_stdout.join();
            let _ = log_sender_thread_stderr.join();
            // TODO: return proper info?
            trace!("Finished waiting on log sender thread");
        }

        info!("Shutting container down…");
        let mut cmd = Command::new("docker").arg("stop").arg(&cname).spawn()?;
        let _ = cmd.wait();
        let mut cmd = Command::new("docker").arg("rm").arg(&cname).spawn()?;
        let _ = cmd.wait();
        Ok(JobResult {
            original_request: build_request,
            logs: vec![],
        })
    }

    fn get_logs(&self) -> Vec<StepLog> {
        todo!()
    }

    fn get_log_for_step(&self, _step_order: usize) -> Option<String> {
        todo!()
    }

    fn is_busy(&self) -> bool {
        self.current_container.is_empty()
    }

    #[instrument]
    fn create_job_volume(&self) -> String {
        let vol_name = generate_docker_name("waterci-volume-");
        if vol_name.ends_with('_') || vol_name.ends_with('-') {
            // try again
            return self.create_job_volume();
        }
        let cmd = Command::new("docker")
            .args(&["volume", "create", &vol_name])
            .spawn();
        assert!(cmd.is_ok());
        let _ = cmd.unwrap().wait();
        vol_name
    }

    #[instrument]
    fn init_execution(&self) -> Result<bool> {
        todo!()
    }
}

#[instrument]
fn init_container(
    image: &str,
    container_name: &str,
    volumes: DockerVolumes,
    env: DockerEnv,
    docker_work_dir: &str,
) -> Result<()> {
    let mut args = Vec::new();
    args.push("run".to_string());
    args.push("--interactive".to_string());
    args.push(format!("--name={}", container_name));
    args.extend(volumes.iter().map(|(k, v)| format!("--volume={}:{}", k, v)));
    args.extend(env.iter().map(|(k, v)| format!("--env=\"{}={}\"", k, v)));
    args.push(format!("--workdir={}", docker_work_dir));
    args.push("--pull=always".to_string());
    args.push(image.to_string());
    args.push("sh".to_string());
    trace!("running docker {}", &args.join(" "));
    let mut cmd = Command::new("docker")
        .args(args)
        .stdin(Stdio::piped())
        .spawn()?;
    let stdin = cmd
        .stdin
        .as_mut()
        .ok_or(Error::msg("Could not write to docker stdin"))?;
    stdin.write_all(b"exit")?;
    let _ = cmd.wait();
    Ok(())
}
