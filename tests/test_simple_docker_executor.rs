use log::debug;
use waterlib::config::JobConfig;
use waterlib::executors::simple_docker::{SimpleDockerExecutor, SimpleDockerExecutorConfig};
use waterlib::executors::Executor;
use waterlib::net::JobBuildRequestMessage;

#[test]
fn test_simple_exec() {
    let _ = pretty_env_logger::try_init();
    let mut executor = SimpleDockerExecutor::new(SimpleDockerExecutorConfig {});
    let jbrm = JobBuildRequestMessage {
        repo_url: "test".to_string(),
        reference: "main".to_string(),
        job: JobConfig {
            id: "test-id".to_string(),
            name: "test".to_string(),
            image: "busybox".to_string(),
            steps: vec!["echo \"Hello World!\"".to_string()],
        },
        volumes: vec![],
    };
    let res = executor.execute(jbrm);
    debug!("{:?}", res);
    assert!(res.is_ok());
    let res = res.unwrap();
    assert_eq!(res.logs.len(), 1);
}

#[test]
fn test_simple_exec_multiple_steps() {
    let _ = pretty_env_logger::try_init();
    let mut executor = SimpleDockerExecutor::new(SimpleDockerExecutorConfig {});
    let jbrm = JobBuildRequestMessage {
        repo_url: "test".to_string(),
        reference: "main".to_string(),
        job: JobConfig {
            id: "test-id".to_string(),
            name: "test".to_string(),
            image: "busybox".to_string(),
            steps: vec!["echo \"Hello World!\"".to_string()],
        },
        volumes: vec![],
    };
    let res = executor.execute(jbrm);
    debug!("{:?}", res);
    assert!(res.is_ok());
    let res = res.unwrap();
    assert_eq!(res.logs.len(), 2);
}
