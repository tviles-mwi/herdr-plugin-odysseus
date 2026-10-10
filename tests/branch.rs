//! Per-task branch names: `--branch` names the run branch; invalid or
//! existing names are refused when the task is created.

mod common;

use common::*;
use herdr_orchestrator::engine::{create_task, NewTask};
use herdr_orchestrator::model::*;

const WF: &str = "  - id: implement\n    type: agent\n    runner: fake-success\n    prompt: \"{{task}}\"\n";

#[test]
fn explicit_branch_names_the_run_branch() {
    let mut h = Harness::new();
    h.workflow("one", WF);
    let t = h.task_with(
        "Keep selection",
        TaskOptions { workflow: Some("one".into()), branch: Some("fix/1742-keep-selection".into()), ..Default::default() },
    );
    let runs = h.settle(&t.task_id);
    assert_eq!(runs[0].status, RunStatus::Succeeded, "{:?}", runs[0].status_reason);
    assert_eq!(runs[0].git.branch.as_deref(), Some("fix/1742-keep-selection"));
}

#[test]
fn invalid_or_existing_branch_is_refused_at_create() {
    let h = Harness::new();
    h.workflow("one", WF);
    let mk = |b: &str| {
        create_task(
            &h.ctx,
            NewTask {
                text: "x".into(),
                repo: h.repo.clone(),
                options: TaskOptions { workflow: Some("one".into()), branch: Some(b.into()), ..Default::default() },
                via: "test".into(),
                ..Default::default()
            },
        )
    };
    assert!(format!("{:#}", mk("bad branch").unwrap_err()).contains("invalid --branch"));
    let out = std::process::Command::new("git").args(["branch", "fix/taken"]).current_dir(&h.repo).output().unwrap();
    assert!(out.status.success());
    assert!(format!("{:#}", mk("fix/taken").unwrap_err()).contains("already exists"));
    // Taken on a remote only (local branch deleted after its PR merged): still refused,
    // or a re-run would push onto the old branch and record its old PR.
    let out = std::process::Command::new("git").args(["remote", "add", "origin", "https://example.invalid/r.git"]).current_dir(&h.repo).output().unwrap();
    assert!(out.status.success());
    let out = std::process::Command::new("git").args(["update-ref", "refs/remotes/origin/fix/remote-taken", "HEAD"]).current_dir(&h.repo).output().unwrap();
    assert!(out.status.success());
    let e = format!("{:#}", mk("fix/remote-taken").unwrap_err());
    assert!(e.contains("already exists on origin"), "{e}");
}
