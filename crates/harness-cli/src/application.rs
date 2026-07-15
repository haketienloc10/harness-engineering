use std::path::PathBuf;

use crate::domain::{
    AuditDispositionRecord, AuditResult, BacklogFilter, BacklogRecord, BoolFlag,
    ContextScoreResult, CsvList, DecisionRecord, FrictionRecord, HarnessStats, ImprovementProposal,
    InputType, IntakeRecord, InterventionRecord, RiskLane, StoryMatrixRecord, StoryVerifyAllResult,
    StoryVerifyStatus, ToolArgSpec, ToolEntry, TraceRecord, TraceScoreResult,
};
use crate::infrastructure::{
    DoctorReport, HarnessRepository, SqliteHarnessRepository, ToolCheckResult, WorkflowPolicy,
};

#[derive(Debug)]
pub struct HarnessContext {
    pub repo_root: PathBuf,
    pub db_path: PathBuf,
    pub schema_dir: PathBuf,
}

#[derive(Debug)]
pub struct IntakeInput {
    pub input_type: InputType,
    pub summary: String,
    pub risk_lane: RiskLane,
    pub risk_flags: CsvList,
    pub affected_docs: CsvList,
    pub story_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug)]
pub struct FrictionAddInput {
    pub task_id: Option<String>,
    pub category: String,
    pub severity: String,
    pub summary: String,
    pub disposition: String,
    pub baseline: Option<String>,
    pub predicted_metric: Option<String>,
    pub observation_window: Option<String>,
}

#[derive(Debug)]
pub struct FrictionResolveInput {
    pub fingerprint: String,
    pub status: String,
    pub actual_outcome: String,
}

#[derive(Debug)]
pub struct AuditDispositionAddInput {
    pub finding_key: String,
    pub entity_id: String,
    pub rationale: String,
    pub provenance: String,
    pub approval_task_id: String,
    pub approval_source: String,
    pub actor: String,
    pub expires_at: Option<String>,
}

#[derive(Debug)]
pub struct AuditDispositionRevokeInput {
    pub id: i64,
    pub actor: String,
    pub reason: String,
}

#[derive(Debug)]
pub struct TaskStartInput {
    pub input_type: InputType,
    pub summary: String,
    pub risk_lane: Option<RiskLane>,
    pub lane_override_reason: Option<String>,
    pub owner: Option<String>,
    pub session_id: Option<String>,
    pub lease_seconds: Option<i64>,
    pub story_id: Option<String>,
    pub behavior_bearing: bool,
    pub risk_flags: Vec<String>,
}

#[derive(Debug)]
pub struct TaskStatusRecord {
    pub id: String,
    pub status: String,
    pub risk_lane: String,
    pub input_type: String,
    pub summary: String,
    pub risk_flags: Vec<String>,
    pub behavior_bearing: bool,
    pub owner: Option<String>,
    pub session_id: Option<String>,
    pub worktree: String,
    pub lease_expires_at: Option<String>,
    pub lease_state: String,
    pub story_id: Option<String>,
    pub allowed_next: Vec<String>,
    pub context_required: usize,
    pub context_acknowledged: usize,
    pub context_acknowledged_paths: Vec<String>,
    pub context_manifest: serde_json::Value,
    pub approvals: usize,
    pub capsule_required: bool,
    pub capsule_path: Option<String>,
    pub capsule_checksum: Option<String>,
    pub capsule_omission_reason: Option<String>,
    pub proof_runs: usize,
    pub latest_proof_state: Option<String>,
    pub latest_proof_head_fresh: Option<bool>,
    pub latest_proof_branch_fresh: Option<bool>,
    pub latest_proof_dirty_fresh: Option<bool>,
    pub latest_proof_output_fresh: Option<bool>,
    pub latest_proof_artifact_fresh: Option<bool>,
}

#[derive(Debug)]
pub struct TaskTransitionInput {
    pub id: String,
    pub status: String,
    pub outcome: Option<String>,
    pub owner: Option<String>,
    pub session_id: Option<String>,
    pub lease_seconds: Option<i64>,
}

#[derive(Debug)]
pub struct TaskHandoffInput {
    pub id: String,
    pub from_owner: String,
    pub from_session: String,
    pub to_owner: String,
    pub to_session: String,
    pub lease_seconds: Option<i64>,
    pub source: String,
    pub evidence: String,
    pub scope: Option<String>,
}

#[derive(Debug)]
pub struct TaskStoryLinkInput {
    pub id: String,
    pub story_id: String,
    pub role: String,
    pub owner: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug)]
pub struct TaskFinishInput {
    pub id: String,
    pub owner: Option<String>,
    pub session_id: Option<String>,
    pub trace_id: Option<i64>,
    pub friction: String,
    pub capsule_path: Option<String>,
}

#[derive(Debug)]
pub struct TaskFinishRecord {
    pub id: String,
    pub status: String,
    pub trace_id: i64,
}

#[derive(Debug)]
pub struct TaskRefreshInput {
    pub id: String,
    pub accept: bool,
}

#[derive(Debug)]
pub struct TaskRefreshRecord {
    pub id: String,
    pub changed: bool,
    pub applied: bool,
    pub previous_checksum: String,
    pub current_checksum: String,
    pub changed_paths: Vec<String>,
}

#[derive(Debug)]
pub struct TaskContextAcknowledgeInput {
    pub id: String,
    pub path: String,
    pub actor: Option<String>,
}

#[derive(Debug)]
pub struct TaskApprovalInput {
    pub id: String,
    pub gate: String,
    pub source: String,
    pub evidence: String,
    pub scope: Option<String>,
}

#[derive(Debug)]
pub struct ProofRunInput {
    pub task_id: String,
    pub story_id: Option<String>,
    pub layer: String,
    pub executable: String,
    pub argv: Vec<String>,
    pub artifact_path: Option<String>,
}

#[derive(Debug)]
pub struct ProofRunRecord {
    pub task_id: String,
    pub layer: String,
    pub state: String,
    pub exit_code: i32,
    pub head_commit: Option<String>,
    pub branch: Option<String>,
    pub stdout_path: String,
    pub stdout_hash: String,
    pub stderr_path: String,
    pub stderr_hash: String,
    pub artifact_path: Option<String>,
    pub artifact_hash: Option<String>,
}

#[derive(Debug)]
pub struct ProofRecord {
    pub story_id: Option<String>,
    pub layer: String,
    pub state: String,
    pub executable: Option<String>,
    pub argv_json: Option<String>,
    pub exit_code: Option<i32>,
    pub head_commit: Option<String>,
    pub branch: Option<String>,
    pub dirty_fingerprint: Option<String>,
    pub cli_version: Option<String>,
    pub platform: Option<String>,
    pub command_digest: Option<String>,
    pub stdout_path: Option<String>,
    pub stdout_hash: Option<String>,
    pub stderr_path: Option<String>,
    pub stderr_hash: Option<String>,
    pub artifact_path: Option<String>,
    pub artifact_hash: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug)]
pub struct StoryAddInput {
    pub id: String,
    pub title: String,
    pub risk_lane: RiskLane,
    pub contract_doc: Option<String>,
    pub verify_command: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug)]
pub struct StoryUpdateInput {
    pub id: String,
    pub status: Option<String>,
    pub evidence: Option<String>,
    pub unit: Option<BoolFlag>,
    pub integration: Option<BoolFlag>,
    pub e2e: Option<BoolFlag>,
    pub platform: Option<BoolFlag>,
    pub verify_command: Option<String>,
}

#[derive(Debug)]
pub struct DecisionAddInput {
    pub id: String,
    pub title: String,
    pub status: String,
    pub doc_path: Option<String>,
    pub verify_command: Option<String>,
    pub predicted_impact: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug)]
pub struct BacklogAddInput {
    pub title: String,
    pub discovered_while: Option<String>,
    pub current_pain: Option<String>,
    pub suggestion: Option<String>,
    pub risk: Option<RiskLane>,
    pub predicted_impact: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug)]
pub struct ToolRegisterInput {
    pub name: String,
    pub command: String,
    pub description: String,
    pub responsibility: String,
    pub args: Vec<ToolArgSpec>,
    pub force: bool,
    pub kind: String,
    pub capability: Option<String>,
    pub scan_target: Option<String>,
}

#[derive(Debug)]
pub struct InterventionAddInput {
    pub trace_id: Option<i64>,
    pub story_id: Option<String>,
    pub intervention_type: String,
    pub description: String,
    pub source: String,
    pub impact: Option<String>,
}

#[derive(Debug, Default)]
pub struct InterventionFilter {
    pub trace_id: Option<i64>,
    pub story_id: Option<String>,
    pub intervention_type: Option<String>,
}

#[derive(Debug)]
pub struct BacklogCloseInput {
    pub id: i64,
    pub status: String,
    pub actual_outcome: Option<String>,
}

#[derive(Debug)]
pub struct TraceInput {
    pub task_summary: String,
    pub intake_id: Option<i64>,
    pub story_id: Option<String>,
    pub agent: Option<String>,
    pub outcome: Option<String>,
    pub duration_seconds: Option<i64>,
    pub token_estimate: Option<i64>,
    pub friction: Option<String>,
    pub notes: Option<String>,
    pub actions: CsvList,
    pub files_read: CsvList,
    pub files_changed: CsvList,
    pub decisions: CsvList,
    pub errors: CsvList,
}

pub struct HarnessService {
    repository: SqliteHarnessRepository,
}

impl HarnessService {
    pub fn new(context: HarnessContext) -> Self {
        Self {
            repository: SqliteHarnessRepository::new(
                context.repo_root,
                context.db_path,
                context.schema_dir,
            ),
        }
    }

    pub fn init(&self) -> crate::infrastructure::Result<InitResult> {
        self.repository.init()
    }

    pub fn migrate(&self) -> crate::infrastructure::Result<MigrateResult> {
        self.repository.migrate()
    }

    pub fn doctor(&self) -> crate::infrastructure::Result<DoctorReport> {
        self.preflight()
    }

    /// Shared, read-only health boundary for commands that are allowed to
    /// operate on durable state. CL-11 wires this into safe ensure/migration;
    /// doctor exposes the same result without changing state.
    pub fn preflight(&self) -> crate::infrastructure::Result<DoctorReport> {
        self.repository.doctor()
    }

    pub fn workflow_policy(&self) -> crate::infrastructure::Result<WorkflowPolicy> {
        self.repository.workflow_policy()
    }

    pub fn import_brownfield(&self) -> crate::infrastructure::Result<BrownfieldImportResult> {
        self.repository.import_brownfield()
    }

    pub fn record_intake(&self, input: IntakeInput) -> crate::infrastructure::Result<i64> {
        self.repository.record_intake(input)
    }

    pub fn start_task(&self, input: TaskStartInput) -> crate::infrastructure::Result<String> {
        self.repository.start_task(input)
    }

    pub fn task_status(&self, id: &str) -> crate::infrastructure::Result<TaskStatusRecord> {
        self.repository.task_status(id)
    }

    pub fn transition_task(
        &self,
        input: TaskTransitionInput,
    ) -> crate::infrastructure::Result<TaskStatusRecord> {
        self.repository.transition_task(input)
    }

    pub fn handoff_task(&self, input: TaskHandoffInput) -> crate::infrastructure::Result<()> {
        self.repository.handoff_task(input)
    }

    pub fn link_task_story(&self, input: TaskStoryLinkInput) -> crate::infrastructure::Result<()> {
        self.repository.link_task_story(input)
    }

    pub fn finish_task(
        &self,
        input: TaskFinishInput,
    ) -> crate::infrastructure::Result<TaskFinishRecord> {
        self.repository.finish_task(input)
    }

    pub fn refresh_task(
        &self,
        input: TaskRefreshInput,
    ) -> crate::infrastructure::Result<TaskRefreshRecord> {
        self.repository.refresh_task(input)
    }

    pub fn acknowledge_task_context(
        &self,
        input: TaskContextAcknowledgeInput,
    ) -> crate::infrastructure::Result<()> {
        self.repository.acknowledge_task_context(input)
    }

    pub fn approve_task(&self, input: TaskApprovalInput) -> crate::infrastructure::Result<()> {
        self.repository.approve_task(input)
    }

    pub fn run_proof(&self, input: ProofRunInput) -> crate::infrastructure::Result<ProofRunRecord> {
        self.repository.run_proof(input)
    }

    pub fn query_proofs(&self, task_id: &str) -> crate::infrastructure::Result<Vec<ProofRecord>> {
        self.repository.query_proofs(task_id)
    }

    pub fn add_story(&self, input: StoryAddInput) -> crate::infrastructure::Result<()> {
        self.repository.add_story(input)
    }

    pub fn update_story(&self, input: StoryUpdateInput) -> crate::infrastructure::Result<()> {
        self.repository.update_story(input)
    }

    pub fn verify_story(&self, id: &str) -> crate::infrastructure::Result<StoryVerifyResult> {
        self.repository.verify_story(id)
    }

    pub fn verify_all_stories(&self) -> crate::infrastructure::Result<StoryVerifyAllResult> {
        self.repository.verify_all_stories()
    }

    pub fn add_decision(&self, input: DecisionAddInput) -> crate::infrastructure::Result<()> {
        self.repository.add_decision(input)
    }

    pub fn verify_decision(&self, id: &str) -> crate::infrastructure::Result<DecisionVerifyResult> {
        self.repository.verify_decision(id)
    }

    pub fn add_backlog(&self, input: BacklogAddInput) -> crate::infrastructure::Result<i64> {
        self.repository.add_backlog(input)
    }

    pub fn close_backlog(&self, input: BacklogCloseInput) -> crate::infrastructure::Result<()> {
        self.repository.close_backlog(input)
    }

    pub fn register_tool(&self, input: ToolRegisterInput) -> crate::infrastructure::Result<()> {
        self.repository.register_tool(input)
    }

    pub fn remove_tool(&self, name: &str) -> crate::infrastructure::Result<()> {
        self.repository.remove_tool(name)
    }

    pub fn check_tools(
        &self,
        name: Option<String>,
    ) -> crate::infrastructure::Result<Vec<ToolCheckResult>> {
        self.repository.check_tools(name)
    }

    pub fn add_intervention(
        &self,
        input: InterventionAddInput,
    ) -> crate::infrastructure::Result<i64> {
        self.repository.add_intervention(input)
    }

    pub fn record_trace(&self, input: TraceInput) -> crate::infrastructure::Result<i64> {
        self.repository.record_trace(input)
    }

    pub fn score_trace(&self, id: Option<i64>) -> crate::infrastructure::Result<TraceScoreResult> {
        self.repository.score_trace(id)
    }

    pub fn score_context(&self, id: i64) -> crate::infrastructure::Result<ContextScoreResult> {
        self.repository.score_context(id)
    }

    pub fn story_verify_status(
        &self,
        id: &str,
    ) -> crate::infrastructure::Result<StoryVerifyStatus> {
        self.repository.story_verify_status(id)
    }

    pub fn query_matrix(&self) -> crate::infrastructure::Result<Vec<StoryMatrixRecord>> {
        self.repository.query_matrix()
    }

    pub fn query_backlog(
        &self,
        filter: BacklogFilter,
    ) -> crate::infrastructure::Result<Vec<BacklogRecord>> {
        self.repository.query_backlog(filter)
    }

    pub fn query_decisions(&self) -> crate::infrastructure::Result<Vec<DecisionRecord>> {
        self.repository.query_decisions()
    }

    pub fn query_intakes(&self) -> crate::infrastructure::Result<Vec<IntakeRecord>> {
        self.repository.query_intakes()
    }

    pub fn query_traces(&self) -> crate::infrastructure::Result<Vec<TraceRecord>> {
        self.repository.query_traces()
    }

    pub fn query_friction(&self) -> crate::infrastructure::Result<Vec<FrictionRecord>> {
        self.repository.query_friction()
    }

    pub fn add_friction(&self, input: FrictionAddInput) -> crate::infrastructure::Result<String> {
        self.repository.add_friction(input)
    }
    pub fn resolve_friction(
        &self,
        input: FrictionResolveInput,
    ) -> crate::infrastructure::Result<()> {
        self.repository.resolve_friction(input)
    }

    pub fn query_tools(
        &self,
        responsibility: Option<String>,
        capability: Option<String>,
    ) -> crate::infrastructure::Result<Vec<ToolEntry>> {
        self.repository.query_tools(responsibility, capability)
    }

    pub fn query_interventions(
        &self,
        filter: InterventionFilter,
    ) -> crate::infrastructure::Result<Vec<InterventionRecord>> {
        self.repository.query_interventions(filter)
    }

    pub fn query_stats(&self) -> crate::infrastructure::Result<HarnessStats> {
        self.repository.query_stats()
    }

    pub fn audit(&self) -> crate::infrastructure::Result<AuditResult> {
        self.repository.audit()
    }

    pub fn add_audit_disposition(
        &self,
        input: AuditDispositionAddInput,
    ) -> crate::infrastructure::Result<i64> {
        self.repository.add_audit_disposition(input)
    }

    pub fn list_audit_dispositions(
        &self,
    ) -> crate::infrastructure::Result<Vec<AuditDispositionRecord>> {
        self.repository.list_audit_dispositions()
    }

    pub fn revoke_audit_disposition(
        &self,
        input: AuditDispositionRevokeInput,
    ) -> crate::infrastructure::Result<()> {
        self.repository.revoke_audit_disposition(input)
    }

    pub fn propose(&self, commit: bool) -> crate::infrastructure::Result<Vec<ImprovementProposal>> {
        self.repository.propose(commit)
    }

    pub fn query_sql(&self, sql: &str) -> crate::infrastructure::Result<QueryTable> {
        self.repository.query_sql(sql)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum InitResult {
    Created { db_path: PathBuf },
    Existing { db_path: PathBuf, version: i64 },
    MigratedExisting { db_path: PathBuf },
}

#[derive(Debug, PartialEq, Eq)]
pub struct MigrateResult {
    pub current_version: i64,
    pub applied: Vec<i64>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct BrownfieldImportResult {
    pub stories: usize,
    pub decisions: usize,
    pub backlog_items: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DecisionVerifyResult {
    pub command: String,
    pub result: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StoryVerifyResult {
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub result: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct QueryTable {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}
