use std::collections::{BTreeMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{
    params, types::ValueRef, Connection, OpenFlags, OptionalExtension, TransactionBehavior,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::application::{
    BacklogAddInput, BacklogCloseInput, BrownfieldImportResult, DecisionAddInput,
    DecisionVerifyResult, HarnessContext, InitResult, IntakeInput, InterventionAddInput,
    InterventionFilter, MigrateResult, QueryTable, StoryAddInput, StoryUpdateInput,
    StoryVerifyResult, ToolRegisterInput, TraceInput,
};
use crate::domain::{
    compiled_tool_registry, infer_context_phase, jsonish_list, normalize_token, score_context,
    score_trace, validate_tool_description, AuditFinding, AuditResult, BacklogFilter,
    BacklogRecord, ContextScoreResult, ContextScoreSource, DecisionRecord, FrictionRecord,
    HarnessStats, ImprovementProposal, IntakeRecord, InterventionRecord, RiskLane,
    StoryMatrixRecord, StoryVerifyAllItem, StoryVerifyAllResult, StoryVerifyStatus, ToolArgSpec,
    ToolEntry, TraceRecord, TraceScoreResult, TraceScoreSource,
};

pub type Result<T> = std::result::Result<T, HarnessInfraError>;

#[derive(Debug, Error)]
pub enum HarnessInfraError {
    #[error("database not found at {0}. Run: harness init")]
    MissingDatabase(String),
    #[error("schema file missing: {0}")]
    MissingSchema(String),
    #[error("brownfield import: missing {0}")]
    MissingBrownfieldPath(String),
    #[error("decision {0} has no verify_command. Configure one with: harness-cli decision add --id {0} --title <title> --verify \"<command>\"")]
    MissingDecisionVerifyCommand(String),
    #[error("story {0} has no verify_command. Configure one with: harness-cli story update --id {0} --verify \"<command>\"")]
    MissingStoryVerifyCommand(String),
    #[error("story update: story '{0}' not found")]
    StoryNotFound(String),
    #[error("tool register: tool '{0}' already exists with command '{1}'")]
    ToolAlreadyExists(String, String),
    #[error("tool remove: tool '{0}' not found")]
    ToolNotFound(String),
    #[error("tool register: command '{0}' was not found. Re-run with --force to register anyway.")]
    ToolCommandNotFound(String),
    #[error("{0}")]
    ToolValidation(#[from] crate::domain::ToolValidationError),
    #[error("backlog close: backlog item '{0}' not found")]
    BacklogNotFound(i64),
    #[error("trace '{0}' not found")]
    TraceNotFound(i64),
    #[error("no traces found")]
    NoTraces,
    #[error("story update: nothing to update")]
    EmptyStoryUpdate,
    #[error(
        "query sql only permits a single read-only SELECT, WITH, or diagnostic PRAGMA statement"
    )]
    UnsafeSql,
    #[error("ensure refused unsafe durable state: {0}")]
    UnsafeDurableState(String),
    #[error("database backup failed: {0}")]
    BackupFailed(String),
    #[error("workflow policy invalid: {0}")]
    WorkflowInvalid(String),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Outcome of one `tool check` scan. The CLI reports these facts; the agent
/// applies policy (skip / degrade / use) based on `status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCheckResult {
    pub name: String,
    pub kind: String,
    pub capability: Option<String>,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorReport {
    pub ok: bool,
    pub code: String,
    pub message: String,
    pub platform: String,
    pub repository_id: Option<String>,
    pub worktree: Option<String>,
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub source_versions: Vec<i64>,
    pub db_versions: Vec<i64>,
    pub findings: Vec<String>,
    pub remediation: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowPolicy {
    pub policy_version: String,
    pub policy_id: String,
    pub mode: String,
    pub repository: WorkflowRepository,
    pub lanes: WorkflowLanes,
    pub classification: WorkflowClassification,
    pub approvals: WorkflowApprovals,
    pub friction: WorkflowFriction,
    pub context: WorkflowContext,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowRepository {
    pub product_docs: String,
    pub stories: String,
    pub decisions: String,
    pub tasks: String,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowLanes {
    pub tiny: WorkflowLane,
    pub normal: WorkflowLane,
    pub high_risk: WorkflowLane,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowLane {
    pub trace_tier: String,
    pub story: String,
    pub proof: Vec<String>,
    pub capsule: String,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowClassification {
    pub normal_min_flags: usize,
    pub high_risk_min_flags: usize,
    pub hard_gates: Vec<String>,
    #[serde(default)]
    pub aliases: BTreeMap<String, String>,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowApprovals {
    pub required_for: Vec<String>,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowFriction {
    pub allowed_dispositions: Vec<String>,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowContext {
    pub stop_condition: String,
    pub token_budget: WorkflowContextTokenBudget,
    pub rules: Vec<WorkflowContextRule>,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowContextTokenBudget {
    pub tiny: usize,
    pub normal: usize,
    pub high_risk: usize,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowContextRule {
    pub id: String,
    #[serde(default)]
    pub phases: Vec<String>,
    #[serde(default)]
    pub lanes: Vec<String>,
    #[serde(default)]
    pub when_paths: Vec<String>,
    #[serde(default)]
    pub when_flags: Vec<String>,
    #[serde(default)]
    pub always: bool,
    #[serde(default)]
    pub must_read: Vec<String>,
    #[serde(default)]
    pub should_read: Vec<String>,
    #[serde(default)]
    pub skip: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowContextEntry {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowContextManifest {
    pub policy_id: String,
    pub policy_version: String,
    pub policy_mode: String,
    pub lane: String,
    pub phase: String,
    pub must_read: Vec<WorkflowContextEntry>,
    pub should_read: Vec<WorkflowContextEntry>,
    pub skip: Vec<WorkflowContextEntry>,
    pub stop_condition: String,
    pub token_budget_hint: usize,
    pub checksum: String,
}

impl WorkflowPolicy {
    pub fn classify(&self, flags: &[String]) -> (String, Vec<String>) {
        let normalized = flags
            .iter()
            .map(|flag| normalize_token(flag))
            .map(|flag| {
                self.classification
                    .aliases
                    .get(&flag)
                    .cloned()
                    .unwrap_or(flag)
            })
            .filter(|flag| !flag.is_empty())
            .collect::<Vec<_>>();
        let hard = normalized
            .iter()
            .filter(|flag| {
                self.classification
                    .hard_gates
                    .iter()
                    .any(|gate| normalize_token(gate) == **flag)
            })
            .cloned()
            .collect::<Vec<_>>();
        let lane =
            if !hard.is_empty() || normalized.len() >= self.classification.high_risk_min_flags {
                "high_risk"
            } else if normalized.len() >= self.classification.normal_min_flags {
                "normal"
            } else {
                "tiny"
            };
        let lane_policy = match lane {
            "high_risk" => &self.lanes.high_risk,
            "normal" => &self.lanes.normal,
            _ => &self.lanes.tiny,
        };
        let mut gates = lane_policy.proof.clone();
        if lane_policy.story == "required" {
            gates.push("story".to_owned());
        }
        gates.extend(hard.into_iter().map(|flag| format!("hard-gate:{flag}")));
        (lane.to_owned(), gates)
    }

    pub fn context_manifest(
        &self,
        lane: &str,
        phase: &str,
        paths: &[String],
        flags: &[String],
        linked_artifacts: &[String],
    ) -> WorkflowContextManifest {
        let lane = normalize_token(lane);
        let phase = normalize_token(phase);
        let normalized_flags = flags
            .iter()
            .map(|flag| normalize_token(flag))
            .map(|flag| {
                self.classification
                    .aliases
                    .get(&flag)
                    .cloned()
                    .unwrap_or(flag)
            })
            .collect::<Vec<_>>();
        let all_paths = paths
            .iter()
            .chain(linked_artifacts)
            .cloned()
            .collect::<Vec<_>>();
        let mut must_read = Vec::new();
        let mut should_read = Vec::new();
        let mut skip = Vec::new();
        for rule in &self.context.rules {
            let lane_matches = rule.lanes.is_empty()
                || rule
                    .lanes
                    .iter()
                    .any(|value| normalize_token(value) == lane);
            let phase_matches = rule.phases.is_empty()
                || rule
                    .phases
                    .iter()
                    .any(|value| normalize_token(value) == phase);
            let paths_match = rule.when_paths.is_empty()
                || all_paths.iter().any(|path| {
                    rule.when_paths
                        .iter()
                        .any(|pattern| glob_matches(pattern, path))
                });
            let flags_match = rule.when_flags.is_empty()
                || normalized_flags.iter().any(|flag| {
                    rule.when_flags
                        .iter()
                        .any(|expected| normalize_token(expected) == *flag)
                });
            let has_trigger =
                rule.always || !rule.when_paths.is_empty() || !rule.when_flags.is_empty();
            if lane_matches && phase_matches && has_trigger && paths_match && flags_match {
                for path in &rule.must_read {
                    promote_context_entry(
                        path,
                        &rule.id,
                        &mut must_read,
                        &mut should_read,
                        &mut skip,
                        ContextPriority::Must,
                    );
                }
                for path in &rule.should_read {
                    promote_context_entry(
                        path,
                        &rule.id,
                        &mut must_read,
                        &mut should_read,
                        &mut skip,
                        ContextPriority::Should,
                    );
                }
                for path in &rule.skip {
                    promote_context_entry(
                        path,
                        &rule.id,
                        &mut must_read,
                        &mut should_read,
                        &mut skip,
                        ContextPriority::Skip,
                    );
                }
            }
        }
        let token_budget_hint = match lane.as_str() {
            "high_risk" => self.context.token_budget.high_risk,
            "normal" => self.context.token_budget.normal,
            _ => self.context.token_budget.tiny,
        };
        let mut manifest = WorkflowContextManifest {
            policy_id: self.policy_id.clone(),
            policy_version: self.policy_version.clone(),
            policy_mode: self.mode.clone(),
            lane,
            phase,
            must_read,
            should_read,
            skip,
            stop_condition: self.context.stop_condition.clone(),
            token_budget_hint,
            checksum: String::new(),
        };
        manifest.checksum = context_manifest_checksum(&manifest);
        manifest
    }
}

fn glob_matches(pattern: &str, path: &str) -> bool {
    fn matches(pattern: &[u8], value: &[u8]) -> bool {
        match pattern {
            [] => value.is_empty(),
            [b'*', b'*', rest @ ..] => {
                matches(rest, value) || (!value.is_empty() && matches(pattern, &value[1..]))
            }
            [b'*', rest @ ..] => {
                matches(rest, value)
                    || (!value.is_empty() && value[0] != b'/' && matches(pattern, &value[1..]))
            }
            [expected, rest @ ..] => {
                !value.is_empty() && *expected == value[0] && matches(rest, &value[1..])
            }
        }
    }
    matches(pattern.as_bytes(), path.as_bytes())
}

#[derive(Clone, Copy)]
enum ContextPriority {
    Must,
    Should,
    Skip,
}

fn promote_context_entry(
    path: &str,
    reason: &str,
    must_read: &mut Vec<WorkflowContextEntry>,
    should_read: &mut Vec<WorkflowContextEntry>,
    skip: &mut Vec<WorkflowContextEntry>,
    priority: ContextPriority,
) {
    if must_read.iter().any(|entry| entry.path == path) {
        return;
    }
    if matches!(priority, ContextPriority::Must) {
        should_read.retain(|entry| entry.path != path);
        skip.retain(|entry| entry.path != path);
        must_read.push(WorkflowContextEntry {
            path: path.to_owned(),
            reason: reason.to_owned(),
        });
        return;
    }
    if should_read.iter().any(|entry| entry.path == path) {
        return;
    }
    if matches!(priority, ContextPriority::Should) {
        skip.retain(|entry| entry.path != path);
        should_read.push(WorkflowContextEntry {
            path: path.to_owned(),
            reason: reason.to_owned(),
        });
        return;
    }
    if !skip.iter().any(|entry| entry.path == path) {
        skip.push(WorkflowContextEntry {
            path: path.to_owned(),
            reason: reason.to_owned(),
        });
    }
}

fn context_manifest_checksum(manifest: &WorkflowContextManifest) -> String {
    let entries = |values: &[WorkflowContextEntry]| {
        values
            .iter()
            .map(|entry| format!("{}:{}", entry.path, entry.reason))
            .collect::<Vec<_>>()
            .join("|")
    };
    let canonical = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        manifest.policy_id,
        manifest.policy_version,
        manifest.policy_mode,
        manifest.lane,
        manifest.phase,
        entries(&manifest.must_read),
        entries(&manifest.should_read),
        entries(&manifest.skip),
        manifest.stop_condition,
        manifest.token_budget_hint,
    );
    format!("{:x}", Sha256::digest(canonical.as_bytes()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceMigration {
    version: i64,
    path: PathBuf,
    checksum: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MigrationManifest {
    lineage: String,
    migrations: Vec<ManifestMigration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ManifestMigration {
    version: i64,
    path: String,
    checksum: String,
}

type RepositoryProvenance = (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Vec<String>,
);

pub trait HarnessRepository {
    fn doctor(&self) -> Result<DoctorReport>;
    fn workflow_policy(&self) -> Result<WorkflowPolicy>;
    fn ensure(&self) -> Result<MigrateResult>;
    fn init(&self) -> Result<InitResult>;
    fn migrate(&self) -> Result<MigrateResult>;
    fn import_brownfield(&self) -> Result<BrownfieldImportResult>;
    fn record_intake(&self, input: IntakeInput) -> Result<i64>;
    fn add_story(&self, input: StoryAddInput) -> Result<()>;
    fn update_story(&self, input: StoryUpdateInput) -> Result<()>;
    fn verify_story(&self, id: &str) -> Result<StoryVerifyResult>;
    fn verify_all_stories(&self) -> Result<StoryVerifyAllResult>;
    fn add_decision(&self, input: DecisionAddInput) -> Result<()>;
    fn verify_decision(&self, id: &str) -> Result<DecisionVerifyResult>;
    fn add_backlog(&self, input: BacklogAddInput) -> Result<i64>;
    fn close_backlog(&self, input: BacklogCloseInput) -> Result<()>;
    fn register_tool(&self, input: ToolRegisterInput) -> Result<()>;
    fn remove_tool(&self, name: &str) -> Result<()>;
    fn check_tools(&self, name: Option<String>) -> Result<Vec<ToolCheckResult>>;
    fn add_intervention(&self, input: InterventionAddInput) -> Result<i64>;
    fn record_trace(&self, input: TraceInput) -> Result<i64>;
    fn score_trace(&self, id: Option<i64>) -> Result<TraceScoreResult>;
    fn score_context(&self, id: i64) -> Result<ContextScoreResult>;
    fn story_verify_status(&self, id: &str) -> Result<StoryVerifyStatus>;
    fn query_matrix(&self) -> Result<Vec<StoryMatrixRecord>>;
    fn query_backlog(&self, filter: BacklogFilter) -> Result<Vec<BacklogRecord>>;
    fn query_decisions(&self) -> Result<Vec<DecisionRecord>>;
    fn query_intakes(&self) -> Result<Vec<IntakeRecord>>;
    fn query_traces(&self) -> Result<Vec<TraceRecord>>;
    fn query_friction(&self) -> Result<Vec<FrictionRecord>>;
    fn query_tools(
        &self,
        responsibility: Option<String>,
        capability: Option<String>,
    ) -> Result<Vec<ToolEntry>>;
    fn query_interventions(&self, filter: InterventionFilter) -> Result<Vec<InterventionRecord>>;
    fn query_stats(&self) -> Result<HarnessStats>;
    fn audit(&self) -> Result<AuditResult>;
    fn propose(&self, commit: bool) -> Result<Vec<ImprovementProposal>>;
    fn query_sql(&self, sql: &str) -> Result<QueryTable>;
}

#[derive(Debug)]
pub struct SqliteHarnessRepository {
    repo_root: PathBuf,
    db_path: PathBuf,
    schema_dir: PathBuf,
}

impl SqliteHarnessRepository {
    pub fn new(repo_root: PathBuf, db_path: PathBuf, schema_dir: PathBuf) -> Self {
        Self {
            repo_root,
            db_path,
            schema_dir,
        }
    }

    fn open_existing(&self) -> Result<Connection> {
        if !self.db_path.exists() {
            return Err(HarnessInfraError::MissingDatabase(
                self.db_path.display().to_string(),
            ));
        }

        let connection = Connection::open(&self.db_path)?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        Ok(connection)
    }

    #[cfg(test)]
    fn open_or_create(&self) -> Result<Connection> {
        let connection = Connection::open(&self.db_path)?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        Ok(connection)
    }

    #[cfg(test)]
    fn schema_version(connection: &Connection) -> Result<i64> {
        let version = connection
            .query_row(
                "SELECT COALESCE(MAX(version),0) FROM schema_version;",
                [],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .unwrap_or(0);
        Ok(version)
    }

    fn apply_schema_v1(&self, connection: &Connection) -> Result<()> {
        let schema_path = self.schema_dir.join("001-init.sql");
        if !schema_path.exists() {
            return Err(HarnessInfraError::MissingSchema(
                schema_path.display().to_string(),
            ));
        }

        let schema = fs::read_to_string(schema_path)?;
        connection.execute_batch(&schema)?;
        Ok(())
    }

    fn apply_pending_migrations_transactionally(
        &self,
        connection: &mut Connection,
        current_version: i64,
        migrations: &[SourceMigration],
    ) -> Result<Vec<i64>> {
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut applied = Vec::new();
        for migration in migrations {
            if migration.version > current_version {
                transaction.execute_batch(&fs::read_to_string(&migration.path)?)?;
                applied.push(migration.version);
            }
        }
        let final_version = applied.last().copied().unwrap_or(current_version);
        if final_version >= 6 {
            let source_commit = self.git_output(&["rev-parse", "HEAD"]).ok();
            for migration in migrations
                .iter()
                .filter(|migration| migration.version <= final_version)
            {
                let name = migration
                    .path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default();
                transaction.execute(
                    "INSERT INTO migration_history(version, name, checksum, cli_version, source_commit)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(version) DO NOTHING",
                    params![migration.version, name, migration.checksum, env!("CARGO_PKG_VERSION"), source_commit],
                )?;
            }
        }
        transaction.commit()?;
        Ok(applied)
    }

    fn backup_existing_database(
        &self,
        current_version: i64,
        lineage: &str,
        checksum: &str,
    ) -> Result<PathBuf> {
        let parent = self.db_path.parent().ok_or_else(|| {
            HarnessInfraError::BackupFailed("database path has no parent directory".to_owned())
        })?;
        let backup_dir = parent.join("harness.db.backups");
        fs::create_dir_all(&backup_dir)
            .map_err(|error| HarnessInfraError::BackupFailed(error.to_string()))?;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| HarnessInfraError::BackupFailed(error.to_string()))?
            .as_nanos();
        let short_checksum = &checksum[..checksum.len().min(12)];
        let file_name =
            format!("harness.db.{timestamp}.v{current_version}.{lineage}.{short_checksum}.bak");
        let backup_path = backup_dir.join(file_name);
        fs::copy(&self.db_path, &backup_path)
            .map_err(|error| HarnessInfraError::BackupFailed(error.to_string()))?;
        for suffix in ["-wal", "-shm"] {
            let sidecar = PathBuf::from(format!("{}{}", self.db_path.display(), suffix));
            if sidecar.exists() {
                let backup_sidecar = PathBuf::from(format!("{}{}", backup_path.display(), suffix));
                fs::copy(sidecar, backup_sidecar)
                    .map_err(|error| HarnessInfraError::BackupFailed(error.to_string()))?;
            }
        }
        self.prune_backups(&backup_dir, 5)?;
        Ok(backup_path)
    }

    fn prune_backups(&self, backup_dir: &Path, retain: usize) -> Result<()> {
        let mut backups = fs::read_dir(backup_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().ends_with(".bak"))
            .collect::<Vec<_>>();
        backups.sort_by_key(|entry| entry.file_name());
        let remove_count = backups.len().saturating_sub(retain);
        for entry in backups.into_iter().take(remove_count) {
            let backup = entry.path();
            fs::remove_file(&backup)?;
            for suffix in ["-wal", "-shm"] {
                let sidecar = PathBuf::from(format!("{}{}", backup.display(), suffix));
                if sidecar.exists() {
                    fs::remove_file(sidecar)?;
                }
            }
        }
        Ok(())
    }

    fn source_migrations(&self) -> Result<(Vec<SourceMigration>, Vec<String>, String)> {
        let manifest_path = self.schema_dir.join("manifest.toml");
        let manifest = match parse_migration_manifest(&manifest_path) {
            Ok(manifest) => manifest,
            Err(error) => {
                return Ok((
                    Vec::new(),
                    vec![format!("MANIFEST_INVALID:{error}")],
                    String::new(),
                ))
            }
        };
        let mut findings = Vec::new();
        let mut files = Vec::new();
        for entry in fs::read_dir(&self.schema_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("sql") {
                continue;
            }
            let file_name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            let Some((prefix, suffix)) = file_name.split_once('-') else {
                findings.push(format!("MIGRATION_FILENAME_INVALID:{file_name}"));
                continue;
            };
            if prefix.len() != 3
                || !prefix.chars().all(|character| character.is_ascii_digit())
                || suffix.is_empty()
            {
                findings.push(format!("MIGRATION_FILENAME_INVALID:{file_name}"));
                continue;
            }
            let version = prefix.parse::<i64>().expect("three ASCII digits parse");
            let checksum = sha256_file(&path)?;
            files.push(SourceMigration {
                version,
                path,
                checksum,
            });
        }
        files.sort_by_key(|migration| migration.version);
        if files.is_empty()
            || files.first().map(|migration| migration.version) != Some(1)
            || files
                .windows(2)
                .any(|pair| pair[1].version != pair[0].version + 1)
        {
            findings.push("MIGRATION_INVENTORY_INVALID".to_owned());
        }
        if files
            .windows(2)
            .any(|pair| pair[0].version == pair[1].version)
        {
            findings.push("MIGRATION_DUPLICATE_VERSION".to_owned());
        }
        for source in &files {
            let relative = source
                .path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            match manifest
                .migrations
                .iter()
                .find(|entry| entry.version == source.version)
            {
                Some(entry) if entry.path == relative && entry.checksum == source.checksum => {}
                Some(_) => findings.push(format!("MANIFEST_CHECKSUM_MISMATCH:{relative}")),
                None => findings.push(format!("MANIFEST_MIGRATION_MISSING:{relative}")),
            }
        }
        for entry in &manifest.migrations {
            if !files.iter().any(|source| source.version == entry.version) {
                findings.push(format!("MANIFEST_SOURCE_MISSING:{}", entry.path));
            }
        }
        Ok((files, findings, manifest.lineage))
    }

    #[allow(clippy::too_many_arguments)]
    fn doctor_report(
        &self,
        ok: bool,
        code: &str,
        message: &str,
        source_versions: Vec<i64>,
        db_versions: Vec<i64>,
        findings: Vec<String>,
        remediation: Vec<String>,
    ) -> DoctorReport {
        let (repository_id, worktree, branch, commit, provenance_findings) =
            self.repository_provenance();
        let mut findings = findings;
        findings.extend(provenance_findings);
        DoctorReport {
            ok,
            code: code.to_owned(),
            message: message.to_owned(),
            platform: format!("{}/{}", env::consts::OS, env::consts::ARCH),
            repository_id,
            worktree,
            branch,
            commit,
            source_versions,
            db_versions,
            findings,
            remediation,
        }
    }

    fn git_output(&self, args: &[&str]) -> std::result::Result<String, String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.repo_root)
            .args(args)
            .output()
            .map_err(|error| error.to_string())?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
        }
    }

    fn repository_provenance(&self) -> RepositoryProvenance {
        let repository_id = fs::read_to_string(self.repo_root.join(".harness-id"))
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());
        let mut findings = Vec::new();
        let worktree = match self.git_output(&["rev-parse", "--show-toplevel"]) {
            Ok(value) => Some(value),
            Err(_) => {
                findings.push("GIT_WORKTREE_UNAVAILABLE".to_owned());
                None
            }
        };
        if let Some(worktree) = &worktree {
            let expected = self.repo_root.canonicalize().ok();
            if expected.as_deref() != Some(Path::new(worktree)) {
                findings.push("REPOSITORY_ROOT_MISMATCH".to_owned());
            }
        }
        let branch = self.git_output(&["rev-parse", "--abbrev-ref", "HEAD"]).ok();
        if branch.is_none() {
            findings.push("GIT_BRANCH_UNAVAILABLE".to_owned());
        }
        let commit = self.git_output(&["rev-parse", "HEAD"]).ok();
        if commit.is_none() {
            findings.push("GIT_COMMIT_UNAVAILABLE".to_owned());
        }
        (repository_id, worktree, branch, commit, findings)
    }

    fn payload_findings(&self) -> Vec<String> {
        let mut findings = Vec::new();
        if !self.repo_root.is_absolute() {
            findings.push("REPOSITORY_ROOT_NOT_ABSOLUTE".to_owned());
        }
        if !self.repo_root.join(".git").exists() {
            findings.push("REPOSITORY_ROOT_INVALID".to_owned());
        }
        for path in ["AGENTS.md", ".harness-id", "_harness/bin/harness-cli"] {
            if !self.repo_root.join(path).is_file() {
                findings.push(format!("REQUIRED_PATH_MISSING:{path}"));
            }
        }
        match validate_workflow_policy(&self.repo_root.join("_harness/workflow.toml")) {
            Ok(()) => {}
            Err(error) => findings.push(format!("WORKFLOW_INVALID:{error}")),
        }
        match fs::read_to_string(self.repo_root.join(".gitignore")) {
            Ok(ignore) => {
                for required in ["harness.db", "harness.db-wal", "harness.db-shm"] {
                    if !ignore.lines().any(|line| line.trim() == required) {
                        findings.push(format!("MANAGED_IGNORE_MISSING:{required}"));
                    }
                }
            }
            Err(_) => findings.push("REQUIRED_PATH_MISSING:.gitignore".to_owned()),
        }
        findings
    }

    fn load_workflow_policy(&self) -> Result<WorkflowPolicy> {
        parse_workflow_policy(&self.repo_root.join("_harness/workflow.toml"))
            .map_err(HarnessInfraError::WorkflowInvalid)
    }

    fn import_matrix(&self, connection: &Connection) -> Result<usize> {
        let (matrix_path, matrix_record_path) = self.matrix_import_path().ok_or_else(|| {
            HarnessInfraError::MissingBrownfieldPath(
                self.repo_root
                    .join("_harness/TEST_MATRIX.md")
                    .display()
                    .to_string(),
            )
        })?;

        let content = fs::read_to_string(matrix_path)?;
        let mut story_count = 0;
        let mut columns: Option<MatrixColumns> = None;

        for line in content.lines() {
            if !line.trim_start().starts_with('|') {
                continue;
            }

            let fields = markdown_table_fields(line);
            if fields.len() < 2 {
                continue;
            }

            if columns.is_none() {
                let candidate = MatrixColumns::from_header(&fields);
                if candidate.story.is_some() && candidate.status.is_some() {
                    columns = Some(candidate);
                }
                continue;
            }

            let columns = columns.as_ref().expect("matrix columns discovered");
            let id = field_at(&fields, columns.story).unwrap_or_default();
            let token = normalize_token(&id);
            if matches!(
                token.as_str(),
                "" | "story" | "tbd" | "todo" | "example" | "examples"
            ) || id.chars().all(|character| character == '-')
            {
                continue;
            }

            let mut title = field_at(&fields, columns.contract).unwrap_or_else(|| id.clone());
            if title.is_empty() {
                title = id.clone();
            }

            let status =
                normalize_story_status(&field_at(&fields, columns.status).unwrap_or_default());
            let unit = proof_from_cell(&field_at(&fields, columns.unit).unwrap_or_default());
            let integration =
                proof_from_cell(&field_at(&fields, columns.integration).unwrap_or_default());
            let e2e = proof_from_cell(&field_at(&fields, columns.e2e).unwrap_or_default());
            let platform =
                proof_from_cell(&field_at(&fields, columns.platform).unwrap_or_default());
            let evidence = columns
                .evidence
                .and_then(|index| evidence_from_fields(&fields, index));

            connection.execute(
                "INSERT INTO story (
                    id, title, risk_lane, contract_doc, status,
                    unit_proof, integration_proof, e2e_proof, platform_proof,
                    evidence, notes
                 ) VALUES (?1, ?2, 'high_risk', ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                    ?10
                 )
                 ON CONFLICT(id) DO UPDATE SET
                    title=excluded.title,
                    contract_doc=excluded.contract_doc,
                    status=excluded.status,
                    unit_proof=excluded.unit_proof,
                    integration_proof=excluded.integration_proof,
                    e2e_proof=excluded.e2e_proof,
                    platform_proof=excluded.platform_proof,
                    evidence=excluded.evidence,
                    notes=excluded.notes;",
                params![
                    id,
                    title,
                    field_at(&fields, columns.contract),
                    status,
                    unit,
                    integration,
                    e2e,
                    platform,
                    evidence,
                    format!("Imported from {matrix_record_path} by harness import brownfield."),
                ],
            )?;
            story_count += 1;
        }

        Ok(story_count)
    }

    fn matrix_import_path(&self) -> Option<(PathBuf, &'static str)> {
        let current = self.repo_root.join("_harness/TEST_MATRIX.md");
        if current.exists() {
            return Some((current, "_harness/TEST_MATRIX.md"));
        }

        let legacy = self.repo_root.join("docs/TEST_MATRIX.md");
        if legacy.exists() {
            return Some((legacy, "docs/TEST_MATRIX.md"));
        }

        None
    }

    fn import_decisions(&self, connection: &Connection) -> Result<usize> {
        let decisions_dir = self.repo_root.join("docs/decisions");
        if !decisions_dir.is_dir() {
            return Err(HarnessInfraError::MissingBrownfieldPath(
                decisions_dir.display().to_string(),
            ));
        }

        let mut files = Vec::new();
        for entry in fs::read_dir(&decisions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("md") {
                continue;
            }
            let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if is_decision_file_name(file_name) {
                files.push(path);
            }
        }
        files.sort();

        let mut decision_count = 0;
        for path in files {
            let content = fs::read_to_string(&path)?;
            let stem = path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_owned();
            let title = content
                .lines()
                .next()
                .and_then(|line| line.strip_prefix("# "))
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or(&stem)
                .to_owned();
            let status =
                normalize_decision_status(&markdown_section_first_value(&content, "Status"));
            let doc_path = format!(
                "docs/decisions/{}",
                path.file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default()
            );

            connection.execute(
                "INSERT INTO decision (id, title, status, doc_path, notes)
                 VALUES (?1, ?2, ?3, ?4,
                    'Imported from docs/decisions by harness import brownfield.'
                 )
                 ON CONFLICT(id) DO UPDATE SET
                    title=excluded.title,
                    status=excluded.status,
                    doc_path=excluded.doc_path,
                    notes=excluded.notes;",
                params![stem, title, status, doc_path],
            )?;
            decision_count += 1;
        }

        Ok(decision_count)
    }

    fn import_backlog(&self, connection: &Connection) -> Result<usize> {
        let backlog_path = self.repo_root.join("docs/HARNESS_BACKLOG.md");
        if !backlog_path.exists() {
            return Ok(0);
        }

        let content = fs::read_to_string(backlog_path)?;
        let items = backlog_items(&content);
        let mut imported = 0;
        for item in items {
            if item.title.is_empty() || item.title == "Short name." {
                continue;
            }

            let risk = if item.risk.is_empty() {
                None
            } else {
                RiskLane::from_str(&item.risk)
                    .ok()
                    .map(|value| value.as_db_value().to_owned())
            };
            let status = normalize_backlog_status(&item.status);
            let discovered = empty_to_none(item.discovered_while);
            let pain = empty_to_none(item.current_pain);
            let suggestion = empty_to_none(item.suggested_improvement);

            connection.execute(
                "INSERT INTO backlog (
                    title, discovered_while, current_pain, suggested_improvement,
                    risk, status, notes
                 )
                 SELECT ?1, ?2, ?3, ?4, ?5, ?6,
                    'Imported from docs/HARNESS_BACKLOG.md by harness import brownfield.'
                 WHERE NOT EXISTS (
                    SELECT 1 FROM backlog WHERE title=?1
                 );",
                params![item.title, discovered, pain, suggestion, risk, status],
            )?;
            imported += 1;
        }

        Ok(imported)
    }
}

impl HarnessRepository for SqliteHarnessRepository {
    fn workflow_policy(&self) -> Result<WorkflowPolicy> {
        self.load_workflow_policy()
    }

    fn doctor(&self) -> Result<DoctorReport> {
        let (migrations, mut findings, expected_lineage) = self.source_migrations()?;
        let source_versions = migrations
            .iter()
            .map(|migration| migration.version)
            .collect::<Vec<_>>();
        if !findings.is_empty() {
            return Ok(self.doctor_report(
                false,
                "SOURCE_MIGRATION_INVALID",
                "Source migration inventory or manifest is invalid.",
                source_versions,
                Vec::new(),
                findings,
                vec!["Repair _harness/scripts/schema/manifest.toml and source migration files before operating on the database.".to_owned()],
            ));
        }
        findings.extend(self.payload_findings());
        if !self.db_path.exists() {
            return Ok(self.doctor_report(
                true,
                "DB_MISSING",
                "No local operational database exists; doctor did not create one.",
                source_versions,
                Vec::new(),
                findings,
                vec!["Run harness-cli init (compatibility command) or task start to create and ensure the local database.".to_owned()],
            ));
        }
        let connection = match Connection::open_with_flags(&self.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
            Ok(connection) => connection,
            Err(error) => return Ok(self.doctor_report(
                false, "DB_UNREADABLE", "The database cannot be opened read-only.", source_versions,
                Vec::new(), vec![format!("SQLITE_OPEN:{error}")],
                vec!["Preserve the database and restore or rebuild from a verified backup; do not reset it in place.".to_owned()],
            )),
        };
        let integrity: String = match connection.query_row("PRAGMA integrity_check", [], |row| row.get(0)) {
            Ok(value) => value,
            Err(error) => return Ok(self.doctor_report(
                false, "DB_UNREADABLE", "SQLite could not inspect database integrity.", source_versions,
                Vec::new(), vec![format!("SQLITE_INTEGRITY_ERROR:{error}")],
                vec!["Preserve the database and restore or rebuild from a verified backup; do not reset it in place.".to_owned()],
            )),
        };
        if integrity != "ok" {
            findings.push(format!("SQLITE_INTEGRITY:{integrity}"));
        }
        let fk_error = connection.prepare("PRAGMA foreign_key_check")?.exists([])?;
        if fk_error {
            findings.push("FOREIGN_KEY_VIOLATION".to_owned());
        }
        let has_schema_version: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='schema_version')",
            [], |row| row.get(0),
        )?;
        if !has_schema_version {
            return Ok(self.doctor_report(
                false, "DB_UNVERSIONED", "The database has no schema lineage table.", source_versions,
                Vec::new(), findings, vec!["Preserve this database and use the reviewed recovery/rebuild procedure; do not run migrations blindly.".to_owned()],
            ));
        }
        let db_versions = connection
            .prepare("SELECT version FROM schema_version ORDER BY version")?
            .query_map([], |row| row.get::<_, i64>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if db_versions.is_empty()
            || db_versions.first() != Some(&1)
            || db_versions.windows(2).any(|pair| pair[1] != pair[0] + 1)
        {
            findings.push("DB_MIGRATION_HISTORY_INVALID".to_owned());
        }
        let has_meta: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='harness_meta')",
            [],
            |row| row.get(0),
        )?;
        if has_meta {
            let lineage = connection
                .query_row(
                    "SELECT value FROM harness_meta WHERE key='schema_lineage'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            if lineage.as_deref() != Some(expected_lineage.as_str()) {
                findings.push("SCHEMA_LINEAGE_MISMATCH".to_owned());
            }
        } else if db_versions.last() == source_versions.last() {
            findings.push("SCHEMA_LINEAGE_UNRECORDED".to_owned());
        }
        let has_history: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='migration_history')",
            [], |row| row.get(0),
        )?;
        if has_history {
            let history = connection
                .prepare("SELECT version, checksum FROM migration_history")?
                .query_map([], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            for migration in &migrations {
                if db_versions.contains(&migration.version) {
                    match history
                        .iter()
                        .find(|(version, _)| *version == migration.version)
                    {
                        Some((_, checksum)) if checksum == &migration.checksum => {}
                        Some(_) => findings
                            .push(format!("MIGRATION_CHECKSUM_MISMATCH:{}", migration.version)),
                        None => findings
                            .push(format!("MIGRATION_HISTORY_MISSING:{}", migration.version)),
                    }
                }
            }
        } else if db_versions.last() == source_versions.last() {
            findings.push("MIGRATION_HISTORY_UNRECORDED".to_owned());
        }
        let code = if db_versions.last() > source_versions.last() {
            "DB_AHEAD_OF_SOURCE"
        } else if db_versions.last() < source_versions.last() {
            "DB_BEHIND_SOURCE"
        } else if findings.is_empty() {
            "HEALTHY"
        } else {
            "DB_UNHEALTHY"
        };
        let remediation = match code {
            "DB_AHEAD_OF_SOURCE" => vec!["Do not downgrade or overwrite the database. Preserve it and use the reviewed rebuild/recovery path for its lineage.".to_owned()],
            "DB_BEHIND_SOURCE" => vec!["Run the safe ensure/migrate path after a verified backup is available.".to_owned()],
            "HEALTHY" => Vec::new(),
            _ => vec!["Resolve the reported health findings before running a state-changing lifecycle command.".to_owned()],
        };
        Ok(self.doctor_report(
            code == "HEALTHY",
            code,
            "Database and source migration health report.",
            source_versions,
            db_versions,
            findings,
            remediation,
        ))
    }
    fn ensure(&self) -> Result<MigrateResult> {
        let report = self.doctor()?;
        let (migrations, source_findings, lineage) = self.source_migrations()?;
        if !source_findings.is_empty() {
            return Err(HarnessInfraError::UnsafeDurableState(
                "source migration inventory is invalid".to_owned(),
            ));
        }
        let latest = migrations
            .last()
            .map(|migration| migration.version)
            .unwrap_or(0);
        if report.code == "HEALTHY" {
            return Ok(MigrateResult {
                current_version: latest,
                applied: Vec::new(),
            });
        }
        if report.code == "DB_MISSING" {
            let parent = self.db_path.parent().ok_or_else(|| {
                HarnessInfraError::UnsafeDurableState(
                    "database path has no parent directory".to_owned(),
                )
            })?;
            fs::create_dir_all(parent)?;
            let temporary_path = parent.join(format!(
                ".{}.create-{}",
                self.db_path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("harness.db"),
                std::process::id()
            ));
            if temporary_path.exists() {
                fs::remove_file(&temporary_path)?;
            }
            let result = (|| -> Result<Vec<i64>> {
                let mut connection = Connection::open(&temporary_path)?;
                connection.pragma_update(None, "foreign_keys", "ON")?;
                self.apply_schema_v1(&connection)?;
                self.apply_pending_migrations_transactionally(&mut connection, 1, &migrations)
            })();
            match result {
                Ok(applied) => {
                    fs::rename(&temporary_path, &self.db_path)?;
                    Ok(MigrateResult {
                        current_version: 0,
                        applied: std::iter::once(1).chain(applied).collect(),
                    })
                }
                Err(error) => {
                    let _ = fs::remove_file(&temporary_path);
                    Err(error)
                }
            }
        } else if report.code == "DB_BEHIND_SOURCE" {
            let current_version = report.db_versions.last().copied().unwrap_or(0);
            let checksum = migrations
                .last()
                .map(|migration| migration.checksum.as_str())
                .unwrap_or("unknown");
            self.backup_existing_database(current_version, &lineage, checksum)?;
            let mut connection = self.open_existing()?;
            let applied = self.apply_pending_migrations_transactionally(
                &mut connection,
                current_version,
                &migrations,
            )?;
            let after = self.doctor()?;
            if after.code != "HEALTHY" {
                return Err(HarnessInfraError::UnsafeDurableState(format!(
                    "post-migration doctor returned {}",
                    after.code
                )));
            }
            Ok(MigrateResult {
                current_version,
                applied,
            })
        } else {
            Err(HarnessInfraError::UnsafeDurableState(report.code))
        }
    }

    fn init(&self) -> Result<InitResult> {
        let existed = self.db_path.exists();
        let result = self.ensure()?;
        if !existed {
            Ok(InitResult::Created {
                db_path: self.db_path.clone(),
            })
        } else if result.applied.is_empty() {
            Ok(InitResult::Existing {
                db_path: self.db_path.clone(),
                version: result.current_version,
            })
        } else {
            Ok(InitResult::MigratedExisting {
                db_path: self.db_path.clone(),
            })
        }
    }

    fn migrate(&self) -> Result<MigrateResult> {
        self.ensure()
    }

    fn import_brownfield(&self) -> Result<BrownfieldImportResult> {
        let connection = self.open_existing()?;
        let stories = self.import_matrix(&connection)?;
        let decisions = self.import_decisions(&connection)?;
        let backlog_items = self.import_backlog(&connection)?;

        Ok(BrownfieldImportResult {
            stories,
            decisions,
            backlog_items,
        })
    }

    fn record_intake(&self, input: IntakeInput) -> Result<i64> {
        let connection = self.open_existing()?;
        connection.execute(
            "INSERT INTO intake (
                input_type, summary, risk_lane, risk_flags, affected_docs, story_id, notes
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);",
            params![
                input.input_type.as_db_value(),
                input.summary,
                input.risk_lane.as_db_value(),
                input.risk_flags.as_json_text(),
                input.affected_docs.as_json_text(),
                input.story_id,
                input.notes,
            ],
        )?;

        Ok(connection.last_insert_rowid())
    }

    fn add_story(&self, input: StoryAddInput) -> Result<()> {
        let connection = self.open_existing()?;
        connection.execute(
            "INSERT INTO story (id, title, risk_lane, contract_doc, verify_command, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
            params![
                input.id,
                input.title,
                input.risk_lane.as_db_value(),
                input.contract_doc,
                input.verify_command,
                input.notes,
            ],
        )?;
        Ok(())
    }

    fn update_story(&self, input: StoryUpdateInput) -> Result<()> {
        if input.status.is_none()
            && input.evidence.is_none()
            && input.unit.is_none()
            && input.integration.is_none()
            && input.e2e.is_none()
            && input.platform.is_none()
            && input.verify_command.is_none()
        {
            return Err(HarnessInfraError::EmptyStoryUpdate);
        }

        let connection = self.open_existing()?;
        connection.execute(
            "UPDATE story SET
                status=COALESCE(?1, status),
                evidence=COALESCE(?2, evidence),
                unit_proof=COALESCE(?3, unit_proof),
                integration_proof=COALESCE(?4, integration_proof),
                e2e_proof=COALESCE(?5, e2e_proof),
                platform_proof=COALESCE(?6, platform_proof),
                verify_command=COALESCE(?7, verify_command)
             WHERE id=?8;",
            params![
                input.status,
                input.evidence,
                input.unit.map(|value| value.0),
                input.integration.map(|value| value.0),
                input.e2e.map(|value| value.0),
                input.platform.map(|value| value.0),
                input.verify_command,
                input.id,
            ],
        )?;

        if connection.changes() == 0 {
            return Err(HarnessInfraError::StoryNotFound(input.id));
        }
        Ok(())
    }

    fn verify_story(&self, id: &str) -> Result<StoryVerifyResult> {
        let connection = self.open_existing()?;
        let verify_command = connection
            .query_row(
                "SELECT verify_command FROM story WHERE id=?1;",
                params![id],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()?
            .flatten()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| HarnessInfraError::MissingStoryVerifyCommand(id.to_owned()))?;

        let (shell, flag) = verifier_shell();
        let output = Command::new(shell)
            .arg(flag)
            .arg(&verify_command)
            .current_dir(&self.repo_root)
            .output()?;
        let result = if output.status.success() {
            "pass"
        } else {
            "fail"
        }
        .to_owned();
        connection.execute(
            "UPDATE story
             SET last_verified_at=datetime('now'), last_verified_result=?1
             WHERE id=?2;",
            params![result, id],
        )?;

        Ok(StoryVerifyResult {
            command: verify_command,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            result,
        })
    }

    fn verify_all_stories(&self) -> Result<StoryVerifyAllResult> {
        let connection = self.open_existing()?;
        let mut statement =
            connection.prepare("SELECT id, title, verify_command FROM story ORDER BY id;")?;
        let story_rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;
        let stories = collect_rows(story_rows)?;
        let mut items = Vec::new();

        for (id, title, verify_command) in stories {
            let Some(command) = verify_command.filter(|value| !value.trim().is_empty()) else {
                items.push(StoryVerifyAllItem {
                    id,
                    title,
                    command: None,
                    result: "skipped".to_owned(),
                    stdout: String::new(),
                    stderr: String::new(),
                });
                continue;
            };

            let (shell, flag) = verifier_shell();
            let output = Command::new(shell)
                .arg(flag)
                .arg(&command)
                .current_dir(&self.repo_root)
                .output()?;
            let result = if output.status.success() {
                "pass"
            } else {
                "fail"
            }
            .to_owned();
            connection.execute(
                "UPDATE story
                 SET last_verified_at=datetime('now'), last_verified_result=?1
                 WHERE id=?2;",
                params![result, id],
            )?;
            items.push(StoryVerifyAllItem {
                id,
                title,
                command: Some(command),
                result,
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }

        Ok(StoryVerifyAllResult { items })
    }

    fn add_decision(&self, input: DecisionAddInput) -> Result<()> {
        let connection = self.open_existing()?;
        connection.execute(
            "INSERT INTO decision (id, title, status, doc_path, verify_command, predicted_impact, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);",
            params![
                input.id,
                input.title,
                input.status,
                input.doc_path,
                input.verify_command,
                input.predicted_impact,
                input.notes,
            ],
        )?;
        Ok(())
    }

    fn verify_decision(&self, id: &str) -> Result<DecisionVerifyResult> {
        let connection = self.open_existing()?;
        let verify_command = connection
            .query_row(
                "SELECT verify_command FROM decision WHERE id=?1;",
                params![id],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()?
            .flatten()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| HarnessInfraError::MissingDecisionVerifyCommand(id.to_owned()))?;

        let (shell, flag) = verifier_shell();
        let status = Command::new(shell)
            .arg(flag)
            .arg(&verify_command)
            .current_dir(&self.repo_root)
            .status()?;
        let result = if status.success() { "pass" } else { "fail" }.to_owned();
        connection.execute(
            "UPDATE decision
             SET last_verified_at=datetime('now'), last_verified_result=?1
             WHERE id=?2;",
            params![result, id],
        )?;

        Ok(DecisionVerifyResult {
            command: verify_command,
            result,
        })
    }

    fn add_backlog(&self, input: BacklogAddInput) -> Result<i64> {
        let connection = self.open_existing()?;
        connection.execute(
            "INSERT INTO backlog (
                title, discovered_while, current_pain, suggested_improvement,
                risk, predicted_impact, notes
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);",
            params![
                input.title,
                input.discovered_while,
                input.current_pain,
                input.suggestion,
                input.risk.map(|value| value.as_db_value().to_owned()),
                input.predicted_impact,
                input.notes,
            ],
        )?;
        Ok(connection.last_insert_rowid())
    }

    fn close_backlog(&self, input: BacklogCloseInput) -> Result<()> {
        let connection = self.open_existing()?;
        connection.execute(
            "UPDATE backlog
             SET status=?1, actual_outcome=?2, implemented_at=datetime('now')
             WHERE id=?3;",
            params![input.status, input.actual_outcome, input.id],
        )?;

        if connection.changes() == 0 {
            return Err(HarnessInfraError::BacklogNotFound(input.id));
        }
        Ok(())
    }

    fn register_tool(&self, input: ToolRegisterInput) -> Result<()> {
        validate_tool_description(&input.description)?;
        // Only exec-probed kinds are PATH-checked at register time. mcp/skill/http
        // are not on PATH by nature, so registering intent always succeeds; their
        // presence is resolved later by `tool check` via scan_target.
        let exec_probed = matches!(input.kind.as_str(), "cli" | "binary");
        if exec_probed && !input.force && !command_available(&self.repo_root, &input.command) {
            return Err(HarnessInfraError::ToolCommandNotFound(input.command));
        }

        let connection = self.open_existing()?;
        let existing = connection
            .query_row(
                "SELECT command FROM tool WHERE name=?1;",
                params![input.name],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        if let Some(command) = existing {
            return Err(HarnessInfraError::ToolAlreadyExists(input.name, command));
        }

        connection.execute(
            "INSERT INTO tool
                (name, provider, command, description, args, responsibility, since,
                 kind, capability, scan_target, status)
             VALUES (?1, 'custom', ?2, ?3, ?4, ?5, 'registered', ?6, ?7, ?8, 'unknown');",
            params![
                input.name,
                input.command,
                input.description,
                tool_args_json(&input.args),
                input.responsibility,
                input.kind,
                input.capability,
                input.scan_target,
            ],
        )?;
        Ok(())
    }

    fn remove_tool(&self, name: &str) -> Result<()> {
        let connection = self.open_existing()?;
        connection.execute("DELETE FROM tool WHERE name=?1;", params![name])?;
        if connection.changes() == 0 {
            return Err(HarnessInfraError::ToolNotFound(name.to_owned()));
        }
        Ok(())
    }

    fn check_tools(&self, name: Option<String>) -> Result<Vec<ToolCheckResult>> {
        let connection = self.open_existing()?;
        let mut statement = connection.prepare(
            "SELECT name, kind, command, scan_target, capability FROM tool
             WHERE (?1 IS NULL OR name = ?1)
             ORDER BY name;",
        )?;
        let rows = statement.query_map(params![name], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })?;
        let tools = collect_rows(rows)?;

        let mut results = Vec::with_capacity(tools.len());
        for (name, kind, command, scan_target, capability) in tools {
            let (status, detail) =
                scan_tool_status(&self.repo_root, &kind, &command, scan_target.as_deref());
            connection.execute(
                "UPDATE tool SET status=?1, checked_at=datetime('now') WHERE name=?2;",
                params![status, name],
            )?;
            results.push(ToolCheckResult {
                name,
                kind,
                capability,
                status: status.to_owned(),
                detail,
            });
        }
        Ok(results)
    }

    fn add_intervention(&self, input: InterventionAddInput) -> Result<i64> {
        let connection = self.open_existing()?;
        connection.execute(
            "INSERT INTO intervention (trace_id, story_id, type, description, source, impact)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
            params![
                input.trace_id,
                input.story_id,
                input.intervention_type,
                input.description,
                input.source,
                input.impact,
            ],
        )?;
        Ok(connection.last_insert_rowid())
    }

    fn record_trace(&self, input: TraceInput) -> Result<i64> {
        let connection = self.open_existing()?;
        connection.execute(
            "INSERT INTO trace (
                task_summary, intake_id, story_id, agent,
                actions_taken, files_read, files_changed, decisions_made, errors,
                outcome, duration_seconds, token_estimate, harness_friction, notes
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14);",
            params![
                input.task_summary,
                input.intake_id,
                input.story_id,
                input.agent,
                input.actions.as_json_text(),
                input.files_read.as_json_text(),
                input.files_changed.as_json_text(),
                input.decisions.as_json_text(),
                input.errors.as_json_text(),
                input.outcome,
                input.duration_seconds,
                input.token_estimate,
                input.friction,
                input.notes,
            ],
        )?;
        Ok(connection.last_insert_rowid())
    }

    fn score_trace(&self, id: Option<i64>) -> Result<TraceScoreResult> {
        let connection = self.open_existing()?;
        let sql = match id {
            Some(_) => {
                "SELECT
                    trace.id,
                    trace.task_summary,
                    trace.intake_id,
                    intake.risk_lane,
                    trace.agent,
                    trace.actions_taken,
                    trace.files_read,
                    trace.files_changed,
                    trace.decisions_made,
                    trace.errors,
                    trace.outcome,
                    trace.duration_seconds,
                    trace.token_estimate,
                    trace.harness_friction,
                    trace.notes
                 FROM trace
                 LEFT JOIN intake ON intake.id = trace.intake_id
                 WHERE trace.id = ?1"
            }
            None => {
                "SELECT
                    trace.id,
                    trace.task_summary,
                    trace.intake_id,
                    intake.risk_lane,
                    trace.agent,
                    trace.actions_taken,
                    trace.files_read,
                    trace.files_changed,
                    trace.decisions_made,
                    trace.errors,
                    trace.outcome,
                    trace.duration_seconds,
                    trace.token_estimate,
                    trace.harness_friction,
                    trace.notes
                 FROM trace
                 LEFT JOIN intake ON intake.id = trace.intake_id
                 ORDER BY trace.id DESC
                 LIMIT 1"
            }
        };

        let source = if let Some(id) = id {
            connection
                .query_row(sql, params![id], trace_score_source_from_row)
                .optional()?
                .ok_or(HarnessInfraError::TraceNotFound(id))?
        } else {
            connection
                .query_row(sql, [], trace_score_source_from_row)
                .optional()?
                .ok_or(HarnessInfraError::NoTraces)?
        };

        Ok(score_trace(source))
    }

    fn score_context(&self, id: i64) -> Result<ContextScoreResult> {
        let connection = self.open_existing()?;
        let source = connection
            .query_row(
                "SELECT
                    trace.id,
                    intake.risk_lane,
                    intake.risk_flags,
                    trace.story_id,
                    trace.files_read,
                    trace.files_changed,
                    trace.outcome
                 FROM trace
                 LEFT JOIN intake ON intake.id = trace.intake_id
                 WHERE trace.id=?1;",
                params![id],
                |row| {
                    Ok(ContextScoreSource {
                        id: row.get(0)?,
                        risk_lane: row.get(1)?,
                        risk_flags: row.get(2)?,
                        story_id: row.get(3)?,
                        files_read: row.get(4)?,
                        files_changed: row.get(5)?,
                        outcome: row.get(6)?,
                    })
                },
            )
            .optional()?
            .ok_or(HarnessInfraError::TraceNotFound(id))?;

        let lane = source.risk_lane.as_deref().unwrap_or("tiny");
        let phase = infer_context_phase(&source);
        let paths = jsonish_list(source.files_changed.as_deref());
        let flags = jsonish_list(source.risk_flags.as_deref());
        let linked_artifacts = source
            .story_id
            .as_ref()
            .map(|_| vec!["docs/stories/".to_owned()])
            .unwrap_or_default();
        let policy = self.load_workflow_policy()?;
        let manifest = policy.context_manifest(lane, &phase, &paths, &flags, &linked_artifacts);
        let expected_must = manifest
            .must_read
            .iter()
            .map(|entry| (entry.reason.clone(), entry.path.clone()))
            .collect::<Vec<_>>();
        let expected_should = manifest
            .should_read
            .iter()
            .map(|entry| (entry.reason.clone(), entry.path.clone()))
            .collect::<Vec<_>>();
        let expected_skip = manifest
            .skip
            .iter()
            .map(|entry| entry.path.clone())
            .collect::<Vec<_>>();
        Ok(score_context(
            source,
            &expected_must,
            &expected_should,
            &expected_skip,
        ))
    }

    fn story_verify_status(&self, id: &str) -> Result<StoryVerifyStatus> {
        let connection = self.open_existing()?;
        connection
            .query_row(
                "SELECT id, verify_command, last_verified_result FROM story WHERE id=?1;",
                params![id],
                |row| {
                    Ok(StoryVerifyStatus {
                        id: row.get(0)?,
                        verify_command: row.get(1)?,
                        last_verified_result: row.get(2)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| HarnessInfraError::StoryNotFound(id.to_owned()))
    }

    fn query_matrix(&self) -> Result<Vec<StoryMatrixRecord>> {
        let connection = self.open_existing()?;
        let mut statement = connection.prepare(
            "SELECT id, title, status, unit_proof, integration_proof, e2e_proof, platform_proof, evidence
             FROM story ORDER BY id;",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(StoryMatrixRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                status: row.get(2)?,
                unit: row.get(3)?,
                integration: row.get(4)?,
                e2e: row.get(5)?,
                platform: row.get(6)?,
                evidence: row.get(7)?,
            })
        })?;

        collect_rows(rows)
    }

    fn query_backlog(&self, filter: BacklogFilter) -> Result<Vec<BacklogRecord>> {
        let connection = self.open_existing()?;
        let where_clause = match filter {
            BacklogFilter::All => "",
            BacklogFilter::Open => "WHERE status IN ('proposed', 'accepted')",
            BacklogFilter::Closed => "WHERE status IN ('implemented', 'rejected')",
        };
        let sql = format!(
            "SELECT id, title, status, risk, predicted_impact, actual_outcome
             FROM backlog {where_clause} ORDER BY status, id;"
        );
        let mut statement = connection.prepare(&sql)?;

        let rows = statement.query_map([], |row| {
            Ok(BacklogRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                status: row.get(2)?,
                risk: row.get(3)?,
                predicted_impact: row.get(4)?,
                actual_outcome: row.get(5)?,
            })
        })?;

        collect_rows(rows)
    }

    fn query_decisions(&self) -> Result<Vec<DecisionRecord>> {
        let connection = self.open_existing()?;
        let mut statement = connection.prepare(
            "SELECT id, title, status, last_verified_at, last_verified_result
             FROM decision ORDER BY id;",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(DecisionRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                status: row.get(2)?,
                last_verified_at: row.get(3)?,
                last_verified_result: row.get(4)?,
            })
        })?;

        collect_rows(rows)
    }

    fn query_intakes(&self) -> Result<Vec<IntakeRecord>> {
        let connection = self.open_existing()?;
        let mut statement = connection.prepare(
            "SELECT id, created_at, input_type, risk_lane, summary
             FROM intake ORDER BY id DESC LIMIT 20;",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(IntakeRecord {
                id: row.get(0)?,
                created_at: row.get(1)?,
                input_type: row.get(2)?,
                risk_lane: row.get(3)?,
                summary: row.get(4)?,
            })
        })?;

        collect_rows(rows)
    }

    fn query_traces(&self) -> Result<Vec<TraceRecord>> {
        let connection = self.open_existing()?;
        let mut statement = connection.prepare(
            "SELECT id, created_at, outcome, task_summary, harness_friction
             FROM trace ORDER BY id DESC LIMIT 20;",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(TraceRecord {
                id: row.get(0)?,
                created_at: row.get(1)?,
                outcome: row.get(2)?,
                task_summary: row.get(3)?,
                harness_friction: row.get(4)?,
            })
        })?;

        collect_rows(rows)
    }

    fn query_friction(&self) -> Result<Vec<FrictionRecord>> {
        let connection = self.open_existing()?;
        let mut statement = connection.prepare(
            "SELECT
                trace.id,
                trace.created_at,
                intake.risk_lane,
                intake.input_type,
                trace.task_summary,
                trace.harness_friction
             FROM trace
             LEFT JOIN intake ON intake.id = trace.intake_id
             WHERE trace.harness_friction IS NOT NULL
               AND TRIM(
                   trace.harness_friction,
                   ' ' || CHAR(9) || CHAR(10) || CHAR(11) || CHAR(12) || CHAR(13)
               ) <> ''
               AND LOWER(TRIM(
                   trace.harness_friction,
                   ' ' || CHAR(9) || CHAR(10) || CHAR(11) || CHAR(12) || CHAR(13)
               )) <> 'none'
             ORDER BY trace.id DESC;",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(FrictionRecord {
                id: row.get(0)?,
                created_at: row.get(1)?,
                risk_lane: row.get(2)?,
                input_type: row.get(3)?,
                task_summary: row.get(4)?,
                harness_friction: row.get(5)?,
            })
        })?;

        collect_rows(rows)
    }

    fn query_tools(
        &self,
        responsibility: Option<String>,
        capability: Option<String>,
    ) -> Result<Vec<ToolEntry>> {
        let connection = self.open_existing()?;
        let mut tools = compiled_tool_registry();
        let mut statement = connection.prepare(
            "SELECT provider, name, command, description, args, responsibility, since,
                    kind, capability, scan_target, status, checked_at
             FROM tool ORDER BY name;",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(ToolEntry {
                provider: row.get(0)?,
                name: row.get(1)?,
                command: row.get(2)?,
                description: row.get(3)?,
                args: parse_stored_tool_args(row.get::<_, Option<String>>(4)?.as_deref()),
                responsibility: row.get(5)?,
                source: "registered".to_owned(),
                since: row.get(6)?,
                kind: row.get(7)?,
                capability: row.get(8)?,
                scan_target: row.get(9)?,
                status: row.get(10)?,
                checked_at: row.get(11)?,
            })
        })?;
        tools.extend(collect_rows(rows)?);
        if let Some(responsibility) = responsibility {
            let normalized = normalize_token(&responsibility);
            tools.retain(|tool| normalize_token(&tool.responsibility) == normalized);
        }
        if let Some(capability) = capability {
            let normalized = normalize_token(&capability);
            tools.retain(|tool| {
                tool.capability
                    .as_deref()
                    .is_some_and(|value| normalize_token(value) == normalized)
            });
        }
        Ok(tools)
    }

    fn query_interventions(&self, filter: InterventionFilter) -> Result<Vec<InterventionRecord>> {
        let connection = self.open_existing()?;
        let mut statement = connection.prepare(
            "SELECT id, created_at, trace_id, story_id, type, description, source, impact
             FROM intervention
             WHERE (?1 IS NULL OR trace_id = ?1)
               AND (?2 IS NULL OR story_id = ?2)
               AND (?3 IS NULL OR type = ?3)
             ORDER BY id DESC;",
        )?;
        let rows = statement.query_map(
            params![filter.trace_id, filter.story_id, filter.intervention_type],
            |row| {
                Ok(InterventionRecord {
                    id: row.get(0)?,
                    created_at: row.get(1)?,
                    trace_id: row.get(2)?,
                    story_id: row.get(3)?,
                    intervention_type: row.get(4)?,
                    description: row.get(5)?,
                    source: row.get(6)?,
                    impact: row.get(7)?,
                })
            },
        )?;
        collect_rows(rows)
    }

    fn query_stats(&self) -> Result<HarnessStats> {
        let connection = self.open_existing()?;
        connection
            .query_row(
                "SELECT
                    (SELECT COUNT(*) FROM intake) AS intakes,
                    (SELECT COUNT(*) FROM story) AS stories,
                    (SELECT COUNT(*) FROM decision) AS decisions,
                    (SELECT COUNT(*) FROM backlog) AS backlog_items,
                    (SELECT COUNT(*) FROM trace) AS traces;",
                [],
                |row| {
                    Ok(HarnessStats {
                        intakes: row.get(0)?,
                        stories: row.get(1)?,
                        decisions: row.get(2)?,
                        backlog_items: row.get(3)?,
                        traces: row.get(4)?,
                    })
                },
            )
            .map_err(HarnessInfraError::from)
    }

    fn audit(&self) -> Result<AuditResult> {
        let connection = self.open_existing()?;
        let mut result = AuditResult {
            orphaned_stories: audit_findings(
                &connection,
                "SELECT story.id, story.title
                 FROM story
                 LEFT JOIN trace ON trace.story_id = story.id
                 WHERE story.status IN ('planned','in_progress') AND trace.id IS NULL
                 ORDER BY story.id;",
            )?,
            unverified_stories: audit_findings(
                &connection,
                "SELECT id, title FROM story
                 WHERE verify_command IS NOT NULL
                   AND TRIM(verify_command) <> ''
                   AND last_verified_result IS NULL
                 ORDER BY id;",
            )?,
            unverified_decisions: audit_findings(
                &connection,
                "SELECT id, title FROM decision
                 WHERE verify_command IS NOT NULL
                   AND TRIM(verify_command) <> ''
                   AND last_verified_result IS NULL
                 ORDER BY id;",
            )?,
            backlog_without_outcomes: audit_findings(
                &connection,
                "SELECT CAST(id AS TEXT), title FROM backlog
                 WHERE predicted_impact IS NOT NULL
                   AND actual_outcome IS NULL
                   AND status='implemented'
                 ORDER BY id;",
            )?,
            stale_stories: audit_findings(
                &connection,
                "SELECT story.id, story.title
                 FROM story
                 JOIN trace ON trace.story_id = story.id
                 WHERE story.status <> 'implemented'
                 GROUP BY story.id, story.title
                 HAVING julianday('now') - julianday(MAX(trace.created_at)) > 30
                 ORDER BY story.id;",
            )?,
            broken_tools: Vec::new(),
        };

        let mut statement =
            connection.prepare("SELECT name, command, kind, status FROM tool ORDER BY name;")?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        for (name, command, kind, status) in collect_rows(rows)? {
            // Exec-probed kinds are checked live against PATH. Scanned kinds
            // (mcp/skill/http) are only "broken" once a scan has positively
            // found them missing; an un-scanned `unknown` is not drift.
            let broken = match kind.as_str() {
                "cli" | "binary" => !command_available(&self.repo_root, &command),
                _ => status == "missing",
            };
            if broken {
                result.broken_tools.push(AuditFinding {
                    id: name,
                    title: command,
                });
            }
        }
        Ok(result)
    }

    fn propose(&self, commit: bool) -> Result<Vec<ImprovementProposal>> {
        let connection = self.open_existing()?;
        let audit = self.audit()?;
        let mut proposals = Vec::new();

        for (text, count) in repeated_friction(&connection)? {
            proposals.push(ImprovementProposal {
                title: format!("Reduce repeated friction: {}", short_title(&text)),
                component: "Failure attribution".to_owned(),
                evidence: format!("{count} traces recorded similar friction: {text}"),
                predicted_impact: "Fewer repeated harness friction entries for similar tasks.".to_owned(),
                risk: "normal".to_owned(),
                suggested_action: "Update the relevant Harness docs, templates, or CLI guidance for this friction pattern.".to_owned(),
                validation_plan: "Review the next five related traces and compare friction frequency.".to_owned(),
                confidence: confidence_for_count(count),
                committed_backlog_id: None,
            });
        }

        for (key, count) in repeated_interventions(&connection)? {
            proposals.push(ImprovementProposal {
                title: format!("Address repeated intervention: {}", short_title(&key)),
                component: "Intervention recording".to_owned(),
                evidence: format!("{count} interventions share the pattern: {key}"),
                predicted_impact: "Fewer repeated human or review interventions for the same issue.".to_owned(),
                risk: "normal".to_owned(),
                suggested_action: "Clarify the relevant operating rule or validation gate that would have caught this earlier.".to_owned(),
                validation_plan: "Future interventions of this type should decrease after the rule change.".to_owned(),
                confidence: confidence_for_count(count),
                committed_backlog_id: None,
            });
        }

        for (category, count) in [
            (
                "orphaned planned or in-progress stories",
                audit.orphaned_stories.len(),
            ),
            ("unverified story commands", audit.unverified_stories.len()),
            (
                "unverified decision commands",
                audit.unverified_decisions.len(),
            ),
            (
                "implemented backlog items without outcomes",
                audit.backlog_without_outcomes.len(),
            ),
            ("stale unfinished stories", audit.stale_stories.len()),
            ("broken registered tools", audit.broken_tools.len()),
        ] {
            if count > 0 {
                proposals.push(ImprovementProposal {
                    title: format!("Clean up {category}"),
                    component: "Entropy auditing".to_owned(),
                    evidence: format!("Audit found {count} {category}."),
                    predicted_impact: "Lower entropy score and stronger completion evidence.".to_owned(),
                    risk: "tiny".to_owned(),
                    suggested_action: "Resolve the listed audit findings or record why they are intentionally retained.".to_owned(),
                    validation_plan: "Run harness-cli audit and confirm the category count decreases.".to_owned(),
                    confidence: "low".to_owned(),
                    committed_backlog_id: None,
                });
            }
        }

        if commit {
            for proposal in &mut proposals {
                connection.execute(
                    "INSERT INTO backlog (
                        title, discovered_while, current_pain, suggested_improvement,
                        risk, predicted_impact, notes
                     ) VALUES (?1, 'harness-cli propose', ?2, ?3, ?4, ?5, ?6);",
                    params![
                        proposal.title,
                        proposal.evidence,
                        proposal.suggested_action,
                        normalize_token(&proposal.risk),
                        proposal.predicted_impact,
                        format!(
                            "component: {}; confidence: {}; validation: {}",
                            proposal.component, proposal.confidence, proposal.validation_plan
                        ),
                    ],
                )?;
                proposal.committed_backlog_id = Some(connection.last_insert_rowid());
            }
        }

        Ok(proposals)
    }

    fn query_sql(&self, sql: &str) -> Result<QueryTable> {
        let normalized = sql.trim();
        let statement = normalized.trim_end_matches(';').trim();
        if statement.is_empty()
            || normalized.trim_end_matches(';').contains(';')
            || !is_read_only_sql(statement)
        {
            return Err(HarnessInfraError::UnsafeSql);
        }
        let connection =
            Connection::open_with_flags(&self.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let mut prepared = connection.prepare(statement)?;
        if !prepared.readonly() {
            return Err(HarnessInfraError::UnsafeSql);
        }
        let headers = prepared
            .column_names()
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>();
        let column_count = prepared.column_count();
        let rows = prepared.query_map([], |row| {
            let mut values = Vec::new();
            for index in 0..column_count {
                values.push(sql_value_to_string(row.get_ref(index)?));
            }
            Ok(values)
        })?;

        Ok(QueryTable {
            headers,
            rows: collect_rows(rows)?,
        })
    }
}

fn is_read_only_sql(statement: &str) -> bool {
    let keyword = statement
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    match keyword.as_str() {
        "select" | "with" => true,
        "pragma" => {
            let value = statement[6..].trim().to_ascii_lowercase();
            matches!(value.as_str(), "integrity_check" | "foreign_key_check")
                || value.starts_with("table_info(")
        }
        _ => false,
    }
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut hash = Sha256::new();
    hash.update(fs::read(path)?);
    Ok(format!("{:x}", hash.finalize()))
}

fn parse_migration_manifest(path: &Path) -> std::result::Result<MigrationManifest, String> {
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let mut lineage = None;
    let mut migrations = Vec::new();
    let mut current: Option<ManifestMigration> = None;
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty()
            || line.starts_with('#')
            || line == "checksum = \"sha256-utf8-lf-no-bom\""
        {
            continue;
        }
        if line == "[[migration]]" {
            if let Some(entry) = current.take() {
                migrations.push(entry);
            }
            current = Some(ManifestMigration {
                version: 0,
                path: String::new(),
                checksum: String::new(),
            });
            continue;
        }
        let Some((key, raw_value)) = line.split_once('=') else {
            return Err(format!("invalid line '{line}'"));
        };
        let value = raw_value.trim().trim_matches('"');
        match (current.as_mut(), key.trim()) {
            (None, "lineage") => lineage = Some(value.to_owned()),
            (Some(entry), "version") => {
                entry.version = value
                    .parse()
                    .map_err(|_| format!("invalid migration version '{value}'"))?
            }
            (Some(entry), "path") => entry.path = value.to_owned(),
            (Some(entry), "sha256") => entry.checksum = value.to_owned(),
            _ => return Err(format!("unsupported manifest key '{}'", key.trim())),
        }
    }
    if let Some(entry) = current {
        migrations.push(entry);
    }
    let lineage = lineage
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "missing lineage".to_owned())?;
    if migrations.iter().any(|entry| {
        entry.version < 1
            || entry.path.is_empty()
            || entry.checksum.len() != 64
            || !entry
                .checksum
                .chars()
                .all(|character| character.is_ascii_hexdigit())
    }) {
        return Err("migration entry is incomplete or has an invalid checksum".to_owned());
    }
    if migrations
        .windows(2)
        .any(|pair| pair[0].version >= pair[1].version)
    {
        return Err("migration entries must be strictly ordered by version".to_owned());
    }
    Ok(MigrationManifest {
        lineage,
        migrations,
    })
}

fn validate_workflow_policy(path: &Path) -> std::result::Result<(), String> {
    parse_workflow_policy(path).map(|_| ())
}

fn parse_workflow_policy(path: &Path) -> std::result::Result<WorkflowPolicy, String> {
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let policy: WorkflowPolicy = toml::from_str(&content).map_err(|error| error.to_string())?;
    let major = policy.policy_version.split('.').next().unwrap_or_default();
    if major != "1" {
        return Err(format!(
            "unsupported policy major version '{}'",
            policy.policy_version
        ));
    }
    if policy.policy_id.trim().is_empty() {
        return Err("policy_id must not be empty".to_owned());
    }
    if !matches!(policy.mode.as_str(), "shadow" | "authority") {
        return Err(format!("invalid policy mode '{}'", policy.mode));
    }
    for path in [
        &policy.repository.product_docs,
        &policy.repository.stories,
        &policy.repository.decisions,
        &policy.repository.tasks,
    ] {
        if path.is_empty()
            || Path::new(path).is_absolute()
            || path.split('/').any(|part| part == "..")
        {
            return Err(format!(
                "repository path '{path}' must be repo-relative without traversal"
            ));
        }
    }
    if policy.classification.normal_min_flags == 0
        || policy.classification.high_risk_min_flags < policy.classification.normal_min_flags
    {
        return Err("classification thresholds must be positive and high_risk_min_flags >= normal_min_flags".to_owned());
    }
    let hard_gates = policy
        .classification
        .hard_gates
        .iter()
        .map(|gate| normalize_token(gate))
        .collect::<HashSet<_>>();
    if hard_gates.len() != policy.classification.hard_gates.len()
        || hard_gates.iter().any(String::is_empty)
    {
        return Err("hard gates must be non-empty and unique after normalization".to_owned());
    }
    for (alias, target) in &policy.classification.aliases {
        if normalize_token(alias).is_empty() || normalize_token(target).is_empty() {
            return Err("classification aliases must map non-empty canonical tokens".to_owned());
        }
    }
    for lane in [
        &policy.lanes.tiny,
        &policy.lanes.normal,
        &policy.lanes.high_risk,
    ] {
        if !matches!(
            lane.trace_tier.as_str(),
            "minimal" | "standard" | "detailed"
        ) {
            return Err(format!("invalid trace_tier '{}'", lane.trace_tier));
        }
        if !matches!(lane.story.as_str(), "required" | "when_behavior_bearing") {
            return Err(format!("invalid story policy '{}'", lane.story));
        }
        if !matches!(lane.capsule.as_str(), "required" | "when_material") {
            return Err(format!("invalid capsule policy '{}'", lane.capsule));
        }
        if lane.proof.is_empty() {
            return Err("lane proof must not be empty".to_owned());
        }
    }
    if policy.context.stop_condition.trim().is_empty()
        || policy.context.token_budget.tiny == 0
        || policy.context.token_budget.normal < policy.context.token_budget.tiny
        || policy.context.token_budget.high_risk < policy.context.token_budget.normal
    {
        return Err(
            "context stop condition and increasing positive token budgets are required".to_owned(),
        );
    }
    let mut rule_ids = HashSet::new();
    for rule in &policy.context.rules {
        if rule.id.trim().is_empty() || !rule_ids.insert(rule.id.clone()) {
            return Err("context rule ids must be non-empty and unique".to_owned());
        }
        if !rule.always && rule.when_paths.is_empty() && rule.when_flags.is_empty() {
            return Err(format!("context rule '{}' has no trigger", rule.id));
        }
        if rule.must_read.is_empty() && rule.should_read.is_empty() && rule.skip.is_empty() {
            return Err(format!("context rule '{}' has no output", rule.id));
        }
        if rule.phases.iter().any(|phase| {
            !matches!(
                normalize_token(phase).as_str(),
                "intake" | "planning" | "work" | "finish"
            )
        }) {
            return Err(format!("context rule '{}' has an invalid phase", rule.id));
        }
        if rule.lanes.iter().any(|lane| {
            !matches!(
                normalize_token(lane).as_str(),
                "tiny" | "normal" | "high_risk"
            )
        }) {
            return Err(format!("context rule '{}' has an invalid lane", rule.id));
        }
        for pattern in &rule.when_paths {
            if pattern.is_empty()
                || Path::new(pattern).is_absolute()
                || pattern.split('/').any(|part| part == "..")
            {
                return Err(format!(
                    "context rule '{}' has an unsafe path glob",
                    rule.id
                ));
            }
        }
    }
    Ok(policy)
}

impl From<HarnessContext> for SqliteHarnessRepository {
    fn from(context: HarnessContext) -> Self {
        Self::new(context.repo_root, context.db_path, context.schema_dir)
    }
}

#[derive(Debug)]
struct MatrixColumns {
    story: Option<usize>,
    contract: Option<usize>,
    unit: Option<usize>,
    integration: Option<usize>,
    e2e: Option<usize>,
    platform: Option<usize>,
    status: Option<usize>,
    evidence: Option<usize>,
}

#[derive(Debug, Default)]
struct BacklogMarkdownItem {
    title: String,
    discovered_while: String,
    current_pain: String,
    suggested_improvement: String,
    risk: String,
    status: String,
}

impl MatrixColumns {
    fn from_header(fields: &[String]) -> Self {
        let mut columns = Self {
            story: None,
            contract: None,
            unit: None,
            integration: None,
            e2e: None,
            platform: None,
            status: None,
            evidence: None,
        };

        for (index, field) in fields.iter().enumerate() {
            match normalize_token(field).as_str() {
                "story" => columns.story = Some(index),
                "contract" => columns.contract = Some(index),
                "unit" => columns.unit = Some(index),
                "integration" => columns.integration = Some(index),
                "e2e" => columns.e2e = Some(index),
                "platform" => columns.platform = Some(index),
                "status" => columns.status = Some(index),
                "evidence" => columns.evidence = Some(index),
                _ => {}
            }
        }

        columns
    }
}

fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>>,
) -> Result<Vec<T>> {
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(HarnessInfraError::from)
}

fn trace_score_source_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TraceScoreSource> {
    Ok(TraceScoreSource {
        id: row.get(0)?,
        task_summary: row.get(1)?,
        intake_id: row.get(2)?,
        risk_lane: row.get(3)?,
        agent: row.get(4)?,
        actions_taken: row.get(5)?,
        files_read: row.get(6)?,
        files_changed: row.get(7)?,
        decisions_made: row.get(8)?,
        errors: row.get(9)?,
        outcome: row.get(10)?,
        duration_seconds: row.get(11)?,
        token_estimate: row.get(12)?,
        harness_friction: row.get(13)?,
        notes: row.get(14)?,
    })
}

fn markdown_table_fields(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let trimmed = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let trimmed = trimmed.strip_suffix('|').unwrap_or(trimmed);
    trimmed
        .split('|')
        .map(|field| field.trim().to_owned())
        .collect()
}

fn field_at(fields: &[String], index: Option<usize>) -> Option<String> {
    index
        .and_then(|value| fields.get(value))
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn evidence_from_fields(fields: &[String], start_index: usize) -> Option<String> {
    fields
        .get(start_index..)
        .map(|values| values.join(" | "))
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn proof_from_cell(value: &str) -> i64 {
    match normalize_token(value).as_str() {
        ""
        | "no"
        | "none"
        | "n_a"
        | "na"
        | "planned"
        | "pending"
        | "blocked"
        | "not_attempted"
        | "not_operator_reviewed" => 0,
        token
            if token.starts_with("no_")
                || token.starts_with("pending")
                || token.starts_with("blocked")
                || token.contains("pending")
                || token.contains("blocked")
                || token.contains("not_attempted")
                || token.contains("not_operator_reviewed") =>
        {
            0
        }
        _ => 1,
    }
}

fn normalize_story_status(value: &str) -> String {
    match normalize_token(value).as_str() {
        "planned" => "planned",
        "in_progress" => "in_progress",
        "implemented" => "implemented",
        "changed" => "changed",
        "retired" => "retired",
        _ => "planned",
    }
    .to_owned()
}

fn normalize_decision_status(value: &str) -> String {
    let token = normalize_token(value);
    match token.as_str() {
        "proposed" => "proposed",
        "accepted" => "accepted",
        "superseded" => "superseded",
        "rejected" => "rejected",
        token if token.starts_with("superseded_") => "superseded",
        _ => "accepted",
    }
    .to_owned()
}

fn normalize_backlog_status(value: &str) -> String {
    match normalize_token(value).as_str() {
        "proposed" => "proposed",
        "accepted" => "accepted",
        "implemented" => "implemented",
        "rejected" => "rejected",
        _ => "proposed",
    }
    .to_owned()
}

fn markdown_section_first_value(content: &str, heading: &str) -> String {
    let target = format!("## {heading}");
    let mut found = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if found && !trimmed.is_empty() {
            return trimmed.to_owned();
        }
        if trimmed == target {
            found = true;
        }
    }
    String::new()
}

fn backlog_items(content: &str) -> Vec<BacklogMarkdownItem> {
    let mut in_items = false;
    let mut current_heading = String::new();
    let mut current = BacklogMarkdownItem::default();
    let mut items = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "## Items" {
            in_items = true;
            current_heading.clear();
            continue;
        }
        if !in_items {
            continue;
        }

        if let Some(heading) = trimmed.strip_prefix("### ") {
            let normalized = normalize_token(heading);
            if normalized == "title" && !current.title.is_empty() {
                items.push(current);
                current = BacklogMarkdownItem::default();
            }
            current_heading = normalized;
            continue;
        }

        if trimmed.is_empty() || current_heading.is_empty() {
            continue;
        }

        let target = match current_heading.as_str() {
            "title" => &mut current.title,
            "discovered_while" => &mut current.discovered_while,
            "current_pain" => &mut current.current_pain,
            "suggested_improvement" => &mut current.suggested_improvement,
            "risk" => &mut current.risk,
            "status" => &mut current.status,
            _ => continue,
        };
        if target.is_empty() {
            *target = trimmed.to_owned();
        }
    }

    if !current.title.is_empty() {
        items.push(current);
    }
    items
}

fn empty_to_none(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn command_available(repo_root: &Path, command: &str) -> bool {
    let first = command.split_whitespace().next().unwrap_or(command);
    if first.is_empty() {
        return false;
    }
    let candidate = Path::new(first);
    if candidate.is_absolute() {
        return candidate.exists();
    }
    if first.contains('/') || first.contains('\\') {
        return repo_root.join(first).exists();
    }
    env::var_os("PATH")
        .is_some_and(|path| env::split_paths(&path).any(|dir| dir.join(first).exists()))
}

/// Kind-aware presence probe. Returns `(status, detail)` where status is one of
/// `present` / `missing` / `unknown`. It never fails: an absent extension is a
/// fact to report, not an error to raise.
fn scan_tool_status(
    repo_root: &Path,
    kind: &str,
    command: &str,
    scan_target: Option<&str>,
) -> (&'static str, String) {
    match kind {
        "cli" | "binary" => {
            if command_available(repo_root, command) {
                ("present", command.to_owned())
            } else {
                ("missing", command.to_owned())
            }
        }
        "mcp" | "skill" => match scan_target.map(str::trim).filter(|t| !t.is_empty()) {
            Some(target) => {
                if scan_target_resolves(repo_root, target) {
                    ("present", target.to_owned())
                } else {
                    ("missing", target.to_owned())
                }
            }
            None => (
                "unknown",
                "no scan target; agent confirms availability".to_owned(),
            ),
        },
        "http" => match scan_target.map(str::trim).filter(|t| !t.is_empty()) {
            Some(target) => {
                if http_reachable(target) || scan_target_resolves(repo_root, target) {
                    ("present", target.to_owned())
                } else {
                    ("missing", target.to_owned())
                }
            }
            None => ("unknown", "no scan target".to_owned()),
        },
        _ => ("unknown", String::new()),
    }
}

/// Resolve a declarative scan target as a filesystem path: `~` expands to HOME,
/// absolute paths are tested directly, relative paths are tested against the
/// repo root.
fn scan_target_resolves(repo_root: &Path, target: &str) -> bool {
    let expanded = expand_home(target);
    let path = Path::new(&expanded);
    if path.is_absolute() {
        path.exists()
    } else {
        repo_root.join(&expanded).exists()
    }
}

fn expand_home(target: &str) -> String {
    if let Some(rest) = target.strip_prefix("~/") {
        if let Some(home) = env::var_os("HOME") {
            return format!("{}/{}", home.to_string_lossy(), rest);
        }
    }
    target.to_owned()
}

/// Best-effort TCP reachability for `http`/`https` scan targets. Any failure
/// (parse, DNS, timeout, refused) is reported as not reachable rather than an
/// error, so a down endpoint degrades the capability instead of breaking intake.
fn http_reachable(target: &str) -> bool {
    use std::net::{TcpStream, ToSocketAddrs};
    use std::time::Duration;

    let (default_port, rest) = if let Some(rest) = target.strip_prefix("https://") {
        (443u16, rest)
    } else if let Some(rest) = target.strip_prefix("http://") {
        (80u16, rest)
    } else {
        return false;
    };

    let authority = rest.split('/').next().unwrap_or("");
    if authority.is_empty() {
        return false;
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (host, port.parse::<u16>().unwrap_or(default_port)),
        None => (authority, default_port),
    };

    let Ok(addresses) = (host, port).to_socket_addrs() else {
        return false;
    };
    addresses
        .into_iter()
        .any(|address| TcpStream::connect_timeout(&address, Duration::from_secs(2)).is_ok())
}

fn tool_args_json(args: &[ToolArgSpec]) -> Option<String> {
    if args.is_empty() {
        return None;
    }
    Some(format!(
        "[{}]",
        args.iter()
            .map(|arg| {
                format!(
                    "{{\"name\":\"{}\",\"type\":\"{}\",\"required\":{},\"help\":\"{}\"}}",
                    escape_json(&arg.name),
                    escape_json(&arg.arg_type),
                    arg.required,
                    escape_json(arg.help.as_deref().unwrap_or(""))
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    ))
}

fn parse_stored_tool_args(value: Option<&str>) -> Vec<ToolArgSpec> {
    let Some(value) = value else {
        return Vec::new();
    };
    if !value.contains("\"name\"") {
        return Vec::new();
    }
    value
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split("},{")
        .filter_map(|raw| {
            let item = raw.trim_matches('{').trim_matches('}');
            let name = json_object_value(item, "name")?;
            let arg_type = json_object_value(item, "type").unwrap_or_else(|| "string".to_owned());
            let required = json_object_value(item, "required")
                .map(|value| value == "true")
                .unwrap_or(false);
            let help = json_object_value(item, "help").filter(|value| !value.is_empty());
            Some(ToolArgSpec {
                name,
                arg_type,
                required,
                help,
            })
        })
        .collect()
}

fn json_object_value(raw: &str, key: &str) -> Option<String> {
    let target = format!("\"{key}\":");
    let start = raw.find(&target)? + target.len();
    let rest = &raw[start..];
    if let Some(rest) = rest.strip_prefix('"') {
        let end = rest.find('"')?;
        Some(rest[..end].to_owned())
    } else {
        Some(rest.split(',').next().unwrap_or_default().trim().to_owned())
    }
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn audit_findings(connection: &Connection, sql: &str) -> Result<Vec<AuditFinding>> {
    let mut statement = connection.prepare(sql)?;
    let rows = statement.query_map([], |row| {
        Ok(AuditFinding {
            id: row.get(0)?,
            title: row.get(1)?,
        })
    })?;
    collect_rows(rows)
}

fn repeated_friction(connection: &Connection) -> Result<Vec<(String, usize)>> {
    let mut statement = connection.prepare(
        "SELECT harness_friction FROM trace
         WHERE harness_friction IS NOT NULL
           AND TRIM(harness_friction) <> ''
           AND LOWER(TRIM(harness_friction)) <> 'none';",
    )?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    let values = collect_rows(rows)?;
    Ok(repeated_values(values))
}

fn repeated_interventions(connection: &Connection) -> Result<Vec<(String, usize)>> {
    let mut statement = connection.prepare(
        "SELECT type || ': ' || description FROM intervention
         WHERE TRIM(description) <> '';",
    )?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    let values = collect_rows(rows)?;
    Ok(repeated_values(values))
}

fn repeated_values(values: Vec<String>) -> Vec<(String, usize)> {
    let mut grouped: Vec<(String, String, usize)> = Vec::new();
    for value in values {
        let key = normalize_token(&value);
        if let Some(existing) = grouped.iter_mut().find(|item| item.0 == key) {
            existing.2 += 1;
        } else {
            grouped.push((key, value, 1));
        }
    }
    grouped
        .into_iter()
        .filter(|(_, _, count)| *count >= 2)
        .map(|(_, value, count)| (value, count))
        .collect()
}

fn confidence_for_count(count: usize) -> String {
    if count >= 3 {
        "high".to_owned()
    } else {
        "medium".to_owned()
    }
}

fn short_title(value: &str) -> String {
    let words = value
        .split_whitespace()
        .take(8)
        .collect::<Vec<_>>()
        .join(" ");
    if words.len() > 72 {
        format!("{}...", &words[..69])
    } else {
        words
    }
}

fn verifier_shell() -> (&'static str, &'static str) {
    if cfg!(windows) {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    }
}

fn is_decision_file_name(file_name: &str) -> bool {
    let Some((prefix, _)) = file_name.split_once('-') else {
        return false;
    };
    prefix.len() == 4 && prefix.chars().all(|character| character.is_ascii_digit())
}

fn sql_value_to_string(value: ValueRef<'_>) -> String {
    match value {
        ValueRef::Null => String::new(),
        ValueRef::Integer(value) => value.to_string(),
        ValueRef::Real(value) => value.to_string(),
        ValueRef::Text(value) => String::from_utf8_lossy(value).into_owned(),
        ValueRef::Blob(value) => format!("<{} bytes>", value.len()),
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::application::{
        BacklogAddInput, BacklogCloseInput, DecisionAddInput, IntakeInput, InterventionAddInput,
        InterventionFilter, StoryAddInput, StoryUpdateInput, ToolRegisterInput, TraceInput,
    };
    use crate::domain::{BacklogFilter, BoolFlag, CsvList, InputType, RiskLane, TraceQualityTier};

    fn source_repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    fn test_schema_dir() -> PathBuf {
        source_repo_root().join("_harness/scripts/schema")
    }

    fn test_repository() -> (TempDir, SqliteHarnessRepository) {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_root = source_repo_root();
        let repository = SqliteHarnessRepository::new(
            repo_root.clone(),
            temp_dir.path().join("harness.db"),
            test_schema_dir(),
        );
        (temp_dir, repository)
    }

    fn doctor_repository() -> (TempDir, SqliteHarnessRepository) {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_root = temp_dir.path().join("repo");
        let schema_dir = repo_root.join("_harness/scripts/schema");
        fs::create_dir_all(&schema_dir).unwrap();
        for entry in fs::read_dir(test_schema_dir()).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                fs::copy(&path, schema_dir.join(path.file_name().unwrap())).unwrap();
            }
        }
        fs::create_dir_all(&repo_root).unwrap();
        assert!(Command::new("git")
            .arg("init")
            .arg("-q")
            .arg(&repo_root)
            .status()
            .unwrap()
            .success());
        fs::create_dir_all(repo_root.join("_harness/bin")).unwrap();
        fs::write(repo_root.join("AGENTS.md"), "# test\n").unwrap();
        fs::write(repo_root.join(".harness-id"), "test-repository-id\n").unwrap();
        fs::write(repo_root.join("_harness/bin/harness-cli"), "test binary\n").unwrap();
        fs::copy(
            source_repo_root().join("_harness/workflow.toml"),
            repo_root.join("_harness/workflow.toml"),
        )
        .unwrap();
        fs::write(
            repo_root.join(".gitignore"),
            "harness.db\nharness.db-wal\nharness.db-shm\n",
        )
        .unwrap();
        assert!(Command::new("git")
            .arg("-C")
            .arg(&repo_root)
            .args(["add", "."])
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .arg("-C")
            .arg(&repo_root)
            .args([
                "-c",
                "user.name=Harness Test",
                "-c",
                "user.email=harness@example.test",
                "commit",
                "-q",
                "-m",
                "initial fixture",
            ])
            .status()
            .unwrap()
            .success());
        let repository = SqliteHarnessRepository::new(
            repo_root.clone(),
            repo_root.join("harness.db"),
            schema_dir,
        );
        (temp_dir, repository)
    }

    fn story_columns(connection: &Connection) -> Vec<String> {
        let mut statement = connection.prepare("PRAGMA table_info(story);").unwrap();
        let rows = statement
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap();
        rows.collect::<std::result::Result<Vec<_>, _>>().unwrap()
    }

    #[test]
    fn init_creates_database_and_schema() {
        let (_temp_dir, repository) = test_repository();

        let result = repository.init().unwrap();

        assert!(matches!(result, InitResult::Created { .. }));
        assert_eq!(repository.query_stats().unwrap().intakes, 0);
        let connection = repository.open_existing().unwrap();
        let schema_version = SqliteHarnessRepository::schema_version(&connection).unwrap();
        assert_eq!(schema_version, 6);
        let story_columns = story_columns(&connection);
        assert!(story_columns.contains(&"verify_command".to_owned()));
        assert!(story_columns.contains(&"last_verified_at".to_owned()));
        assert!(story_columns.contains(&"last_verified_result".to_owned()));
    }

    #[test]
    fn doctor_reports_missing_database_without_creating_it() {
        let (_temp_dir, repository) = doctor_repository();
        let db_path = repository.db_path.clone();

        let report = repository.doctor().unwrap();

        assert_eq!(report.code, "DB_MISSING");
        assert!(report.ok);
        assert!(!db_path.exists());
        assert_eq!(report.source_versions, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn doctor_detects_ahead_database_and_never_mutates_it() {
        let (_temp_dir, repository) = doctor_repository();
        repository.init().unwrap();
        let connection = repository.open_existing().unwrap();
        connection
            .execute("INSERT INTO schema_version(version) VALUES (7)", [])
            .unwrap();
        drop(connection);
        let db_path = repository.db_path.clone();
        let before = sha256_file(&db_path).unwrap();

        let report = repository.doctor().unwrap();

        assert_eq!(report.code, "DB_AHEAD_OF_SOURCE");
        assert!(!report.ok);
        assert_eq!(sha256_file(&db_path).unwrap(), before);
        let connection = repository.open_existing().unwrap();
        assert_eq!(
            SqliteHarnessRepository::schema_version(&connection).unwrap(),
            7
        );
    }

    #[test]
    fn doctor_accepts_a_checksum_and_lineage_verified_latest_database() {
        let (_temp_dir, repository) = doctor_repository();
        repository.init().unwrap();

        let report = repository.doctor().unwrap();

        assert_eq!(report.code, "HEALTHY");
        assert!(report.ok);
        assert!(report.findings.is_empty());
        assert_eq!(report.repository_id.as_deref(), Some("test-repository-id"));
        assert_eq!(
            report.worktree.as_deref(),
            Some(repository.repo_root.to_str().unwrap())
        );
        assert!(report.branch.is_some());
        assert!(report
            .commit
            .as_deref()
            .is_some_and(|value| value.len() == 40));
    }

    #[test]
    fn doctor_detects_migration_checksum_mismatch() {
        let (_temp_dir, repository) = doctor_repository();
        repository.init().unwrap();
        let connection = repository.open_existing().unwrap();
        connection
            .execute(
                "UPDATE migration_history SET checksum='wrong' WHERE version=1",
                [],
            )
            .unwrap();
        drop(connection);

        let report = repository.doctor().unwrap();

        assert_eq!(report.code, "DB_UNHEALTHY");
        assert!(report
            .findings
            .iter()
            .any(|finding| finding == "MIGRATION_CHECKSUM_MISMATCH:1"));
    }

    #[test]
    fn doctor_reports_unversioned_database() {
        let (_temp_dir, repository) = doctor_repository();
        let connection = repository.open_or_create().unwrap();
        connection
            .execute("CREATE TABLE unrelated (id INTEGER PRIMARY KEY)", [])
            .unwrap();
        drop(connection);

        let report = repository.doctor().unwrap();

        assert_eq!(report.code, "DB_UNVERSIONED");
        assert!(!report.ok);
    }

    #[test]
    fn doctor_reports_foreign_key_violations() {
        let (_temp_dir, repository) = doctor_repository();
        repository.init().unwrap();
        let connection = repository.open_existing().unwrap();
        connection
            .pragma_update(None, "foreign_keys", "OFF")
            .unwrap();
        connection
            .execute(
                "INSERT INTO trace(task_summary, intake_id, outcome) VALUES ('orphan', 999, 'completed')",
                [],
            )
            .unwrap();
        drop(connection);

        let report = repository.doctor().unwrap();

        assert_eq!(report.code, "DB_UNHEALTHY");
        assert!(report
            .findings
            .iter()
            .any(|finding| finding == "FOREIGN_KEY_VIOLATION"));
    }

    #[test]
    fn doctor_reports_invalid_workflow_policy_and_missing_payload() {
        let (_temp_dir, repository) = doctor_repository();
        fs::write(
            repository.repo_root.join("_harness/workflow.toml"),
            "policy_version = \"2.0\"\n",
        )
        .unwrap();
        fs::remove_file(repository.repo_root.join(".harness-id")).unwrap();
        repository.init().unwrap();

        let report = repository.doctor().unwrap();

        assert_eq!(report.code, "DB_UNHEALTHY");
        assert!(report
            .findings
            .iter()
            .any(|finding| finding == "REQUIRED_PATH_MISSING:.harness-id"));
        assert!(report
            .findings
            .iter()
            .any(|finding| finding.starts_with("WORKFLOW_INVALID:")));
    }

    #[test]
    fn workflow_policy_rejects_unknown_keys_and_path_traversal() {
        let (_temp_dir, repository) = doctor_repository();
        let workflow = repository.repo_root.join("_harness/workflow.toml");
        let original = fs::read_to_string(&workflow).unwrap();
        fs::write(&workflow, format!("{original}\nunknown_key = true\n")).unwrap();
        assert!(repository.workflow_policy().is_err());
        fs::write(
            &workflow,
            original.replace(
                "product_docs = \"docs/product\"",
                "product_docs = \"../escape\"",
            ),
        )
        .unwrap();
        assert!(repository.workflow_policy().is_err());
    }

    #[test]
    fn workflow_policy_classifies_boundaries_from_config() {
        let (_temp_dir, repository) = doctor_repository();
        let policy = repository.workflow_policy().unwrap();
        assert_eq!(policy.classify(&[]).0, "tiny");
        assert_eq!(
            policy.classify(&["ui".to_owned(), "api".to_owned()]).0,
            "normal"
        );
        let (lane, gates) = policy.classify(&["auth".to_owned()]);
        assert_eq!(lane, "high_risk");
        assert!(gates.contains(&"hard-gate:auth".to_owned()));
        assert_eq!(
            policy.classify(&["external systems".to_owned()]).0,
            "high_risk"
        );
        assert_eq!(
            policy.classify(&["audit/security".to_owned()]).0,
            "high_risk"
        );
    }

    #[test]
    fn workflow_context_golden_lane_phase_matrix() {
        let (_temp_dir, repository) = doctor_repository();
        let policy = repository.workflow_policy().unwrap();

        let tiny_intake = policy.context_manifest("tiny", "intake", &[], &[], &[]);
        assert_eq!(tiny_intake.token_budget_hint, 2000);
        assert!(tiny_intake
            .must_read
            .iter()
            .any(|entry| entry.path == "AGENTS.md"));
        assert!(tiny_intake
            .skip
            .iter()
            .any(|entry| entry.path == "_harness/ARCHITECTURE.md"));

        let normal_planning = policy.context_manifest("normal", "planning", &[], &[], &[]);
        assert_eq!(normal_planning.token_budget_hint, 5000);
        assert!(normal_planning
            .must_read
            .iter()
            .any(|entry| entry.path == "_harness/templates/story.md"));

        let high_finish = policy.context_manifest(
            "high-risk",
            "finish",
            &[],
            &["audit/security".to_owned()],
            &[],
        );
        assert_eq!(high_finish.token_budget_hint, 10000);
        assert!(high_finish
            .must_read
            .iter()
            .any(|entry| entry.path == "_harness/TRACE_SPEC.md"));
        assert!(high_finish
            .must_read
            .iter()
            .any(|entry| entry.path == "docs/decisions/"));
    }

    #[test]
    fn workflow_context_excludes_unrelated_docs_from_must_read() {
        let (_temp_dir, repository) = doctor_repository();
        let policy = repository.workflow_policy().unwrap();
        let manifest = policy.context_manifest("tiny", "work", &["README.md".to_owned()], &[], &[]);
        assert!(!manifest
            .must_read
            .iter()
            .any(|entry| entry.path.starts_with("docs/decisions/")));
    }

    #[test]
    fn policy_parity_fixture_is_versioned_and_has_unique_case_ids() {
        let content =
            fs::read_to_string(source_repo_root().join("_harness/tests/policy-parity-cases.toml"))
                .unwrap();
        let fixture: toml::Value = toml::from_str(&content).unwrap();
        assert_eq!(fixture["schema_version"].as_integer(), Some(1));

        let mut ids = HashSet::new();
        for section in [
            "classification_cases",
            "context_cases",
            "intentional_deltas",
        ] {
            for case in fixture[section].as_array().unwrap() {
                let id = case["id"].as_str().unwrap();
                assert!(!id.is_empty());
                assert!(ids.insert(id.to_owned()), "duplicate parity case id: {id}");
            }
        }
    }

    #[test]
    fn workflow_context_deduplicates_matching_rules_with_reasons() {
        let (_temp_dir, repository) = doctor_repository();
        let policy = repository.workflow_policy().unwrap();
        let manifest = policy.context_manifest(
            "normal",
            "work",
            &[
                "crates/harness-cli/src/interface.rs".to_owned(),
                "_harness/scripts/schema/006-command-first-foundation.sql".to_owned(),
                "crates/harness-cli/src/application.rs".to_owned(),
            ],
            &[],
            &[],
        );
        assert!(manifest.must_read.iter().any(|entry| entry
            .path
            .ends_with("0005-prebuilt-rust-harness-cli.md")
            && entry.reason == "cli-distribution"));
        assert!(manifest
            .must_read
            .iter()
            .any(|entry| entry.path.ends_with("0004-sqlite-durable-layer.md")
                && entry.reason == "schema-change"));
        assert_eq!(
            manifest
                .must_read
                .iter()
                .filter(|entry| entry.path.ends_with("0005-prebuilt-rust-harness-cli.md"))
                .count(),
            1
        );
        assert_eq!(manifest.token_budget_hint, 5000);
        assert_eq!(manifest.checksum.len(), 64);
    }

    #[test]
    fn doctor_rejects_a_source_migration_gap() {
        let (_temp_dir, repository) = doctor_repository();
        fs::remove_file(repository.schema_dir.join("002-story-verify.sql")).unwrap();

        let report = repository.doctor().unwrap();

        assert_eq!(report.code, "SOURCE_MIGRATION_INVALID");
        assert!(report
            .findings
            .iter()
            .any(|finding| finding == "MIGRATION_INVENTORY_INVALID"));
        assert!(report
            .findings
            .iter()
            .any(|finding| finding == "MANIFEST_SOURCE_MISSING:002-story-verify.sql"));
    }

    #[test]
    fn doctor_rejects_a_duplicate_source_migration_version() {
        let (_temp_dir, repository) = doctor_repository();
        fs::copy(
            repository.schema_dir.join("001-init.sql"),
            repository.schema_dir.join("001-duplicate.sql"),
        )
        .unwrap();

        let report = repository.doctor().unwrap();

        assert_eq!(report.code, "SOURCE_MIGRATION_INVALID");
        assert!(report
            .findings
            .iter()
            .any(|finding| finding == "MIGRATION_DUPLICATE_VERSION"));
    }

    #[test]
    fn doctor_rejects_a_foreign_schema_lineage() {
        let (_temp_dir, repository) = doctor_repository();
        repository.init().unwrap();
        let connection = repository.open_existing().unwrap();
        connection
            .execute(
                "UPDATE harness_meta SET value='foreign' WHERE key='schema_lineage'",
                [],
            )
            .unwrap();
        drop(connection);

        let report = repository.doctor().unwrap();

        assert_eq!(report.code, "DB_UNHEALTHY");
        assert!(report
            .findings
            .iter()
            .any(|finding| finding == "SCHEMA_LINEAGE_MISMATCH"));
    }

    #[test]
    fn doctor_rejects_a_corrupt_database_without_repairing_it() {
        let (_temp_dir, repository) = doctor_repository();
        fs::write(&repository.db_path, b"not a sqlite database").unwrap();
        let before = fs::read(&repository.db_path).unwrap();

        let report = repository.doctor().unwrap();

        assert_eq!(report.code, "DB_UNREADABLE");
        assert_eq!(fs::read(&repository.db_path).unwrap(), before);
    }

    #[test]
    fn doctor_reports_a_legacy_version_one_database_as_behind() {
        let (_temp_dir, repository) = doctor_repository();
        let connection = repository.open_or_create().unwrap();
        repository.apply_schema_v1(&connection).unwrap();
        drop(connection);

        let report = repository.doctor().unwrap();

        assert_eq!(report.code, "DB_BEHIND_SOURCE");
        assert!(!report.ok);
        assert_eq!(report.db_versions, vec![1]);
    }

    #[test]
    fn migrate_applies_story_verify_columns_to_existing_database() {
        let (_temp_dir, repository) = doctor_repository();
        let connection = repository.open_or_create().unwrap();
        repository.apply_schema_v1(&connection).unwrap();
        drop(connection);

        let result = repository.migrate().unwrap();

        assert_eq!(result.current_version, 1);
        assert_eq!(result.applied, vec![2, 3, 4, 5, 6]);
        let connection = repository.open_existing().unwrap();
        assert_eq!(
            SqliteHarnessRepository::schema_version(&connection).unwrap(),
            6
        );
        let story_columns = story_columns(&connection);
        assert!(story_columns.contains(&"verify_command".to_owned()));
        assert!(story_columns.contains(&"last_verified_at".to_owned()));
        assert!(story_columns.contains(&"last_verified_result".to_owned()));
        let backups = fs::read_dir(repository.repo_root.join("harness.db.backups"))
            .unwrap()
            .collect::<std::result::Result<Vec<_>, _>>()
            .unwrap();
        assert!(backups
            .iter()
            .any(|entry| entry.file_name().to_string_lossy().ends_with(".bak")));
    }

    #[test]
    fn ensure_rolls_back_a_failing_migration_after_creating_a_backup() {
        let (_temp_dir, repository) = doctor_repository();
        let connection = repository.open_or_create().unwrap();
        repository.apply_schema_v1(&connection).unwrap();
        drop(connection);
        let migration = repository.schema_dir.join("002-story-verify.sql");
        fs::write(
            &migration,
            "ALTER TABLE story ADD COLUMN should_not_persist TEXT;\nINVALID SQL;\n",
        )
        .unwrap();
        let checksum = sha256_file(&migration).unwrap();
        let manifest = repository.schema_dir.join("manifest.toml");
        let content = fs::read_to_string(&manifest).unwrap().replace(
            "6e788f0a712e1280b196843b7e4e49a71f649e0c989913dc1b9d28e098df54de",
            &checksum,
        );
        fs::write(manifest, content).unwrap();

        assert!(matches!(
            repository.migrate(),
            Err(HarnessInfraError::Sqlite(_))
        ));

        let connection = repository.open_existing().unwrap();
        assert_eq!(
            SqliteHarnessRepository::schema_version(&connection).unwrap(),
            1
        );
        assert!(!story_columns(&connection).contains(&"should_not_persist".to_owned()));
        let backups = fs::read_dir(repository.repo_root.join("harness.db.backups"))
            .unwrap()
            .collect::<std::result::Result<Vec<_>, _>>()
            .unwrap();
        assert!(backups
            .iter()
            .any(|entry| entry.file_name().to_string_lossy().ends_with(".bak")));
    }

    #[test]
    fn backup_can_restore_the_pre_migration_database() {
        let (_temp_dir, repository) = doctor_repository();
        let connection = repository.open_or_create().unwrap();
        repository.apply_schema_v1(&connection).unwrap();
        drop(connection);
        repository.migrate().unwrap();
        let backup = fs::read_dir(repository.repo_root.join("harness.db.backups"))
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .find(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .ends_with(".bak")
            })
            .unwrap();
        for suffix in ["", "-wal", "-shm"] {
            let path = PathBuf::from(format!("{}{}", repository.db_path.display(), suffix));
            if path.exists() {
                fs::remove_file(&path).unwrap();
            }
            let backup_path = PathBuf::from(format!("{}{}", backup.display(), suffix));
            if backup_path.exists() {
                fs::copy(backup_path, path).unwrap();
            }
        }

        let connection = repository.open_existing().unwrap();
        assert_eq!(
            SqliteHarnessRepository::schema_version(&connection).unwrap(),
            1
        );
        drop(connection);
        assert_eq!(repository.doctor().unwrap().code, "DB_BEHIND_SOURCE");
    }

    #[test]
    fn backup_retention_preserves_the_newest_five_backups() {
        let (_temp_dir, repository) = doctor_repository();
        repository.init().unwrap();
        let checksum = repository
            .source_migrations()
            .unwrap()
            .0
            .last()
            .unwrap()
            .checksum
            .clone();
        let mut newest = PathBuf::new();
        for _ in 0..7 {
            newest = repository
                .backup_existing_database(6, "main", &checksum)
                .unwrap();
        }

        let backups = fs::read_dir(repository.repo_root.join("harness.db.backups"))
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().ends_with(".bak"))
            .collect::<Vec<_>>();
        assert_eq!(backups.len(), 5);
        assert!(newest.exists());
    }

    #[test]
    fn ensure_refuses_an_ahead_database_without_writing_or_backing_it_up() {
        let (_temp_dir, repository) = doctor_repository();
        repository.init().unwrap();
        let connection = repository.open_existing().unwrap();
        connection
            .execute("INSERT INTO schema_version(version) VALUES (7)", [])
            .unwrap();
        drop(connection);
        let before = sha256_file(&repository.db_path).unwrap();

        let error = repository.migrate().unwrap_err();

        assert!(matches!(error, HarnessInfraError::UnsafeDurableState(_)));
        assert_eq!(sha256_file(&repository.db_path).unwrap(), before);
        assert!(!repository.repo_root.join("harness.db.backups").exists());
    }

    #[test]
    fn ensure_is_idempotent_for_a_healthy_latest_database() {
        let (_temp_dir, repository) = doctor_repository();
        repository.init().unwrap();

        let result = repository.migrate().unwrap();

        assert_eq!(result.current_version, 6);
        assert!(result.applied.is_empty());
        assert!(!repository.repo_root.join("harness.db.backups").exists());
    }

    #[test]
    fn query_sql_enforces_a_read_only_single_statement_boundary() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();
        let before = sha256_file(&repository.db_path).unwrap();

        for statement in [
            "INSERT INTO story(id, title, risk_lane) VALUES ('US-999', 'bad', 'tiny')",
            "UPDATE story SET title='bad'",
            "DELETE FROM story",
            "CREATE TABLE escaped (id INTEGER)",
            "DROP TABLE story",
            "ATTACH DATABASE 'other.db' AS other",
            "PRAGMA foreign_keys=OFF",
            "VACUUM",
            "SELECT 1; DELETE FROM story",
        ] {
            assert!(matches!(
                repository.query_sql(statement),
                Err(HarnessInfraError::UnsafeSql)
            ));
            assert_eq!(sha256_file(&repository.db_path).unwrap(), before);
        }
        assert_eq!(
            repository.query_sql("SELECT 1 AS value").unwrap().rows,
            vec![vec!["1".to_owned()]]
        );
        assert_eq!(
            repository
                .query_sql("WITH value AS (SELECT 2) SELECT * FROM value")
                .unwrap()
                .rows,
            vec![vec!["2".to_owned()]]
        );
        assert!(repository.query_sql("PRAGMA table_info(story)").is_ok());
    }

    #[test]
    fn migration_005_backfills_kind_from_command_prefix() {
        let (_temp_dir, repository) = doctor_repository();
        let schema_dir = repository.schema_dir.clone();

        // Build a pre-kind (v4) database: v1 base plus migrations 002-004 only.
        let connection = repository.open_or_create().unwrap();
        repository.apply_schema_v1(&connection).unwrap();
        for file in [
            "002-story-verify.sql",
            "003-tool-registry.sql",
            "004-intervention.sql",
        ] {
            let sql = std::fs::read_to_string(schema_dir.join(file)).unwrap();
            connection.execute_batch(&sql).unwrap();
        }
        assert_eq!(
            SqliteHarnessRepository::schema_version(&connection).unwrap(),
            4
        );

        // Insert tools the old way (no kind column existed yet).
        for (name, command) in [
            ("mcp-example", "mcp:example-server"),
            ("skill-example", "skill:example-skill"),
            ("cli-example", "./deploy.sh"),
        ] {
            connection
                .execute(
                    "INSERT INTO tool (name, command, description, responsibility)
                     VALUES (?1, ?2, 'pre-kind registered tool example', 'Verification');",
                    params![name, command],
                )
                .unwrap();
        }
        drop(connection);

        // Upgrade: migration 005 must infer kind from the command prefix.
        assert_eq!(repository.migrate().unwrap().applied, vec![5, 6]);
        let connection = repository.open_existing().unwrap();
        let kind_of = |name: &str| -> String {
            connection
                .query_row(
                    "SELECT kind FROM tool WHERE name=?1;",
                    params![name],
                    |row| row.get::<_, String>(0),
                )
                .unwrap()
        };
        assert_eq!(kind_of("mcp-example"), "mcp");
        assert_eq!(kind_of("skill-example"), "skill");
        assert_eq!(kind_of("cli-example"), "cli");
    }

    #[test]
    fn records_and_queries_intake() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();

        let id = repository
            .record_intake(IntakeInput {
                input_type: InputType::HarnessImprovement,
                summary: "Port one CLI slice".to_owned(),
                risk_lane: RiskLane::HighRisk,
                risk_flags: CsvList::from_optional(Some("public contracts".to_owned())),
                affected_docs: CsvList::from_optional(None),
                story_id: Some("US-002".to_owned()),
                notes: None,
            })
            .unwrap();

        let intakes = repository.query_intakes().unwrap();
        assert_eq!(id, 1);
        assert_eq!(intakes[0].summary, "Port one CLI slice");
        assert_eq!(intakes[0].input_type, "harness_improvement");
        assert_eq!(intakes[0].risk_lane, "high_risk");

        let connection = repository.open_existing().unwrap();
        let missing_lists_are_null: (bool, bool) = connection
            .query_row(
                "SELECT risk_flags IS NULL, affected_docs IS NULL FROM intake WHERE id=?1;",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(missing_lists_are_null, (false, true));
    }

    #[test]
    fn decision_verify_runs_from_repo_root() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_root = temp_dir.path().join("repo");
        fs::create_dir_all(&repo_root).unwrap();
        let repository = SqliteHarnessRepository::new(
            repo_root.clone(),
            temp_dir.path().join("harness.db"),
            test_schema_dir(),
        );
        repository.init().unwrap();

        let pwd_output = repo_root.join("verify-pwd.txt");
        let verify_command = if cfg!(windows) {
            "cd > verify-pwd.txt".to_owned()
        } else {
            "pwd > verify-pwd.txt".to_owned()
        };
        repository
            .add_decision(DecisionAddInput {
                id: "0001-test".to_owned(),
                title: "Verify from root".to_owned(),
                status: "accepted".to_owned(),
                doc_path: None,
                verify_command: Some(verify_command),
                predicted_impact: None,
                notes: None,
            })
            .unwrap();

        let result = repository.verify_decision("0001-test").unwrap();

        assert_eq!(result.result, "pass");
        assert_eq!(
            fs::canonicalize(fs::read_to_string(pwd_output).unwrap().trim()).unwrap(),
            fs::canonicalize(repo_root).unwrap()
        );
    }

    #[test]
    fn story_add_update_and_verify_status_store_verify_command() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();

        repository
            .add_story(StoryAddInput {
                id: "US-VERIFY".to_owned(),
                title: "Verify command story".to_owned(),
                risk_lane: RiskLane::Normal,
                contract_doc: None,
                verify_command: Some("echo ok".to_owned()),
                notes: None,
            })
            .unwrap();
        assert_eq!(
            repository
                .story_verify_status("US-VERIFY")
                .unwrap()
                .verify_command
                .as_deref(),
            Some("echo ok")
        );

        repository
            .update_story(StoryUpdateInput {
                id: "US-VERIFY".to_owned(),
                status: None,
                evidence: None,
                unit: None,
                integration: None,
                e2e: None,
                platform: None,
                verify_command: Some("npm test".to_owned()),
            })
            .unwrap();

        assert_eq!(
            repository
                .story_verify_status("US-VERIFY")
                .unwrap()
                .verify_command
                .as_deref(),
            Some("npm test")
        );
    }

    #[test]
    fn story_verify_records_pass_fail_and_missing_command() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_root = temp_dir.path().join("repo");
        fs::create_dir_all(&repo_root).unwrap();
        let repository = SqliteHarnessRepository::new(
            repo_root.clone(),
            temp_dir.path().join("harness.db"),
            test_schema_dir(),
        );
        repository.init().unwrap();

        let pwd_output = repo_root.join("story-verify-pwd.txt");
        let verify_command = if cfg!(windows) {
            "cd > story-verify-pwd.txt".to_owned()
        } else {
            "pwd > story-verify-pwd.txt".to_owned()
        };
        repository
            .add_story(StoryAddInput {
                id: "US-PASS".to_owned(),
                title: "Passing story".to_owned(),
                risk_lane: RiskLane::Normal,
                contract_doc: None,
                verify_command: Some(verify_command),
                notes: None,
            })
            .unwrap();
        let pass = repository.verify_story("US-PASS").unwrap();
        assert_eq!(pass.result, "pass");
        assert_eq!(
            fs::canonicalize(fs::read_to_string(pwd_output).unwrap().trim()).unwrap(),
            fs::canonicalize(repo_root).unwrap()
        );
        assert_eq!(
            repository
                .story_verify_status("US-PASS")
                .unwrap()
                .last_verified_result
                .as_deref(),
            Some("pass")
        );

        repository
            .add_story(StoryAddInput {
                id: "US-FAIL".to_owned(),
                title: "Failing story".to_owned(),
                risk_lane: RiskLane::Normal,
                contract_doc: None,
                verify_command: Some("exit 1".to_owned()),
                notes: None,
            })
            .unwrap();
        let fail = repository.verify_story("US-FAIL").unwrap();
        assert_eq!(fail.result, "fail");
        assert_eq!(
            repository
                .story_verify_status("US-FAIL")
                .unwrap()
                .last_verified_result
                .as_deref(),
            Some("fail")
        );

        repository
            .add_story(StoryAddInput {
                id: "US-MISSING".to_owned(),
                title: "Missing command story".to_owned(),
                risk_lane: RiskLane::Normal,
                contract_doc: None,
                verify_command: None,
                notes: None,
            })
            .unwrap();
        assert!(matches!(
            repository.verify_story("US-MISSING"),
            Err(HarnessInfraError::MissingStoryVerifyCommand(id)) if id == "US-MISSING"
        ));
    }

    #[test]
    fn story_verify_all_reports_pass_fail_and_skipped() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();
        for (id, command) in [
            ("US-PASS", Some("exit 0")),
            ("US-FAIL", Some("exit 1")),
            ("US-SKIP", None),
        ] {
            repository
                .add_story(StoryAddInput {
                    id: id.to_owned(),
                    title: id.to_owned(),
                    risk_lane: RiskLane::Normal,
                    contract_doc: None,
                    verify_command: command.map(str::to_owned),
                    notes: None,
                })
                .unwrap();
        }

        let result = repository.verify_all_stories().unwrap();

        assert_eq!(result.passed(), 1);
        assert_eq!(result.failed(), 1);
        assert_eq!(result.skipped(), 1);
        assert_eq!(
            repository
                .story_verify_status("US-PASS")
                .unwrap()
                .last_verified_result
                .as_deref(),
            Some("pass")
        );
        assert_eq!(
            repository
                .story_verify_status("US-FAIL")
                .unwrap()
                .last_verified_result
                .as_deref(),
            Some("fail")
        );
    }

    #[test]
    fn tool_registry_register_query_and_remove_work() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();

        repository
            .register_tool(ToolRegisterInput {
                name: "deploy-check".to_owned(),
                command: "definitely-missing-tool".to_owned(),
                description: "Verify deploy health before release".to_owned(),
                responsibility: "Verification".to_owned(),
                args: Vec::new(),
                force: true,
                kind: "cli".to_owned(),
                capability: Some("deploy-verification".to_owned()),
                scan_target: None,
            })
            .unwrap();
        assert!(matches!(
            repository.register_tool(ToolRegisterInput {
                name: "deploy-check".to_owned(),
                command: "definitely-missing-tool".to_owned(),
                description: "Verify deploy health before release".to_owned(),
                responsibility: "Verification".to_owned(),
                args: Vec::new(),
                force: true,
                kind: "cli".to_owned(),
                capability: Some("deploy-verification".to_owned()),
                scan_target: None,
            }),
            Err(HarnessInfraError::ToolAlreadyExists(_, _))
        ));

        let verification_tools = repository
            .query_tools(Some("Verification".to_owned()), None)
            .unwrap();
        assert!(verification_tools
            .iter()
            .any(|tool| tool.name == "deploy-check" && tool.source == "registered"));

        // Capability lookup returns the registered provider.
        let by_capability = repository
            .query_tools(None, Some("deploy-verification".to_owned()))
            .unwrap();
        assert!(by_capability.iter().any(|tool| tool.name == "deploy-check"));

        repository.remove_tool("deploy-check").unwrap();
        assert!(!repository
            .query_tools(None, None)
            .unwrap()
            .iter()
            .any(|tool| tool.name == "deploy-check"));
    }

    #[test]
    fn tool_check_scans_and_persists_status_per_kind() {
        let (temp_dir, repository) = test_repository();
        repository.init().unwrap();

        // Absolute scan targets keep the test hermetic: test_repository's
        // repo_root points at the real project, so relative targets would
        // resolve against the checkout rather than the temp dir.
        let present_target = temp_dir.path().join("skill-present");
        std::fs::create_dir_all(&present_target).unwrap();
        let missing_target = temp_dir.path().join("mcp-missing");

        // An mcp tool whose scan target does not exist -> missing.
        repository
            .register_tool(ToolRegisterInput {
                name: "mcp-example".to_owned(),
                command: "mcp:example-server".to_owned(),
                description: "Example MCP-backed provider".to_owned(),
                responsibility: "Verification".to_owned(),
                args: Vec::new(),
                force: false,
                kind: "mcp".to_owned(),
                capability: Some("impact-analysis".to_owned()),
                scan_target: Some(missing_target.to_string_lossy().into_owned()),
            })
            .unwrap();

        // A skill tool whose scan target exists -> present.
        repository
            .register_tool(ToolRegisterInput {
                name: "skill-example".to_owned(),
                command: "skill:example-skill".to_owned(),
                description: "Example skill-backed provider".to_owned(),
                responsibility: "Verification".to_owned(),
                args: Vec::new(),
                force: false,
                kind: "skill".to_owned(),
                capability: Some("impact-analysis".to_owned()),
                scan_target: Some(present_target.to_string_lossy().into_owned()),
            })
            .unwrap();

        let results = repository.check_tools(None).unwrap();
        let mcp_tool = results.iter().find(|r| r.name == "mcp-example").unwrap();
        let skill_tool = results.iter().find(|r| r.name == "skill-example").unwrap();
        assert_eq!(mcp_tool.status, "missing");
        assert_eq!(skill_tool.status, "present");

        // Status is persisted, not just returned.
        let stored = repository
            .query_tools(None, Some("impact-analysis".to_owned()))
            .unwrap();
        assert_eq!(stored.len(), 2);
        assert!(stored
            .iter()
            .all(|tool| tool.checked_at.as_deref().is_some_and(|v| !v.is_empty())));
        assert_eq!(
            stored
                .iter()
                .find(|t| t.name == "skill-example")
                .unwrap()
                .status,
            "present"
        );
    }

    #[test]
    fn interventions_can_be_added_and_filtered() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();
        repository
            .add_story(StoryAddInput {
                id: "US-I".to_owned(),
                title: "Intervention story".to_owned(),
                risk_lane: RiskLane::Normal,
                contract_doc: None,
                verify_command: None,
                notes: None,
            })
            .unwrap();
        let trace_id = repository
            .record_trace(TraceInput {
                task_summary: "Trace for intervention".to_owned(),
                intake_id: None,
                story_id: Some("US-I".to_owned()),
                agent: Some("codex".to_owned()),
                outcome: Some("completed".to_owned()),
                duration_seconds: None,
                token_estimate: None,
                friction: Some("none".to_owned()),
                notes: None,
                actions: CsvList::from_optional(None),
                files_read: CsvList::from_optional(None),
                files_changed: CsvList::from_optional(None),
                decisions: CsvList::from_optional(None),
                errors: CsvList::from_optional(None),
            })
            .unwrap();
        repository
            .add_intervention(InterventionAddInput {
                trace_id: Some(trace_id),
                story_id: Some("US-I".to_owned()),
                intervention_type: "correction".to_owned(),
                description: "Use error handling instead of unwrap".to_owned(),
                source: "human".to_owned(),
                impact: Some("Reduced panic risk".to_owned()),
            })
            .unwrap();

        assert_eq!(
            repository
                .query_interventions(InterventionFilter {
                    trace_id: Some(trace_id),
                    story_id: None,
                    intervention_type: None,
                })
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            repository
                .query_interventions(InterventionFilter {
                    trace_id: None,
                    story_id: Some("US-I".to_owned()),
                    intervention_type: Some("override".to_owned()),
                })
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn audit_detects_drift_and_propose_can_commit_backlog_items() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();
        repository
            .add_story(StoryAddInput {
                id: "US-AUDIT".to_owned(),
                title: "Audit story".to_owned(),
                risk_lane: RiskLane::Normal,
                contract_doc: None,
                verify_command: Some("exit 0".to_owned()),
                notes: None,
            })
            .unwrap();
        repository
            .update_story(StoryUpdateInput {
                id: "US-AUDIT".to_owned(),
                status: Some("in_progress".to_owned()),
                evidence: None,
                unit: None,
                integration: None,
                e2e: None,
                platform: None,
                verify_command: None,
            })
            .unwrap();
        repository
            .add_backlog(BacklogAddInput {
                title: "Implemented without outcome".to_owned(),
                discovered_while: None,
                current_pain: None,
                suggestion: None,
                risk: Some(RiskLane::Tiny),
                predicted_impact: Some("Expected improvement".to_owned()),
                notes: None,
            })
            .unwrap();
        repository
            .close_backlog(BacklogCloseInput {
                id: 1,
                status: "implemented".to_owned(),
                actual_outcome: None,
            })
            .unwrap();
        repository
            .register_tool(ToolRegisterInput {
                name: "missing-tool".to_owned(),
                command: "definitely-missing-tool".to_owned(),
                description: "Missing command for audit coverage".to_owned(),
                responsibility: "Verification".to_owned(),
                args: Vec::new(),
                force: true,
                kind: "cli".to_owned(),
                capability: None,
                scan_target: None,
            })
            .unwrap();
        for _ in 0..2 {
            repository
                .record_trace(TraceInput {
                    task_summary: "Repeated friction trace".to_owned(),
                    intake_id: None,
                    story_id: None,
                    agent: Some("codex".to_owned()),
                    outcome: Some("completed".to_owned()),
                    duration_seconds: None,
                    token_estimate: None,
                    friction: Some("Context rules missed schema decision".to_owned()),
                    notes: None,
                    actions: CsvList::from_optional(Some("read".to_owned())),
                    files_read: CsvList::from_optional(Some("_harness/HARNESS.md".to_owned())),
                    files_changed: CsvList::from_optional(Some(
                        "_harness/scripts/schema/003-tool-registry.sql".to_owned(),
                    )),
                    decisions: CsvList::from_optional(None),
                    errors: CsvList::from_optional(None),
                })
                .unwrap();
        }

        let audit = repository.audit().unwrap();
        assert_eq!(audit.orphaned_stories.len(), 1);
        assert_eq!(audit.unverified_stories.len(), 1);
        assert_eq!(audit.backlog_without_outcomes.len(), 1);
        assert_eq!(audit.broken_tools.len(), 1);
        assert!(audit.entropy_score() > 0);

        let proposals = repository.propose(true).unwrap();
        assert!(proposals.iter().any(|proposal| proposal
            .evidence
            .contains("Context rules missed schema decision")));
        assert!(proposals
            .iter()
            .all(|proposal| proposal.committed_backlog_id.is_some()));
        assert!(!repository
            .query_backlog(BacklogFilter::Open)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn story_backlog_trace_and_queries_work() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();

        repository
            .add_story(StoryAddInput {
                id: "US-T".to_owned(),
                title: "Test story".to_owned(),
                risk_lane: RiskLane::Normal,
                contract_doc: None,
                verify_command: None,
                notes: None,
            })
            .unwrap();
        repository
            .update_story(StoryUpdateInput {
                id: "US-T".to_owned(),
                status: Some("implemented".to_owned()),
                evidence: Some("unit test".to_owned()),
                unit: Some(BoolFlag(1)),
                integration: None,
                e2e: None,
                platform: None,
                verify_command: None,
            })
            .unwrap();
        assert_eq!(repository.query_matrix().unwrap()[0].unit, 1);

        let backlog_id = repository
            .add_backlog(BacklogAddInput {
                title: "Improve CLI".to_owned(),
                discovered_while: None,
                current_pain: Some("manual SQL".to_owned()),
                suggestion: None,
                risk: Some(RiskLane::HighRisk),
                predicted_impact: None,
                notes: None,
            })
            .unwrap();
        repository
            .close_backlog(BacklogCloseInput {
                id: backlog_id,
                status: "implemented".to_owned(),
                actual_outcome: Some("done".to_owned()),
            })
            .unwrap();
        assert_eq!(
            repository.query_backlog(BacklogFilter::All).unwrap()[0]
                .actual_outcome
                .as_deref(),
            Some("done")
        );

        let trace_id = repository
            .record_trace(TraceInput {
                task_summary: "Test trace".to_owned(),
                intake_id: None,
                story_id: Some("US-T".to_owned()),
                agent: Some("test".to_owned()),
                outcome: Some("completed".to_owned()),
                duration_seconds: None,
                token_estimate: None,
                friction: Some("Query friction".to_owned()),
                notes: None,
                actions: CsvList::from_optional(Some("one,two".to_owned())),
                files_read: CsvList::from_optional(None),
                files_changed: CsvList::from_optional(None),
                decisions: CsvList::from_optional(None),
                errors: CsvList::from_optional(None),
            })
            .unwrap();
        assert_eq!(trace_id, 1);
        assert_eq!(
            repository.query_traces().unwrap()[0].task_summary,
            "Test trace"
        );
        assert_eq!(
            repository.query_friction().unwrap()[0].harness_friction,
            "Query friction"
        );
    }

    #[test]
    fn friction_query_includes_intake_context_and_filters_non_friction_values() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();
        let intake_id = repository
            .record_intake(IntakeInput {
                input_type: InputType::ChangeRequest,
                summary: "Friction query context".to_owned(),
                risk_lane: RiskLane::Normal,
                risk_flags: CsvList::from_optional(None),
                affected_docs: CsvList::from_optional(None),
                story_id: None,
                notes: None,
            })
            .unwrap();
        for (task_summary, friction) in [
            ("Trace without friction", None),
            ("Trace with empty friction", Some("")),
            ("Trace with whitespace friction", Some(" \t ")),
            ("Trace with none friction", Some("none")),
            ("Trace with normalized none friction", Some(" NONE ")),
        ] {
            repository
                .record_trace(TraceInput {
                    task_summary: task_summary.to_owned(),
                    intake_id: Some(intake_id),
                    story_id: None,
                    agent: Some("codex".to_owned()),
                    outcome: Some("completed".to_owned()),
                    duration_seconds: None,
                    token_estimate: None,
                    friction: friction.map(str::to_owned),
                    notes: None,
                    actions: CsvList::from_optional(None),
                    files_read: CsvList::from_optional(None),
                    files_changed: CsvList::from_optional(None),
                    decisions: CsvList::from_optional(None),
                    errors: CsvList::from_optional(None),
                })
                .unwrap();
        }
        repository
            .record_trace(TraceInput {
                task_summary: "Trace with linked friction".to_owned(),
                intake_id: Some(intake_id),
                story_id: None,
                agent: Some("codex".to_owned()),
                outcome: Some("completed".to_owned()),
                duration_seconds: None,
                token_estimate: None,
                friction: Some("Linked friction".to_owned()),
                notes: None,
                actions: CsvList::from_optional(None),
                files_read: CsvList::from_optional(None),
                files_changed: CsvList::from_optional(None),
                decisions: CsvList::from_optional(None),
                errors: CsvList::from_optional(None),
            })
            .unwrap();
        repository
            .record_trace(TraceInput {
                task_summary: "Trace with unlinked friction".to_owned(),
                intake_id: None,
                story_id: None,
                agent: Some("codex".to_owned()),
                outcome: Some("completed".to_owned()),
                duration_seconds: None,
                token_estimate: None,
                friction: Some("Unlinked friction".to_owned()),
                notes: None,
                actions: CsvList::from_optional(None),
                files_read: CsvList::from_optional(None),
                files_changed: CsvList::from_optional(None),
                decisions: CsvList::from_optional(None),
                errors: CsvList::from_optional(None),
            })
            .unwrap();

        let friction = repository.query_friction().unwrap();

        assert_eq!(friction.len(), 2);
        assert_eq!(friction[0].risk_lane, None);
        assert_eq!(friction[0].input_type, None);
        assert_eq!(friction[1].risk_lane.as_deref(), Some("normal"));
        assert_eq!(friction[1].input_type.as_deref(), Some("change_request"));
    }

    #[test]
    fn import_brownfield_seeds_markdown_state_idempotently() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_root = temp_dir.path().join("repo");
        fs::create_dir_all(repo_root.join("docs/decisions")).unwrap();
        fs::write(
            repo_root.join("docs/TEST_MATRIX.md"),
            r#"# Test Matrix

| Story | Contract | Unit | Integration | E2E | Platform | Status | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| US-010 | docs/product/tasks.md | yes | pending | no | mac smoke | implemented | cargo test |
"#,
        )
        .unwrap();
        fs::write(
            repo_root.join("docs/decisions/0007-test-decision.md"),
            r#"# Test Decision

## Status

Accepted
"#,
        )
        .unwrap();
        fs::write(
            repo_root.join("docs/HARNESS_BACKLOG.md"),
            r#"# Harness Backlog

## Items

### Title

Import existing docs

### Discovered While

Testing brownfield import

### Current Pain

Existing Harness v0 repos have markdown truth.

### Suggested Improvement

Seed the durable database.

### Risk

normal

### Status

accepted

### Title

Keep installer checksum

### Discovered While

Testing release install

### Current Pain

Downloads need verification.

### Suggested Improvement

Verify sha256 files.

### Risk

high-risk

### Status

implemented
"#,
        )
        .unwrap();

        let repository = SqliteHarnessRepository::new(
            repo_root.clone(),
            temp_dir.path().join("harness.db"),
            test_schema_dir(),
        );
        repository.init().unwrap();

        let first = repository.import_brownfield().unwrap();
        let second = repository.import_brownfield().unwrap();

        assert_eq!(
            first,
            BrownfieldImportResult {
                stories: 1,
                decisions: 1,
                backlog_items: 2,
            }
        );
        assert_eq!(second.backlog_items, 2);

        let matrix = repository.query_matrix().unwrap();
        assert_eq!(matrix[0].id, "US-010");
        assert_eq!(matrix[0].title, "docs/product/tasks.md");
        assert_eq!(matrix[0].status, "implemented");
        assert_eq!(matrix[0].unit, 1);
        assert_eq!(matrix[0].integration, 0);
        assert_eq!(matrix[0].platform, 1);

        let decisions = repository.query_decisions().unwrap();
        assert_eq!(decisions[0].id, "0007-test-decision");
        assert_eq!(decisions[0].status, "accepted");

        let backlog = repository.query_backlog(BacklogFilter::All).unwrap();
        assert_eq!(backlog.len(), 2);
        assert!(backlog
            .iter()
            .any(|item| item.title == "Import existing docs"
                && item.status == "accepted"
                && item.risk.as_deref() == Some("normal")));
        assert!(backlog
            .iter()
            .any(|item| item.title == "Keep installer checksum"
                && item.status == "implemented"
                && item.risk.as_deref() == Some("high_risk")));
    }

    #[test]
    fn filters_open_and_closed_backlog_items() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();

        let proposed_id = repository
            .add_backlog(BacklogAddInput {
                title: "Proposed item".to_owned(),
                discovered_while: None,
                current_pain: None,
                suggestion: None,
                risk: Some(RiskLane::Tiny),
                predicted_impact: Some("Should improve trace review.".to_owned()),
                notes: None,
            })
            .unwrap();
        let implemented_id = repository
            .add_backlog(BacklogAddInput {
                title: "Implemented item".to_owned(),
                discovered_while: None,
                current_pain: None,
                suggestion: None,
                risk: Some(RiskLane::Normal),
                predicted_impact: Some("Should reduce missing proof.".to_owned()),
                notes: None,
            })
            .unwrap();
        repository
            .close_backlog(BacklogCloseInput {
                id: implemented_id,
                status: "implemented".to_owned(),
                actual_outcome: Some("Proof gaps were found earlier.".to_owned()),
            })
            .unwrap();

        let all = repository.query_backlog(BacklogFilter::All).unwrap();
        let open = repository.query_backlog(BacklogFilter::Open).unwrap();
        let closed = repository.query_backlog(BacklogFilter::Closed).unwrap();

        assert_eq!(all.len(), 2);
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].id, proposed_id);
        assert_eq!(closed.len(), 1);
        assert_eq!(closed[0].id, implemented_id);
        assert_eq!(
            closed[0].actual_outcome.as_deref(),
            Some("Proof gaps were found earlier.")
        );
    }

    #[test]
    fn scores_latest_and_specific_trace_with_lane_lookup() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();
        let intake_id = repository
            .record_intake(IntakeInput {
                input_type: InputType::HarnessImprovement,
                summary: "High risk trace quality test".to_owned(),
                risk_lane: RiskLane::HighRisk,
                risk_flags: CsvList::from_optional(None),
                affected_docs: CsvList::from_optional(None),
                story_id: None,
                notes: None,
            })
            .unwrap();
        let first_trace = repository
            .record_trace(TraceInput {
                task_summary: "Minimal trace test".to_owned(),
                intake_id: None,
                story_id: None,
                agent: None,
                outcome: Some("completed".to_owned()),
                duration_seconds: None,
                token_estimate: None,
                friction: None,
                notes: None,
                actions: CsvList::from_optional(None),
                files_read: CsvList::from_optional(None),
                files_changed: CsvList::from_optional(None),
                decisions: CsvList::from_optional(None),
                errors: CsvList::from_optional(None),
            })
            .unwrap();
        repository
            .record_trace(TraceInput {
                task_summary: "Standard trace linked to high risk intake".to_owned(),
                intake_id: Some(intake_id),
                story_id: None,
                agent: Some("codex".to_owned()),
                outcome: Some("completed".to_owned()),
                duration_seconds: None,
                token_estimate: None,
                friction: Some("none".to_owned()),
                notes: None,
                actions: CsvList::from_optional(Some("read,patched".to_owned())),
                files_read: CsvList::from_optional(Some("PHASE3.md".to_owned())),
                files_changed: CsvList::from_optional(Some(
                    "crates/harness-cli/src/domain.rs".to_owned(),
                )),
                decisions: CsvList::from_optional(None),
                errors: CsvList::from_optional(None),
            })
            .unwrap();

        let latest = repository.score_trace(None).unwrap();
        assert_eq!(latest.achieved, TraceQualityTier::Standard);
        assert_eq!(latest.required, Some(TraceQualityTier::Detailed));
        assert!(!latest.meets_requirement);
        assert!(latest
            .missing_detailed
            .iter()
            .any(|field| field.starts_with("decisions_made")));

        let specific = repository.score_trace(Some(first_trace)).unwrap();
        assert_eq!(specific.trace_id, first_trace);
        assert_eq!(specific.achieved, TraceQualityTier::Minimal);
        assert_eq!(specific.required, None);
        assert!(specific.meets_requirement);
    }
}
