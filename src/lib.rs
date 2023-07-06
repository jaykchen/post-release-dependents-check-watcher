use chrono::{DateTime, Utc};
use flowsnet_platform_sdk::write_error_log;
use github_flows::{get_octo, GithubLogin};
use schedule_flows::schedule_cron_job;
use serde::Deserialize;
use store_flows::{get, set};

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() {
    schedule_cron_job(
        String::from("30 * * * *"),
        String::from("cron_job_evoked"),
        handler,
    )
    .await;
}

async fn handler(_: Vec<u8>) {
    let owner_repo_workflow_id_obj = vec![
        ("second-state", "microservice-rust-mysql", "ci.yml"),
        ("second-state", "wasmedge-quickjs", "examples.yml"),
    ];

    for (owner, repo, workflow_id) in owner_repo_workflow_id_obj {
        if let Ok(true) = workflow_run_success(owner, repo, workflow_id).await {
            let key = &format!("{owner}-{repo}");
            match get(key) {
                Some(v) => match v.as_bool() {
                    Some(false) => set(key, serde_json::json!(true), None),
                    _ => {}
                },
                None => set(key, serde_json::json!(false), None),
            }
        }
    }
}

pub async fn workflow_run_success(
    owner: &str,
    repo: &str,
    workflow_id: &str,
) -> anyhow::Result<bool> {
    let octocrab = get_octo(&GithubLogin::Default);
    let route = format!("repos/{owner}/{repo}/actions/workflows/{workflow_id}/runs");
    // let route = format!("repos/jaykchen/chatgpt-private-test/actions/workflows/rust.yml/runs");

    let res: WorkflowRunPayload = octocrab.get(route, None::<&()>).await?;
    let is_dipatch_event = res.workflow_runs[0].event == "workflow_dispatch";
    let is_completed = res.workflow_runs[0].status == "completed";
    let is_success = res.workflow_runs[0].conclusion == "success";
    Ok(is_dipatch_event && is_completed && is_success)
}

#[derive(Deserialize)]
pub struct WorkflowRun {
    id: u64,
    name: String,
    path: String,
    display_title: String,
    run_number: u32,
    event: String,
    status: String,
    conclusion: String,
    workflow_id: u64,
    url: String,
    html_url: String,
    pull_requests: Vec<String>, // Change this type based on your pull request structure
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct WorkflowRunPayload {
    total_count: u32,
    workflow_runs: Vec<WorkflowRun>,
}
