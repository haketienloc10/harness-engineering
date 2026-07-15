use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn run(cli: &Path, database: &Path, args: &[&str]) -> Output {
    Command::new(cli)
        .args(args)
        .current_dir(workspace_root())
        .env("HARNESS_REPO_ROOT", workspace_root())
        .env("HARNESS_DB", database)
        .output()
        .unwrap()
}

fn run_ok(cli: &Path, database: &Path, args: &[&str]) -> Output {
    let output = run(cli, database, args);
    assert!(
        output.status.success(),
        "command failed: {:?}\nstdout: {}\nstderr: {}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn start_tiny(cli: &Path, database: &Path, session: &str) -> String {
    let output = run_ok(
        cli,
        database,
        &[
            "task",
            "start",
            "--type",
            "change request",
            "--summary",
            "Phase 4 packaged failure fixture",
            "--owner",
            "codex",
            "--session",
            session,
            "--behavior-bearing",
            "no",
            "--json",
        ],
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn start_expanded(cli: &Path, database: &Path, story: &str, session: &str, flags: &str) -> String {
    run_ok(
        cli,
        database,
        &[
            "story",
            "add",
            "--id",
            story,
            "--title",
            "Phase 4 packaged story",
            "--lane",
            "normal",
        ],
    );
    let output = run_ok(
        cli,
        database,
        &[
            "task",
            "start",
            "--type",
            "change request",
            "--summary",
            "Phase 4 packaged expanded fixture",
            "--owner",
            "codex",
            "--session",
            session,
            "--story",
            story,
            "--flags",
            flags,
            "--behavior-bearing",
            "yes",
            "--json",
        ],
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn task_snapshot(database: &Path, id: &str) -> Option<String> {
    Connection::open(database)
        .unwrap()
        .query_row(
            "SELECT printf('%s|%s|%s|%s|%s|%s|%s|%s', status,
                coalesce(outcome,''), coalesce(closed_at,''), updated_at,
                coalesce(capsule_path,''), coalesce(capsule_checksum,''),
                coalesce(capsule_omission_reason,''), coalesce(closure_nonce,''))
             FROM task WHERE id=?1;",
            params![id],
            |row| row.get(0),
        )
        .optional()
        .unwrap()
}

fn capsule_snapshot() -> Vec<(String, String)> {
    fn visit(root: &Path, current: &Path, files: &mut Vec<(String, String)>) {
        let Ok(entries) = fs::read_dir(current) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, files);
            } else if path.is_file() {
                let bytes = fs::read(&path).unwrap();
                files.push((
                    path.strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                    format!("{:x}", Sha256::digest(bytes)),
                ));
            }
        }
    }
    let root = workspace_root().join("docs/tasks");
    let mut files = Vec::new();
    visit(&root, &root, &mut files);
    files.sort();
    files
}

struct PreparedCase {
    _temp: TempDir,
    database: PathBuf,
    task_id: String,
    args: Vec<String>,
}

type Prepare = fn(&Path) -> PreparedCase;

fn base(cli: &Path) -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let database = temp.path().join("harness.db");
    run_ok(cli, &database, &["init"]);
    (temp, database)
}

fn finish_args(id: &str, owner: &str, session: &str, friction: &str) -> Vec<String> {
    [
        "task",
        "finish",
        "--id",
        id,
        "--owner",
        owner,
        "--session",
        session,
        "--trace",
        "999999",
        "--outcome",
        "completed",
        "--friction",
        friction,
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn missing_task(cli: &Path) -> PreparedCase {
    let (temp, database) = base(cli);
    let task_id = "TASK-999999".to_owned();
    PreparedCase {
        _temp: temp,
        args: finish_args(&task_id, "codex", "missing", "none"),
        database,
        task_id,
    }
}

fn explicit_friction(cli: &Path) -> PreparedCase {
    let (temp, database) = base(cli);
    let task_id = start_tiny(cli, &database, "phase4-friction");
    PreparedCase {
        _temp: temp,
        args: finish_args(&task_id, "codex", "phase4-friction", "backlog"),
        database,
        task_id,
    }
}

fn unmet_context(cli: &Path) -> PreparedCase {
    let (temp, database) = base(cli);
    let task_id = start_tiny(cli, &database, "phase4-context");
    PreparedCase {
        _temp: temp,
        args: finish_args(&task_id, "codex", "phase4-context", "none"),
        database,
        task_id,
    }
}

fn missing_proof(cli: &Path) -> PreparedCase {
    let (temp, database) = base(cli);
    let task_id = start_tiny(cli, &database, "phase4-proof");
    run_ok(
        cli,
        &database,
        &[
            "task",
            "context",
            "acknowledge",
            "--id",
            &task_id,
            "--read",
            "<changed-files>",
        ],
    );
    PreparedCase {
        _temp: temp,
        args: finish_args(&task_id, "codex", "phase4-proof", "none"),
        database,
        task_id,
    }
}

fn missing_trace(cli: &Path) -> PreparedCase {
    let prepared = missing_proof(cli);
    run_ok(
        cli,
        &prepared.database,
        &[
            "proof",
            "run",
            "--task",
            &prepared.task_id,
            "--layer",
            "quick",
            "--",
            "git",
            "--version",
        ],
    );
    prepared
}

fn owner_conflict(cli: &Path) -> PreparedCase {
    let (temp, database) = base(cli);
    let task_id = start_tiny(cli, &database, "phase4-owner");
    PreparedCase {
        _temp: temp,
        args: finish_args(&task_id, "reviewer", "phase4-owner", "none"),
        database,
        task_id,
    }
}

fn capsule_required(cli: &Path) -> PreparedCase {
    let (temp, database) = base(cli);
    let task_id = start_expanded(
        cli,
        &database,
        "CL-PACKAGED-NORMAL",
        "phase4-capsule",
        "public-contract,weak-proof",
    );
    PreparedCase {
        _temp: temp,
        args: finish_args(&task_id, "codex", "phase4-capsule", "none"),
        database,
        task_id,
    }
}

fn approval_required(cli: &Path) -> PreparedCase {
    let (temp, database) = base(cli);
    let task_id = start_expanded(
        cli,
        &database,
        "CL-PACKAGED-HIGH",
        "phase4-approval",
        "auth",
    );
    PreparedCase {
        _temp: temp,
        args: finish_args(&task_id, "codex", "phase4-approval", "none"),
        database,
        task_id,
    }
}

fn run_matrix(cli: &Path) {
    struct Case {
        name: &'static str,
        prepare: Prepare,
        exit: i32,
        code: &'static str,
    }
    let cases = [
        Case {
            name: "missing task root",
            prepare: missing_task,
            exit: 5,
            code: "TASK_NOT_FOUND",
        },
        Case {
            name: "explicit friction",
            prepare: explicit_friction,
            exit: 5,
            code: "TASK_FRICTION_UNRESOLVED",
        },
        Case {
            name: "unmet context",
            prepare: unmet_context,
            exit: 5,
            code: "TASK_CONTEXT_UNMET",
        },
        Case {
            name: "missing proof",
            prepare: missing_proof,
            exit: 5,
            code: "TASK_PROOF_MISSING",
        },
        Case {
            name: "missing trace",
            prepare: missing_trace,
            exit: 5,
            code: "TASK_TRACE_MISSING",
        },
        Case {
            name: "owner conflict",
            prepare: owner_conflict,
            exit: 8,
            code: "TASK_OWNERSHIP_CONFLICT",
        },
        Case {
            name: "missing capsule",
            prepare: capsule_required,
            exit: 5,
            code: "TASK_CAPSULE_REQUIRED",
        },
        Case {
            name: "missing approval",
            prepare: approval_required,
            exit: 9,
            code: "TASK_APPROVAL_REQUIRED",
        },
    ];

    for case in cases {
        let prepared = (case.prepare)(cli);
        let before_task = task_snapshot(&prepared.database, &prepared.task_id);
        let before_capsules = capsule_snapshot();
        let mut json_args = prepared.args.clone();
        json_args.push("--json".to_owned());
        let json_refs = json_args.iter().map(String::as_str).collect::<Vec<_>>();

        let json_output = run(cli, &prepared.database, &json_refs);
        assert_eq!(json_output.status.code(), Some(case.exit), "{}", case.name);
        let result: Value = serde_json::from_slice(&json_output.stderr).unwrap_or_else(|error| {
            panic!(
                "{} did not return JSON: {error}: {}",
                case.name,
                String::from_utf8_lossy(&json_output.stderr)
            )
        });
        assert_eq!(result["ok"], false, "{}", case.name);
        assert_eq!(result["code"], case.code, "{}", case.name);
        assert!(
            result["remediation"]
                .as_array()
                .is_some_and(|items| !items.is_empty()),
            "{}",
            case.name
        );

        let human_refs = prepared.args.iter().map(String::as_str).collect::<Vec<_>>();
        let human_output = run(cli, &prepared.database, &human_refs);
        assert_eq!(human_output.status.code(), Some(case.exit), "{}", case.name);
        let human = String::from_utf8(human_output.stderr).unwrap();
        assert!(human.contains(&format!("error: {}", result["code"].as_str().unwrap())));
        assert!(human.contains(result["message"].as_str().unwrap()));
        for remediation in result["remediation"].as_array().unwrap() {
            assert!(
                human.contains(remediation.as_str().unwrap()),
                "{}",
                case.name
            );
        }

        assert_eq!(
            task_snapshot(&prepared.database, &prepared.task_id),
            before_task,
            "{} changed task closure state",
            case.name
        );
        assert_eq!(
            capsule_snapshot(),
            before_capsules,
            "{} changed capsule files",
            case.name
        );
    }
}

#[test]
fn source_cli_phase4_failure_matrix() {
    run_matrix(Path::new(env!("CARGO_BIN_EXE_harness-cli")));
}

#[test]
#[ignore = "run after install-harness-cli.sh to validate the packaged binary"]
fn packaged_cli_phase4_failure_matrix() {
    run_matrix(&workspace_root().join("_harness/bin/harness-cli"));
}
