use std::env;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Args, CommandFactory, Parser, Subcommand};
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::application::{
    AuditDispositionAddInput, AuditDispositionRevokeInput, BacklogAddInput, BacklogCloseInput,
    BrownfieldImportResult, DecisionAddInput, FrictionAddInput, FrictionResolveInput,
    HarnessContext, HarnessService, InitResult, IntakeInput, InterventionAddInput,
    InterventionFilter, MigrateResult, ProofRecord, ProofRunInput, QueryTable, StoryAddInput,
    StoryUpdateInput, TaskApprovalInput, TaskContextAcknowledgeInput, TaskFinishInput,
    TaskHandoffInput, TaskRefreshInput, TaskStartInput, TaskStoryLinkInput, TaskTransitionInput,
    ToolRegisterInput, TraceInput,
};
use crate::domain::{
    normalize_capability, normalize_token, parse_optional_integer, parse_tool_args, proof_display,
    validate_responsibility, validate_tool_kind, AuditDispositionRecord, BacklogFilter,
    BacklogRecord, BoolFlag, ContextScoreResult, CsvList, DecisionRecord, FrictionRecord,
    HarnessStats, ImprovementProposal, InputType, IntakeRecord, InterventionRecord, RiskLane,
    StoryMatrixRecord, StoryVerifyAllResult, StructuredErrorResult, ToolEntry, TraceQualityTier,
    TraceRecord, TraceScoreResult, RISK_LANE_HELP,
};
use crate::infrastructure::ToolCheckResult;

#[derive(Parser, Debug)]
#[command(name = "harness-cli")]
#[command(bin_name = "_harness/bin/harness-cli")]
#[command(about = "durable layer for the project harness", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Inspect repository and database health without changing state.
    Doctor(DoctorArgs),
    /// Validate or explain the typed Harness workflow policy.
    Workflow(WorkflowArgs),
    /// Create the harness database if it does not already exist.
    Init,
    /// Apply schema migrations.
    Migrate,
    /// Seed or refresh the database from existing markdown state.
    Import(ImportArgs),
    /// Record a feature intake classification.
    Intake(IntakeArgs),
    /// Start or inspect command-first lifecycle tasks.
    Task(TaskArgs),
    /// Run a structured proof command for a lifecycle task.
    Proof(ProofArgs),
    /// Record or resolve structured harness friction.
    Friction(FrictionArgs),
    /// Validate or inspect Git-tracked semantic artifacts without writing state.
    Memory(MemoryArgs),
    /// Add or update a story.
    Story(StoryArgs),
    /// Add a decision or run its verification.
    Decision(DecisionArgs),
    /// Add or close a backlog item.
    Backlog(BacklogArgs),
    /// Register or remove external tools.
    Tool(ToolArgs),
    /// Record a human, review, CI, or agent intervention.
    Intervention(InterventionArgs),
    /// Record an agent execution trace.
    Trace(TraceArgs),
    /// Score a trace against the trace quality tiers.
    ScoreTrace(ScoreTraceArgs),
    /// Score trace context reads against CONTEXT_RULES.md.
    ScoreContext { trace_id: String },
    /// Run drift audit and entropy score.
    Audit(AuditArgs),
    /// Generate improvement proposals from observed patterns.
    Propose(ProposeArgs),
    /// Query harness data.
    Query(QueryArgs),
}

#[derive(Args, Debug)]
struct DoctorArgs {
    #[arg(long)]
    json: bool,
    /// Treat pending or missing durable state as a failure.
    #[arg(long)]
    strict: bool,
}

#[derive(Args, Debug)]
struct AuditArgs {
    #[command(subcommand)]
    action: Option<AuditAction>,
    /// Exit with consistency-failure status when audit debt or unknown coverage remains.
    #[arg(long)]
    strict: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum AuditAction {
    /// Add, list, or revoke explicit historical audit dispositions.
    Disposition(AuditDispositionArgs),
}

#[derive(Args, Debug)]
struct AuditDispositionArgs {
    #[command(subcommand)]
    action: AuditDispositionAction,
}

#[derive(Subcommand, Debug)]
enum AuditDispositionAction {
    Add(AuditDispositionAddArgs),
    List {
        #[arg(long)]
        json: bool,
    },
    Revoke(AuditDispositionRevokeArgs),
}

#[derive(Args, Debug)]
struct AuditDispositionAddArgs {
    #[arg(long)]
    finding_key: String,
    #[arg(long)]
    entity_id: String,
    #[arg(long)]
    rationale: String,
    #[arg(long)]
    provenance: String,
    #[arg(long)]
    approval_task: String,
    #[arg(long)]
    approval_source: String,
    #[arg(long)]
    actor: String,
    #[arg(long)]
    expires_at: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct AuditDispositionRevokeArgs {
    #[arg(long)]
    id: String,
    #[arg(long)]
    actor: String,
    #[arg(long)]
    reason: String,
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct WorkflowArgs {
    #[command(subcommand)]
    action: WorkflowAction,
}

#[derive(Subcommand, Debug)]
enum WorkflowAction {
    Validate {
        #[arg(long)]
        json: bool,
    },
    Explain {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        flags: Option<String>,
    },
    Context {
        #[arg(long)]
        paths: Option<String>,
        #[arg(long, default_value = "tiny", value_name = "tiny|normal|high-risk")]
        lane: String,
        #[arg(
            long,
            default_value = "work",
            value_name = "intake|planning|work|finish"
        )]
        phase: String,
        #[arg(long)]
        flags: Option<String>,
        #[arg(long)]
        linked_artifacts: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Render the command tree compiled from the Clap definition.
    Commands {
        #[arg(long)]
        json: bool,
    },
    /// Check the shadow policy against its tracked parity fixture and payload contracts.
    Parity {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyParityFixture {
    schema_version: u32,
    source_policy: String,
    #[serde(default)]
    classification_cases: Vec<PolicyParityClassificationCase>,
    #[serde(default)]
    context_cases: Vec<PolicyParityContextCase>,
    #[serde(default)]
    intentional_deltas: Vec<PolicyParityDelta>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyParityClassificationCase {
    id: String,
    #[serde(default)]
    flags: Vec<String>,
    expected_lane: String,
    comparison: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyParityContextCase {
    id: String,
    lane: String,
    phase: String,
    #[serde(default)]
    paths: Vec<String>,
    #[serde(default)]
    flags: Vec<String>,
    #[serde(default)]
    must_include: Vec<String>,
    #[serde(default)]
    should_include: Vec<String>,
    #[serde(default)]
    skip_include: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyParityDelta {
    id: String,
    current_markdown: String,
    shadow_behavior: String,
    disposition: String,
    decision: String,
}

#[derive(Debug, Default)]
struct PolicyParityResult {
    checked: Vec<String>,
    deltas: Vec<String>,
    failures: Vec<String>,
}

pub fn compiled_command_manifest() -> Vec<String> {
    fn collect(prefix: &str, command: &clap::Command, output: &mut Vec<String>) {
        for child in command
            .get_subcommands()
            .filter(|child| child.get_name() != "help")
        {
            let path = if prefix.is_empty() {
                child.get_name().to_owned()
            } else {
                format!("{prefix} {}", child.get_name())
            };
            output.push(path.clone());
            collect(&path, child, output);
        }
    }

    let command = Cli::command();
    let mut output = Vec::new();
    collect("", &command, &mut output);
    output
}

#[derive(Args, Debug)]
#[command(after_help = RISK_LANE_HELP)]
struct IntakeArgs {
    #[arg(long = "type")]
    input_type: String,
    #[arg(long)]
    summary: String,
    #[arg(long, value_name = "tiny|normal|high-risk")]
    lane: String,
    #[arg(long)]
    flags: Option<String>,
    #[arg(long)]
    docs: Option<String>,
    #[arg(long)]
    story: Option<String>,
    #[arg(long)]
    notes: Option<String>,
}

#[derive(Args, Debug)]
struct TaskArgs {
    #[command(subcommand)]
    action: TaskAction,
}

#[derive(Args, Debug)]
struct ProofArgs {
    #[command(subcommand)]
    action: ProofAction,
}

#[derive(Args, Debug)]
struct FrictionArgs {
    #[command(subcommand)]
    action: FrictionAction,
}
#[derive(Subcommand, Debug)]
enum FrictionAction {
    Add {
        #[arg(long)]
        task: Option<String>,
        #[arg(long)]
        category: String,
        #[arg(long)]
        severity: String,
        #[arg(long)]
        summary: String,
        #[arg(long)]
        disposition: String,
        #[arg(long)]
        baseline: Option<String>,
        #[arg(long = "predicted-metric")]
        predicted_metric: Option<String>,
        #[arg(long = "observation-window")]
        observation_window: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Resolve {
        #[arg(long)]
        fingerprint: String,
        #[arg(long)]
        status: String,
        #[arg(long = "actual-outcome")]
        actual_outcome: String,
        #[arg(long)]
        json: bool,
    },
    Query,
}

#[derive(Subcommand, Debug)]
enum ProofAction {
    Run(ProofRunArgs),
    Query {
        #[arg(long = "task")]
        task_id: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Args, Debug)]
struct ProofRunArgs {
    #[arg(long = "task")]
    task_id: String,
    #[arg(long)]
    story: Option<String>,
    #[arg(long)]
    layer: String,
    #[arg(long, value_name = "REPO_RELATIVE_PATH")]
    artifact: Option<String>,
    #[arg(last = true, required = true)]
    command: Vec<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum TaskAction {
    Start(TaskStartArgs),
    Block(TaskBlockArgs),
    Resume(TaskResumeArgs),
    Abandon(TaskAbandonArgs),
    Handoff(TaskHandoffArgs),
    LinkStory(TaskLinkStoryArgs),
    Finish(TaskFinishArgs),
    Refresh(TaskRefreshArgs),
    Context(TaskContextArgs),
    Approve(TaskApproveArgs),
    Status {
        #[arg(long)]
        id: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Args, Debug)]
struct TaskStartArgs {
    #[arg(long = "type")]
    input_type: String,
    #[arg(long)]
    summary: String,
    #[arg(long, value_name = "tiny|normal|high-risk")]
    lane: Option<String>,
    #[arg(long)]
    lane_reason: Option<String>,
    #[arg(long)]
    owner: Option<String>,
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    lease_seconds: Option<i64>,
    #[arg(long)]
    story: Option<String>,
    #[arg(long)]
    flags: Option<String>,
    #[arg(long, value_name = "auto|yes|no", default_value = "auto")]
    behavior_bearing: String,
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct TaskContextArgs {
    #[command(subcommand)]
    action: TaskContextAction,
}

#[derive(Subcommand, Debug)]
enum TaskContextAction {
    Acknowledge {
        #[arg(long)]
        id: String,
        #[arg(long = "read")]
        path: String,
        #[arg(long)]
        actor: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Args, Debug)]
struct TaskBlockArgs {
    #[arg(long)]
    id: String,
    #[arg(long)]
    owner: Option<String>,
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct TaskResumeArgs {
    #[arg(long)]
    id: String,
    #[arg(long)]
    owner: Option<String>,
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    lease_seconds: Option<i64>,
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct TaskAbandonArgs {
    #[arg(long)]
    id: String,
    #[arg(long)]
    owner: Option<String>,
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    outcome: String,
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct TaskHandoffArgs {
    #[arg(long)]
    id: String,
    #[arg(long = "from")]
    from_owner: String,
    #[arg(long = "from-session")]
    from_session: String,
    #[arg(long = "to")]
    to_owner: String,
    #[arg(long = "to-session")]
    to_session: String,
    #[arg(long)]
    lease_seconds: Option<i64>,
    #[arg(long)]
    source: String,
    #[arg(long)]
    evidence: String,
    #[arg(long)]
    scope: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct TaskLinkStoryArgs {
    #[arg(long)]
    id: String,
    #[arg(long)]
    story: String,
    #[arg(long, value_name = "primary|secondary")]
    role: String,
    #[arg(long)]
    owner: Option<String>,
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct TaskFinishArgs {
    #[arg(long)]
    id: String,
    #[arg(long)]
    owner: Option<String>,
    #[arg(long)]
    session: Option<String>,
    #[arg(long)]
    trace: Option<String>,
    #[arg(long)]
    outcome: String,
    #[arg(long)]
    friction: String,
    #[arg(long)]
    capsule: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct TaskRefreshArgs {
    #[arg(long)]
    id: String,
    #[arg(long)]
    accept: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct TaskApproveArgs {
    #[arg(long)]
    id: String,
    #[arg(long)]
    gate: String,
    #[arg(long)]
    source: String,
    #[arg(long)]
    evidence: String,
    #[arg(long)]
    scope: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct ImportArgs {
    #[command(subcommand)]
    source: ImportSource,
}

#[derive(Subcommand, Debug)]
enum ImportSource {
    /// Import TEST_MATRIX, decisions, and backlog markdown.
    Brownfield,
}

#[derive(Args, Debug)]
struct StoryArgs {
    #[command(subcommand)]
    action: StoryAction,
}

#[derive(Subcommand, Debug)]
enum StoryAction {
    #[command(after_help = RISK_LANE_HELP)]
    Add(StoryAddArgs),
    #[command(
        after_help = "Proof flags use numeric booleans: --unit 1 --integration 1 --e2e 0 --platform 0. Do not use yes/no."
    )]
    Update(StoryUpdateArgs),
    #[command(
        after_help = "story verify only accepts the story id. Configure proof with story add/update --verify, then record proof flags with story update."
    )]
    Verify {
        /// Story id to verify.
        id: String,
    },
    /// Verify every story, skipping stories without verify_command.
    VerifyAll,
    /// Validate a tracked legacy or v1 story artifact without writing state.
    Check(ArtifactCheckArgs),
}

#[derive(Args, Debug)]
struct StoryAddArgs {
    #[arg(long)]
    id: String,
    #[arg(long)]
    title: String,
    #[arg(long, value_name = "tiny|normal|high-risk")]
    lane: String,
    #[arg(long)]
    contract: Option<String>,
    #[arg(long)]
    verify: Option<String>,
    #[arg(long)]
    notes: Option<String>,
}

#[derive(Args, Debug)]
struct StoryUpdateArgs {
    #[arg(long)]
    id: String,
    #[arg(long)]
    status: Option<String>,
    #[arg(long)]
    evidence: Option<String>,
    #[arg(long, value_name = "0|1")]
    unit: Option<String>,
    #[arg(long, value_name = "0|1")]
    integration: Option<String>,
    #[arg(long, value_name = "0|1")]
    e2e: Option<String>,
    #[arg(long, value_name = "0|1")]
    platform: Option<String>,
    #[arg(long)]
    verify: Option<String>,
}

#[derive(Args, Debug)]
struct DecisionArgs {
    #[command(subcommand)]
    action: DecisionAction,
}

#[derive(Subcommand, Debug)]
enum DecisionAction {
    Add(DecisionAddArgs),
    Verify {
        id: String,
    },
    /// Validate a tracked legacy or v1 decision artifact without writing state.
    Check(ArtifactCheckArgs),
}

#[derive(Args, Debug)]
struct MemoryArgs {
    #[command(subcommand)]
    action: MemoryAction,
}

#[derive(Subcommand, Debug)]
enum MemoryAction {
    /// Validate all canonical artifacts; this command is always read-only in CL-30.
    Check {
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    /// Validate artifacts and initialize an isolated temporary rebuild database.
    Rebuild {
        #[arg(long)]
        dry_run: bool,
        /// Replace the active DB only after a validated rebuild and backup.
        #[arg(long)]
        apply: bool,
        /// Replace a foreign/unhealthy active DB after validation and backup.
        #[arg(long, requires = "apply")]
        recover_foreign: bool,
        /// Keep the validated rebuilt database at this new repository-relative path.
        #[arg(long)]
        output: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Render or inspect portable task capsules.
    Capsule(CapsuleArgs),
}

#[derive(Args, Debug)]
struct CapsuleArgs {
    #[command(subcommand)]
    action: CapsuleAction,
}

#[derive(Subcommand, Debug)]
enum CapsuleAction {
    Render {
        #[arg(long)]
        id: String,
        #[arg(long)]
        date: String,
        #[arg(long, value_name = "tiny|normal|high-risk")]
        lane: String,
        #[arg(long)]
        outcome: String,
        #[arg(long)]
        summary: String,
        #[arg(long)]
        json: bool,
    },
    Check {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Args, Debug)]
struct ArtifactCheckArgs {
    /// Repository-relative path. Defaults to every artifact of this type.
    #[arg(long)]
    path: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct DecisionAddArgs {
    #[arg(long)]
    id: String,
    #[arg(long)]
    title: String,
    #[arg(long, default_value = "accepted")]
    status: String,
    #[arg(long)]
    doc: Option<String>,
    #[arg(long)]
    verify: Option<String>,
    #[arg(long)]
    predicted: Option<String>,
    #[arg(long)]
    notes: Option<String>,
}

#[derive(Args, Debug)]
struct BacklogArgs {
    #[command(subcommand)]
    action: BacklogAction,
}

#[derive(Subcommand, Debug)]
enum BacklogAction {
    #[command(after_help = RISK_LANE_HELP)]
    Add(BacklogAddArgs),
    Close(BacklogCloseArgs),
}

#[derive(Args, Debug)]
struct BacklogAddArgs {
    #[arg(long)]
    title: String,
    #[arg(long = "while")]
    discovered_while: Option<String>,
    #[arg(long)]
    pain: Option<String>,
    #[arg(long)]
    suggestion: Option<String>,
    #[arg(long, value_name = "tiny|normal|high-risk")]
    risk: Option<String>,
    #[arg(long)]
    predicted: Option<String>,
    #[arg(long)]
    notes: Option<String>,
}

#[derive(Args, Debug)]
struct BacklogCloseArgs {
    #[arg(long)]
    id: String,
    #[arg(long, default_value = "implemented")]
    status: String,
    #[arg(long)]
    outcome: Option<String>,
}

#[derive(Args, Debug)]
struct ToolArgs {
    #[command(subcommand)]
    action: ToolAction,
}

#[derive(Subcommand, Debug)]
enum ToolAction {
    Register(ToolRegisterArgs),
    /// Scan registered tools and persist present/missing/unknown status.
    Check(ToolCheckArgs),
    Remove {
        #[arg(long)]
        name: String,
    },
}

#[derive(Args, Debug)]
struct ToolRegisterArgs {
    #[arg(long)]
    name: String,
    #[arg(long)]
    command: String,
    #[arg(long)]
    description: String,
    #[arg(long)]
    responsibility: String,
    #[arg(long)]
    args: Option<String>,
    #[arg(long)]
    force: bool,
    /// How the tool is reached and probed: cli, binary, mcp, skill, http.
    #[arg(long, default_value = "cli")]
    kind: String,
    /// Workflow purpose a step looks the tool up by (kebab-case).
    #[arg(long)]
    capability: Option<String>,
    /// Declarative path/URL `tool check` resolves to decide presence.
    #[arg(long)]
    scan: Option<String>,
}

#[derive(Args, Debug)]
struct ToolCheckArgs {
    /// Check one tool by name; omit to check every registered tool.
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct InterventionArgs {
    #[command(subcommand)]
    action: InterventionAction,
}

#[derive(Subcommand, Debug)]
enum InterventionAction {
    Add(InterventionAddArgs),
}

#[derive(Args, Debug)]
struct InterventionAddArgs {
    #[arg(long)]
    trace: Option<String>,
    #[arg(long)]
    story: Option<String>,
    #[arg(long = "type")]
    intervention_type: String,
    #[arg(long)]
    description: String,
    #[arg(long)]
    source: String,
    #[arg(long)]
    impact: Option<String>,
}

#[derive(Args, Debug)]
struct TraceArgs {
    #[arg(long)]
    summary: String,
    #[arg(long)]
    intake: Option<String>,
    #[arg(long)]
    story: Option<String>,
    #[arg(long)]
    agent: Option<String>,
    #[arg(long)]
    outcome: Option<String>,
    #[arg(long)]
    duration: Option<String>,
    #[arg(long)]
    tokens: Option<String>,
    #[arg(long)]
    friction: Option<String>,
    #[arg(long)]
    actions: Option<String>,
    #[arg(long = "read")]
    files_read: Option<String>,
    #[arg(long = "changed")]
    files_changed: Option<String>,
    #[arg(long)]
    decisions: Option<String>,
    #[arg(long)]
    errors: Option<String>,
    #[arg(long)]
    notes: Option<String>,
}

#[derive(Args, Debug)]
struct ScoreTraceArgs {
    /// Score a specific trace id. Defaults to the latest trace.
    #[arg(long)]
    id: Option<String>,
}

#[derive(Args, Debug)]
struct ProposeArgs {
    #[arg(long)]
    commit: bool,
}

#[derive(Args, Debug)]
struct QueryArgs {
    #[command(subcommand)]
    view: QueryView,
}

#[derive(Args, Debug)]
struct MatrixQueryArgs {
    /// Render proof flags as CLI input values, 1 and 0, instead of yes and no.
    #[arg(long)]
    numeric: bool,
}

#[derive(Args, Debug)]
struct BacklogQueryArgs {
    /// Show only proposed and accepted backlog items.
    #[arg(long, conflicts_with = "closed")]
    open: bool,
    /// Show only implemented and rejected backlog items.
    #[arg(long)]
    closed: bool,
}

#[derive(Subcommand, Debug)]
enum QueryView {
    /// Test matrix.
    Matrix(MatrixQueryArgs),
    /// Harness improvement proposals.
    Backlog(BacklogQueryArgs),
    /// Decision records.
    Decisions,
    /// Recent intake classifications.
    Intakes,
    /// Recent traces.
    Traces,
    /// Traces with harness friction.
    Friction,
    /// Machine-readable and registered tool manifest.
    Tools(ToolsQueryArgs),
    /// Intervention records.
    Interventions(InterventionsQueryArgs),
    /// Summary counts.
    Stats,
    /// Run arbitrary SQL.
    Sql { query: Vec<String> },
}

#[derive(Args, Debug)]
struct ToolsQueryArgs {
    #[arg(long)]
    json: bool,
    #[arg(long)]
    summary: bool,
    #[arg(long)]
    responsibility: Option<String>,
    /// Filter to tools that provide this capability.
    #[arg(long)]
    capability: Option<String>,
    /// Filter to tools with this scanned status: present, missing, unknown.
    #[arg(long)]
    status: Option<String>,
}

#[derive(Args, Debug)]
struct InterventionsQueryArgs {
    #[arg(long)]
    trace: Option<String>,
    #[arg(long)]
    story: Option<String>,
    #[arg(long = "type")]
    intervention_type: Option<String>,
}

#[derive(Debug, Error)]
pub enum InterfaceError {
    #[error("{0}")]
    ParseHarnessValue(#[from] crate::domain::ParseHarnessValueError),
    #[error("{0}")]
    ToolValidation(#[from] crate::domain::ToolValidationError),
    #[error("{0}")]
    Infrastructure(#[from] crate::infrastructure::HarnessInfraError),
    #[error("could not determine current directory: {0}")]
    CurrentDir(std::io::Error),
    #[error("repository root error: {0}")]
    RepositoryRoot(String),
    #[error("query sql requires a SQL statement")]
    EmptySql,
    #[error("workflow parity: {0}")]
    WorkflowParity(String),
    #[error("{0}")]
    Usage(String),
}

impl Cli {
    pub fn requests_json(&self) -> bool {
        matches!(&self.command, Command::Audit(args) if audit_args_request_json(args))
            || matches!(
                &self.command,
                Command::Task(TaskArgs {
                action: TaskAction::Finish(args),
                }) if args.json
            )
    }
}

fn audit_args_request_json(args: &AuditArgs) -> bool {
    args.json
        || matches!(
            &args.action,
            Some(AuditAction::Disposition(AuditDispositionArgs {
                action: AuditDispositionAction::Add(AuditDispositionAddArgs { json: true, .. })
                    | AuditDispositionAction::List { json: true }
                    | AuditDispositionAction::Revoke(AuditDispositionRevokeArgs { json: true, .. })
            }))
        )
}

impl InterfaceError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Infrastructure(crate::infrastructure::HarnessInfraError::UnsafeDurableState(
                _,
            )) => 3,
            Self::Infrastructure(
                crate::infrastructure::HarnessInfraError::BackupFailed(_)
                | crate::infrastructure::HarnessInfraError::Sqlite(_),
            ) => 4,
            Self::Infrastructure(crate::infrastructure::HarnessInfraError::TaskFinishGate(
                result,
            )) if result.code == "TASK_APPROVAL_REQUIRED" => 9,
            Self::Infrastructure(crate::infrastructure::HarnessInfraError::TaskFinishGate(_)) => 5,
            Self::Infrastructure(crate::infrastructure::HarnessInfraError::TaskNotFound(_)) => 5,
            Self::Infrastructure(
                crate::infrastructure::HarnessInfraError::TaskIdentityPairRequired
                | crate::infrastructure::HarnessInfraError::InvalidTaskLeaseDuration,
            ) => 2,
            Self::ParseHarnessValue(_)
            | Self::ToolValidation(_)
            | Self::Usage(_)
            | Self::EmptySql
            | Self::Infrastructure(
                crate::infrastructure::HarnessInfraError::WorkflowInvalid(_)
                | crate::infrastructure::HarnessInfraError::UnknownApprovalGate(_)
                | crate::infrastructure::HarnessInfraError::TaskLaneOverrideReasonRequired
                | crate::infrastructure::HarnessInfraError::TaskLaneOverrideCannotLower { .. },
            ) => 2,
            Self::Infrastructure(
                crate::infrastructure::HarnessInfraError::TaskOwnerConflict { .. }
                | crate::infrastructure::HarnessInfraError::TaskOwnerMismatch { .. }
                | crate::infrastructure::HarnessInfraError::TaskOwnerRequired(_)
                | crate::infrastructure::HarnessInfraError::TaskSessionRequired(_)
                | crate::infrastructure::HarnessInfraError::TaskSessionMismatch { .. }
                | crate::infrastructure::HarnessInfraError::TaskLeaseExpired
                | crate::infrastructure::HarnessInfraError::TaskLeaseConflict { .. }
                | crate::infrastructure::HarnessInfraError::TaskHandoffSameOwner
                | crate::infrastructure::HarnessInfraError::TaskHandoffSameSession,
            ) => 8,
            Self::WorkflowParity(_) => 6,
            _ => 10,
        }
    }

    pub fn structured_result(&self) -> StructuredErrorResult {
        match self {
            Self::Infrastructure(crate::infrastructure::HarnessInfraError::TaskFinishGate(
                result,
            )) => result.clone(),
            Self::Infrastructure(crate::infrastructure::HarnessInfraError::TaskNotFound(id)) => {
                StructuredErrorResult::new(
                    "TASK_NOT_FOUND",
                    self.to_string(),
                    ["Use task status with the canonical task id before retrying."],
                )
                .with_detail("task_id", id)
            }
            Self::Infrastructure(
                crate::infrastructure::HarnessInfraError::TaskOwnerConflict { .. }
                | crate::infrastructure::HarnessInfraError::TaskOwnerMismatch { .. }
                | crate::infrastructure::HarnessInfraError::TaskOwnerRequired(_)
                | crate::infrastructure::HarnessInfraError::TaskSessionRequired(_)
                | crate::infrastructure::HarnessInfraError::TaskSessionMismatch { .. }
                | crate::infrastructure::HarnessInfraError::TaskLeaseExpired
                | crate::infrastructure::HarnessInfraError::TaskLeaseConflict { .. },
            ) => StructuredErrorResult::new(
                "TASK_OWNERSHIP_CONFLICT",
                self.to_string(),
                ["Use the matching owner/session, or record an explicit task handoff before retrying."],
            ),
            Self::Infrastructure(crate::infrastructure::HarnessInfraError::UnsafeDurableState(
                code,
            )) => StructuredErrorResult::new(
                code,
                self.to_string(),
                ["Run doctor --json and follow its remediation before retrying the lifecycle command."],
            ),
            Self::Infrastructure(crate::infrastructure::HarnessInfraError::Sqlite(_)) => {
                StructuredErrorResult::new(
                    "DATABASE_OPERATION_FAILED",
                    self.to_string(),
                    ["Preserve the database and capsule, run doctor --json, then retry only after the durable state is healthy."],
                )
            }
            Self::Infrastructure(crate::infrastructure::HarnessInfraError::Io(_)) => {
                StructuredErrorResult::new(
                    "FILESYSTEM_OPERATION_FAILED",
                    self.to_string(),
                    ["Preserve the task capsule, correct the filesystem path or permissions, and retry task finish."],
                )
            }
            Self::ParseHarnessValue(_) | Self::Usage(_) => StructuredErrorResult::new(
                "CLI_USAGE_ERROR",
                self.to_string(),
                ["Correct the documented argument value and retry the same command."],
            ),
            _ => StructuredErrorResult::new(
                "CLI_ERROR",
                self.to_string(),
                ["Inspect the command context and retry after correcting the reported failure."],
            ),
        }
    }
}

pub fn usage_error_result(error: &clap::Error) -> StructuredErrorResult {
    StructuredErrorResult::new(
        "CLI_USAGE_ERROR",
        error.to_string().trim().to_owned(),
        ["Run the same command with --help, correct the arguments, and retry."],
    )
    .with_detail("kind", format!("{:?}", error.kind()))
}

pub fn render_error(result: &StructuredErrorResult, json: bool) {
    if json {
        eprintln!(
            "{}",
            serde_json::to_string(result).expect("structured error result serializes")
        );
        return;
    }
    eprintln!("error: {}", result.code);
    eprintln!("{}", result.message);
    for (key, value) in &result.details {
        eprintln!("{key}: {value}");
    }
    for remediation in &result.remediation {
        eprintln!("remediation: {remediation}");
    }
}

pub fn run(cli: Cli) -> Result<(), InterfaceError> {
    let context = resolve_context()?;
    let repo_root = context.repo_root.clone();
    let active_db_path = context.db_path.clone();
    let service = HarnessService::new(context);

    match cli.command {
        Command::Doctor(args) => {
            let report = service.doctor()?;
            if args.json {
                println!(
                    "{{\"ok\":{},\"code\":\"{}\",\"message\":\"{}\",\"details\":{{\"platform\":\"{}\",\"repository_id\":{},\"worktree\":{},\"branch\":{},\"commit\":{},\"source_versions\":{},\"db_versions\":{},\"findings\":{}}},\"remediation\":{}}}",
                    report.ok,
                    json_escape(&report.code),
                    json_escape(&report.message),
                    json_escape(&report.platform),
                    json_optional(report.repository_id.as_deref()),
                    json_optional(report.worktree.as_deref()),
                    json_optional(report.branch.as_deref()),
                    json_optional(report.commit.as_deref()),
                    json_numbers(&report.source_versions),
                    json_numbers(&report.db_versions),
                    json_strings(&report.findings),
                    json_strings(&report.remediation),
                );
            } else {
                println!("doctor: {}", report.code);
                println!("{}", report.message);
                println!("platform: {}", report.platform);
                println!(
                    "repository id: {}",
                    report.repository_id.as_deref().unwrap_or("<missing>")
                );
                println!(
                    "worktree: {}",
                    report.worktree.as_deref().unwrap_or("<unavailable>")
                );
                println!(
                    "branch: {}",
                    report.branch.as_deref().unwrap_or("<unavailable>")
                );
                println!(
                    "commit: {}",
                    report.commit.as_deref().unwrap_or("<unavailable>")
                );
                println!("source migrations: {:?}", report.source_versions);
                println!("database migrations: {:?}", report.db_versions);
                for finding in &report.findings {
                    println!("- {finding}");
                }
                for remediation in &report.remediation {
                    println!("remediation: {remediation}");
                }
            }
            if !report.ok || (args.strict && report.code != "HEALTHY") {
                std::process::exit(3);
            }
        }
        Command::Workflow(args) => {
            let policy = service.workflow_policy()?;
            match args.action {
                WorkflowAction::Validate { json } => {
                    if json {
                        println!("{{\"ok\":true,\"code\":\"WORKFLOW_VALID\",\"policy_id\":\"{}\",\"policy_version\":\"{}\",\"mode\":\"{}\"}}", json_escape(&policy.policy_id), json_escape(&policy.policy_version), json_escape(&policy.mode));
                    } else {
                        println!(
                            "workflow: valid ({} {}, mode={})",
                            policy.policy_id, policy.policy_version, policy.mode
                        );
                    }
                }
                WorkflowAction::Explain { json, flags } => {
                    let flags = flags
                        .unwrap_or_default()
                        .split(',')
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(str::to_owned)
                        .collect::<Vec<_>>();
                    let (lane, gates) = policy.classify(&flags);
                    if json {
                        println!("{{\"policy_id\":\"{}\",\"policy_version\":\"{}\",\"mode\":\"{}\",\"lane\":\"{}\",\"gates\":{}}}", json_escape(&policy.policy_id), json_escape(&policy.policy_version), json_escape(&policy.mode), lane, json_strings(&gates));
                    } else {
                        println!(
                            "workflow {} {}\nlane: {}\ngates: {}",
                            policy.policy_id,
                            policy.policy_version,
                            lane,
                            gates.join(", ")
                        );
                    }
                }
                WorkflowAction::Context {
                    paths,
                    lane,
                    phase,
                    flags,
                    linked_artifacts,
                    json,
                } => {
                    let paths = paths
                        .unwrap_or_default()
                        .split(',')
                        .map(str::trim)
                        .filter(|path| !path.is_empty())
                        .map(str::to_owned)
                        .collect::<Vec<_>>();
                    let flags = flags
                        .unwrap_or_default()
                        .split(',')
                        .map(str::trim)
                        .filter(|flag| !flag.is_empty())
                        .map(str::to_owned)
                        .collect::<Vec<_>>();
                    let linked_artifacts = linked_artifacts
                        .unwrap_or_default()
                        .split(',')
                        .map(str::trim)
                        .filter(|path| !path.is_empty())
                        .map(str::to_owned)
                        .collect::<Vec<_>>();
                    let manifest =
                        policy.context_manifest(&lane, &phase, &paths, &flags, &linked_artifacts);
                    if json {
                        println!(
                            "{{\"policy_id\":\"{}\",\"policy_version\":\"{}\",\"mode\":\"{}\",\"lane\":\"{}\",\"phase\":\"{}\",\"must_read\":{},\"should_read\":{},\"skip\":{},\"stop_condition\":\"{}\",\"token_budget_hint\":{},\"checksum\":\"{}\"}}",
                            json_escape(&manifest.policy_id),
                            json_escape(&manifest.policy_version),
                            json_escape(&manifest.policy_mode),
                            json_escape(&manifest.lane),
                            json_escape(&manifest.phase),
                            context_entries_json(&manifest.must_read),
                            context_entries_json(&manifest.should_read),
                            context_entries_json(&manifest.skip),
                            json_escape(&manifest.stop_condition),
                            manifest.token_budget_hint,
                            manifest.checksum,
                        );
                    } else {
                        for entry in manifest.must_read {
                            println!("must_read {} ({})", entry.path, entry.reason);
                        }
                        for entry in manifest.should_read {
                            println!("should_read {} ({})", entry.path, entry.reason);
                        }
                        for entry in manifest.skip {
                            println!("skip {} ({})", entry.path, entry.reason);
                        }
                        println!("{}", manifest.stop_condition);
                        println!("token budget hint: {}", manifest.token_budget_hint);
                        println!("manifest checksum: {}", manifest.checksum);
                    }
                }
                WorkflowAction::Commands { json } => {
                    let commands = compiled_command_manifest();
                    if json {
                        println!("{{\"commands\":{}}}", json_strings(&commands));
                    } else {
                        for command in commands {
                            println!("{command}");
                        }
                    }
                }
                WorkflowAction::Parity { json } => {
                    let result =
                        workflow_parity(&repo_root, &policy, &compiled_command_manifest())?;
                    if json {
                        println!(
                            "{{\"ok\":{},\"code\":\"{}\",\"checked\":{},\"intentional_deltas\":{},\"failures\":{}}}",
                            result.failures.is_empty(),
                            if result.failures.is_empty() { "WORKFLOW_PARITY_OK" } else { "WORKFLOW_PARITY_DRIFT" },
                            json_strings(&result.checked),
                            json_strings(&result.deltas),
                            json_strings(&result.failures),
                        );
                    } else {
                        for check in &result.checked {
                            println!("ok: {check}");
                        }
                        for delta in &result.deltas {
                            println!("intentional delta: {delta}");
                        }
                        for failure in &result.failures {
                            println!("drift: {failure}");
                        }
                    }
                    if !result.failures.is_empty() {
                        std::process::exit(6);
                    }
                }
            }
        }
        Command::Init => print_init_result(service.init()?),
        Command::Migrate => print_migrate_result(service.migrate()?),
        Command::Import(args) => match args.source {
            ImportSource::Brownfield => {
                print_brownfield_import_result(service.import_brownfield()?)
            }
        },
        Command::Intake(args) => {
            let id = service.record_intake(IntakeInput {
                input_type: InputType::from_str(&args.input_type)?,
                summary: args.summary,
                risk_lane: RiskLane::from_str(&args.lane)?,
                risk_flags: CsvList::from_optional(args.flags),
                affected_docs: CsvList::from_optional(args.docs),
                story_id: args.story,
                notes: args.notes,
            })?;
            println!("Intake #{id} recorded.");
        }
        Command::Task(args) => match args.action {
            TaskAction::Start(args) => {
                let input_type = InputType::from_str(&args.input_type)?;
                let risk_flags = args
                    .flags
                    .as_deref()
                    .map(|value| {
                        value
                            .split(',')
                            .map(str::trim)
                            .filter(|item| !item.is_empty())
                            .map(str::to_owned)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let behavior = classify_behavior_bearing(
                    &args.behavior_bearing,
                    &input_type,
                    &risk_flags,
                    args.story.as_deref(),
                )?;
                let (owner, session, ownership_reason) =
                    resolve_start_identity(args.owner, args.session)?;
                let policy = service.workflow_policy()?;
                let (recommended_lane, policy_gates) = policy.classify(&risk_flags);
                let mut lane_reasons = if risk_flags.is_empty() {
                    vec!["policy-default:no-risk-flags".to_owned()]
                } else {
                    risk_flags
                        .iter()
                        .map(|flag| format!("explicit-flag:{}", normalize_token(flag)))
                        .collect::<Vec<_>>()
                };
                if let Some(requested) = args.lane.as_deref() {
                    if normalize_token(requested) != normalize_token(&recommended_lane) {
                        lane_reasons.push(format!(
                            "approved-override:{}",
                            args.lane_reason.as_deref().unwrap_or("<missing>")
                        ));
                    }
                }
                let id = service.start_task(TaskStartInput {
                    input_type,
                    summary: args.summary,
                    risk_lane: args.lane.as_deref().map(RiskLane::from_str).transpose()?,
                    lane_override_reason: args.lane_reason,
                    owner: Some(owner.clone()),
                    session_id: Some(session.clone()),
                    lease_seconds: args.lease_seconds,
                    story_id: args.story.clone(),
                    behavior_bearing: behavior.value,
                    risk_flags: risk_flags.clone(),
                })?;
                let task = service.task_status(&id)?;
                let contract = task_start_contract_json(
                    &task,
                    &policy,
                    &behavior,
                    &lane_reasons,
                    &policy_gates,
                    &ownership_reason,
                );
                if args.json {
                    println!("{}", serde_json::to_string(&contract).expect("task start contract serializes"));
                } else {
                    print_task_start_contract_human(&task, &contract);
                }
            }
            TaskAction::Status { id, json } => {
                let task = service.task_status(&id)?;
                let contract = task_status_contract_json(&service, &task)?;
                if json {
                    println!("{}", serde_json::to_string(&contract).expect("task status contract serializes"));
                } else {
                    print_task_status_contract_human(&contract);
                }
            }
            TaskAction::Block(args) => {
                let task = service.transition_task(TaskTransitionInput {
                    id: args.id,
                    status: "blocked".to_owned(),
                    outcome: None,
                    owner: args.owner,
                    session_id: args.session,
                    lease_seconds: None,
                })?;
                print_task_transition(&task, args.json);
            }
            TaskAction::Resume(args) => {
                let task = service.transition_task(TaskTransitionInput {
                    id: args.id,
                    status: "in_progress".to_owned(),
                    outcome: None,
                    owner: args.owner,
                    session_id: args.session,
                    lease_seconds: args.lease_seconds,
                })?;
                print_task_transition(&task, args.json);
            }
            TaskAction::Abandon(args) => {
                let task = service.transition_task(TaskTransitionInput {
                    id: args.id,
                    status: "abandoned".to_owned(),
                    outcome: Some(args.outcome),
                    owner: args.owner,
                    session_id: args.session,
                    lease_seconds: None,
                })?;
                print_task_transition(&task, args.json);
            }
            TaskAction::Handoff(args) => {
                service.handoff_task(TaskHandoffInput {
                    id: args.id,
                    from_owner: args.from_owner,
                    from_session: args.from_session,
                    to_owner: args.to_owner,
                    to_session: args.to_session,
                    lease_seconds: args.lease_seconds,
                    source: args.source,
                    evidence: args.evidence,
                    scope: args.scope,
                })?;
                if args.json {
                    println!("{{\"ok\":true,\"handed_off\":true}}");
                } else {
                    println!("Task handoff recorded.");
                }
            }
            TaskAction::LinkStory(args) => {
                service.link_task_story(TaskStoryLinkInput {
                    id: args.id,
                    story_id: args.story,
                    role: args.role,
                    owner: args.owner,
                    session_id: args.session,
                })?;
                if args.json {
                    println!("{{\"ok\":true,\"linked\":true}}");
                } else {
                    println!("Task story link recorded.");
                }
            }
            TaskAction::Finish(args) => {
                if args.outcome != "completed" {
                    return Err(InterfaceError::WorkflowParity(
                        "task finish currently accepts only --outcome completed".to_owned(),
                    ));
                }
                let trace_id = args.trace.map(|value| value.parse::<i64>().map_err(|_| {
                    InterfaceError::Usage(
                        "task finish: --trace must be a numeric trace id".to_owned(),
                    )
                })).transpose()?;
                let finished = service.finish_task(TaskFinishInput {
                    id: args.id,
                    owner: args.owner,
                    session_id: args.session,
                    trace_id,
                    friction: args.friction,
                    capsule_path: args.capsule,
                })?;
                if args.json {
                    println!(
                        "{{\"ok\":true,\"task_id\":\"{}\",\"status\":\"{}\",\"trace_id\":{}}}",
                        json_escape(&finished.id),
                        json_escape(&finished.status),
                        finished.trace_id,
                    );
                } else {
                    println!("Task {} completed.", finished.id);
                }
            }
            TaskAction::Refresh(args) => {
                let refresh = service.refresh_task(TaskRefreshInput {
                    id: args.id,
                    accept: args.accept,
                })?;
                if args.json {
                    println!("{{\"ok\":{},\"task_id\":\"{}\",\"changed\":{},\"applied\":{},\"previous_checksum\":\"{}\",\"current_checksum\":\"{}\",\"changed_paths\":{}}}", !refresh.changed || refresh.applied, json_escape(&refresh.id), refresh.changed, refresh.applied, json_escape(&refresh.previous_checksum), json_escape(&refresh.current_checksum), json_strings(&refresh.changed_paths));
                } else if refresh.changed {
                    println!(
                        "Task {} context changed: {}. Re-run with --accept to apply.",
                        refresh.id,
                        refresh.changed_paths.join(", ")
                    );
                } else {
                    println!("Task {} context is current.", refresh.id);
                }
                if refresh.changed && !refresh.applied {
                    std::process::exit(5);
                }
            }
            TaskAction::Context(args) => match args.action {
                TaskContextAction::Acknowledge {
                    id,
                    path,
                    actor,
                    json,
                } => {
                    service.acknowledge_task_context(TaskContextAcknowledgeInput {
                        id,
                        path,
                        actor,
                    })?;
                    if json {
                        println!("{{\"ok\":true,\"acknowledged\":true}}");
                    } else {
                        println!("Task context acknowledged.");
                    }
                }
            },
            TaskAction::Approve(args) => {
                service.approve_task(TaskApprovalInput {
                    id: args.id,
                    gate: args.gate,
                    source: args.source,
                    evidence: args.evidence,
                    scope: args.scope,
                })?;
                if args.json {
                    println!("{{\"ok\":true,\"approved\":true}}");
                } else {
                    println!("Task approval recorded.");
                }
            }
        },
        Command::Proof(args) => match args.action {
            ProofAction::Run(args) => {
                let (executable, argv) =
                    args.command
                        .split_first()
                        .ok_or(InterfaceError::WorkflowParity(
                            "proof run requires a command after --".to_owned(),
                        ))?;
                let proof = service.run_proof(ProofRunInput {
                    task_id: args.task_id,
                    story_id: args.story,
                    layer: args.layer,
                    executable: executable.clone(),
                    argv: argv.to_vec(),
                    artifact_path: args.artifact,
                })?;
                if args.json {
                    println!("{{\"ok\":{},\"task_id\":\"{}\",\"layer\":\"{}\",\"state\":\"{}\",\"exit_code\":{},\"head_commit\":{},\"branch\":{},\"stdout_path\":\"{}\",\"stdout_hash\":\"{}\",\"stderr_path\":\"{}\",\"stderr_hash\":\"{}\",\"artifact_path\":{},\"artifact_hash\":{}}}", proof.state == "pass", json_escape(&proof.task_id), json_escape(&proof.layer), json_escape(&proof.state), proof.exit_code, json_optional(proof.head_commit.as_deref()), json_optional(proof.branch.as_deref()), json_escape(&proof.stdout_path), json_escape(&proof.stdout_hash), json_escape(&proof.stderr_path), json_escape(&proof.stderr_hash), json_optional(proof.artifact_path.as_deref()), json_optional(proof.artifact_hash.as_deref()));
                } else {
                    println!(
                        "Proof {} for task {}: {} (exit {}).",
                        proof.layer, proof.task_id, proof.state, proof.exit_code
                    );
                }
                if proof.state != "pass" {
                    std::process::exit(7);
                }
            }
            ProofAction::Query { task_id, json } => {
                let proofs = service.query_proofs(&task_id)?;
                if json {
                    println!("[{}]", proofs.iter().map(proof_record_json).collect::<Vec<_>>().join(","));
                } else if proofs.is_empty() {
                    println!("No proof runs.");
                } else {
                    for proof in proofs {
                        println!(
                            "{}\t{}\t{}\t{}",
                            proof.layer,
                            proof.state,
                            proof
                                .exit_code
                                .map(|value| value.to_string())
                                .unwrap_or_else(|| "<none>".to_owned()),
                            proof.head_commit.unwrap_or_else(|| "<none>".to_owned())
                        );
                    }
                }
            }
        },
        Command::Friction(args) => match args.action {
            FrictionAction::Add { task, category, severity, summary, disposition, baseline, predicted_metric, observation_window, json } => {
                let fingerprint = service.add_friction(FrictionAddInput { task_id: task, category, severity, summary, disposition, baseline, predicted_metric, observation_window })?;
                if json { println!("{{\"ok\":true,\"fingerprint\":\"{}\"}}", json_escape(&fingerprint)); } else { println!("Friction recorded: {fingerprint}"); }
            }
            FrictionAction::Resolve { fingerprint, status, actual_outcome, json } => {
                service.resolve_friction(FrictionResolveInput { fingerprint, status, actual_outcome })?;
                if json { println!("{{\"ok\":true,\"resolved\":true}}"); } else { println!("Friction resolved."); }
            }
            FrictionAction::Query => print_query_table(&service.query_sql(
                "SELECT fingerprint, category, severity, disposition, status, summary, actual_outcome FROM friction ORDER BY id DESC;"
            )?),
        },
        Command::Memory(args) => match args.action {
            MemoryAction::Check { dry_run, json } => {
                if !dry_run {
                    return Err(InterfaceError::WorkflowParity(
                        "memory check requires --dry-run in CL-30; rebuild is owned by CL-31"
                            .to_owned(),
                    ));
                }
                print_artifact_check(artifact_check(&repo_root, None, None), json);
            }
            MemoryAction::Rebuild {
                dry_run,
                apply,
                recover_foreign,
                output,
                json,
            } => {
                if dry_run == apply {
                    return Err(InterfaceError::WorkflowParity(
                        "memory rebuild requires exactly one of --dry-run or --apply".to_owned(),
                    ));
                }
                if apply && output.is_some() {
                    return Err(InterfaceError::WorkflowParity(
                        "memory rebuild --apply cannot be combined with --output".to_owned(),
                    ));
                }
                let active_code = if apply {
                    let active_report = service.doctor()?;
                    if !rebuild_apply_state_allowed(&active_report.code, recover_foreign) {
                        return Err(InterfaceError::WorkflowParity(format!(
                            "memory rebuild --apply refused active DB state {}; use --recover-foreign only for a reviewed foreign/unhealthy recovery",
                            active_report.code
                        )));
                    }
                    Some(active_report.code)
                } else {
                    None
                };
                let artifacts = artifact_check(&repo_root, None, None);
                if !artifacts.errors.is_empty() {
                    print_artifact_check(artifacts, json);
                    return Ok(());
                }
                let output = output
                    .map(|path| {
                        if std::path::Path::new(&path).is_absolute()
                            || path.split('/').any(|part| part == "..")
                        {
                            Err(InterfaceError::WorkflowParity(
                                "memory rebuild --output must be repo-relative without traversal"
                                    .to_owned(),
                            ))
                        } else {
                            Ok(repo_root.join(path))
                        }
                    })
                    .transpose()?;
                if let Some(path) = &output {
                    if path.exists() {
                        return Err(InterfaceError::WorkflowParity(format!(
                            "memory rebuild refuses to overwrite existing output {}",
                            path.display()
                        )));
                    }
                }
                let temporary =
                    active_db_path.with_extension(format!("rebuild-{}.db", std::process::id()));
                let _ = fs::remove_file(&temporary);
                let preserve_operational = apply
                    && active_db_path.exists()
                    && active_code.as_deref() == Some("HEALTHY");
                let backup_path = if apply && active_db_path.exists() {
                    checkpoint_rebuild_database(&active_db_path)?;
                    let backup_dir = active_db_path
                        .parent()
                        .unwrap_or(&repo_root)
                        .join("harness.db.backups");
                    fs::create_dir_all(&backup_dir).map_err(|error| {
                        InterfaceError::WorkflowParity(format!(
                            "cannot create rebuild backup directory: {error}"
                        ))
                    })?;
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos();
                    let backup = backup_dir.join(format!("rebuild-{timestamp}.db"));
                    fs::copy(&active_db_path, &backup).map_err(|error| {
                        InterfaceError::WorkflowParity(format!("cannot back up active DB: {error}"))
                    })?;
                    if preserve_operational {
                        fs::copy(&active_db_path, &temporary).map_err(|error| {
                            InterfaceError::WorkflowParity(format!(
                                "cannot create retained-state rebuild candidate: {error}"
                            ))
                        })?;
                    }
                    Some(backup)
                } else {
                    None
                };
                let rebuild = HarnessService::new(HarnessContext {
                    repo_root: repo_root.clone(),
                    db_path: temporary.clone(),
                    schema_dir: resolve_schema_dir(&repo_root),
                });
                rebuild.init()?;
                let projected_records = project_artifact_index(
                    &temporary,
                    &artifacts.artifacts,
                    preserve_operational,
                )?;
                let logical_digest = rebuild_logical_digest(&temporary)?;
                let parity = rebuild_parity_report(&temporary, &artifacts.artifacts)?;
                let report = rebuild.doctor()?;
                let candidate_audit = rebuild.audit()?;
                if !report.ok || parity["state"] != "pass" {
                    let _ = fs::remove_file(&temporary);
                    let _ = fs::remove_file(format!("{}-wal", temporary.display()));
                    let _ = fs::remove_file(format!("{}-shm", temporary.display()));
                    return Err(InterfaceError::WorkflowParity(format!(
                        "memory rebuild candidate failed validation: doctor={}, parity={}; active DB was not replaced",
                        report.code, parity["state"]
                    )));
                }
                checkpoint_rebuild_database(&temporary)?;
                let output_path = if apply {
                    fs::rename(&temporary, &active_db_path).map_err(|error| {
                        InterfaceError::WorkflowParity(format!(
                            "cannot atomically replace active DB: {error}"
                        ))
                    })?;
                    active_db_path.to_string_lossy().into_owned()
                } else if let Some(output) = output {
                    fs::rename(&temporary, &output).map_err(|error| {
                        InterfaceError::WorkflowParity(format!(
                            "cannot publish validated rebuild output: {error}"
                        ))
                    })?;
                    output.to_string_lossy().into_owned()
                } else {
                    fs::remove_file(&temporary).map_err(|error| {
                        InterfaceError::WorkflowParity(format!(
                            "cannot remove temporary rebuild DB: {error}"
                        ))
                    })?;
                    "<discarded>".to_owned()
                };
                let _ = fs::remove_file(format!("{}-wal", temporary.display()));
                let _ = fs::remove_file(format!("{}-shm", temporary.display()));
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "ok": true,
                            "mode": if apply { "apply" } else { "dry_run" },
                            "artifacts_checked": artifacts.checked.len(),
                            "temp_schema_version": parity["schema_version"],
                            "doctor": report.code,
                            "candidate_audit": {
                                "strict": candidate_audit.strict_passes(),
                                "finding_count": candidate_audit.finding_count(),
                                "unknown_coverage": candidate_audit.unknown_coverage,
                            },
                            "projected_records": projected_records,
                            "logical_digest": logical_digest,
                            "parity": parity,
                            "preserved_operational_state": preserve_operational,
                            "backup": backup_path.as_ref().map(|path| path.strip_prefix(&repo_root).unwrap_or(path).to_string_lossy().into_owned()),
                            "output": output_path,
                        })
                    );
                } else {
                    println!("rebuild {}: {} artifacts validated; temporary database doctor={}; semantic parity=pass; projected records={projected_records}; logical digest={logical_digest}", if apply { "apply" } else { "dry-run" }, artifacts.checked.len(), report.code);
                }
            }
            MemoryAction::Capsule(args) => match args.action {
                CapsuleAction::Render {
                    id,
                    date,
                    lane,
                    outcome,
                    summary,
                    json,
                } => {
                    let path = render_capsule(
                        &repo_root,
                        &active_db_path,
                        &id,
                        &date,
                        &lane,
                        &outcome,
                        &summary,
                    )?;
                    if json {
                        println!("{{\"ok\":true,\"path\":\"{}\"}}", json_escape(&path));
                    } else {
                        println!("capsule: {path}");
                    }
                }
                CapsuleAction::Check { json } => {
                    let result = capsule_check(&repo_root)?;
                    if json {
                        println!(
                            "{{\"ok\":{},\"orphans\":{}}}",
                            result.is_empty(),
                            json_strings(&result)
                        );
                    } else {
                        for path in &result {
                            println!("orphan: {path}");
                        }
                    }
                    if !result.is_empty() {
                        std::process::exit(6);
                    }
                }
            },
        },
        Command::Story(args) => match args.action {
            StoryAction::Add(args) => {
                service.add_story(StoryAddInput {
                    id: args.id.clone(),
                    title: args.title,
                    risk_lane: RiskLane::from_str(&args.lane)?,
                    contract_doc: args.contract,
                    verify_command: args.verify,
                    notes: args.notes,
                })?;
                println!("Story {} added.", args.id);
            }
            StoryAction::Update(args) => {
                service.update_story(StoryUpdateInput {
                    id: args.id.clone(),
                    status: args.status,
                    evidence: args.evidence,
                    unit: parse_optional_bool("story update: --unit", args.unit)?,
                    integration: parse_optional_bool(
                        "story update: --integration",
                        args.integration,
                    )?,
                    e2e: parse_optional_bool("story update: --e2e", args.e2e)?,
                    platform: parse_optional_bool("story update: --platform", args.platform)?,
                    verify_command: args.verify,
                })?;
                println!("Story {} updated.", args.id);
            }
            StoryAction::Verify { id } => {
                let result = service.verify_story(&id)?;
                println!("Running: {}", result.command);
                print!("{}", result.stdout);
                print!("{}", result.stderr);
                println!("Story {id} verification: {}", result.result);
                if result.result == "fail" {
                    std::process::exit(1);
                }
            }
            StoryAction::VerifyAll => {
                let result = service.verify_all_stories()?;
                print_story_verify_all(&result);
                if result.failed() > 0 {
                    std::process::exit(1);
                }
            }
            StoryAction::Check(args) => {
                print_artifact_check(
                    artifact_check(&repo_root, Some("story"), args.path),
                    args.json,
                );
            }
        },
        Command::Decision(args) => match args.action {
            DecisionAction::Add(args) => {
                service.add_decision(DecisionAddInput {
                    id: args.id.clone(),
                    title: args.title,
                    status: args.status,
                    doc_path: args.doc,
                    verify_command: args.verify,
                    predicted_impact: args.predicted,
                    notes: args.notes,
                })?;
                println!("Decision {} added.", args.id);
            }
            DecisionAction::Verify { id } => {
                let result = service.verify_decision(&id)?;
                println!("Running: {}", result.command);
                println!("Decision {id} verification: {}", result.result);
                if result.result == "fail" {
                    std::process::exit(1);
                }
            }
            DecisionAction::Check(args) => {
                print_artifact_check(
                    artifact_check(&repo_root, Some("decision"), args.path),
                    args.json,
                );
            }
        },
        Command::Backlog(args) => match args.action {
            BacklogAction::Add(args) => {
                let id = service.add_backlog(BacklogAddInput {
                    title: args.title,
                    discovered_while: args.discovered_while,
                    current_pain: args.pain,
                    suggestion: args.suggestion,
                    risk: args
                        .risk
                        .map(|value| RiskLane::from_str(&value))
                        .transpose()?,
                    predicted_impact: args.predicted,
                    notes: args.notes,
                })?;
                println!("Backlog #{id} added.");
            }
            BacklogAction::Close(args) => {
                let id = parse_optional_integer("backlog close: --id", Some(args.id))?
                    .expect("value provided");
                let status = args.status;
                service.close_backlog(BacklogCloseInput {
                    id,
                    status: status.clone(),
                    actual_outcome: args.outcome,
                })?;
                println!("Backlog #{id} closed as {status}.");
            }
        },
        Command::Tool(args) => match args.action {
            ToolAction::Register(args) => {
                let kind = validate_tool_kind(&args.kind)?;
                let capability = args
                    .capability
                    .as_deref()
                    .map(normalize_capability)
                    .transpose()?;
                service.register_tool(ToolRegisterInput {
                    name: args.name.clone(),
                    command: args.command,
                    description: args.description,
                    responsibility: validate_responsibility(&args.responsibility)?,
                    args: parse_tool_args(args.args)?,
                    force: args.force,
                    kind,
                    capability,
                    scan_target: args.scan,
                })?;
                println!("Tool {} registered.", args.name);
            }
            ToolAction::Check(args) => {
                let results = service.check_tools(args.name)?;
                if args.json {
                    print_tool_check_json(&results);
                } else {
                    print_tool_check_summary(&results);
                }
            }
            ToolAction::Remove { name } => {
                service.remove_tool(&name)?;
                println!("Tool {name} removed.");
            }
        },
        Command::Intervention(args) => match args.action {
            InterventionAction::Add(args) => {
                let id = service.add_intervention(InterventionAddInput {
                    trace_id: parse_optional_integer("intervention add: --trace", args.trace)?,
                    story_id: args.story,
                    intervention_type: args.intervention_type,
                    description: args.description,
                    source: args.source,
                    impact: args.impact,
                })?;
                println!("Intervention #{id} recorded.");
            }
        },
        Command::Trace(args) => {
            let story_id = args.story.clone();
            let id = service.record_trace(TraceInput {
                task_summary: args.summary,
                intake_id: parse_optional_integer("trace: --intake", args.intake)?,
                story_id: args.story,
                agent: args.agent,
                outcome: args.outcome,
                duration_seconds: parse_optional_integer("trace: --duration", args.duration)?,
                token_estimate: parse_optional_integer("trace: --tokens", args.tokens)?,
                friction: args.friction,
                notes: args.notes,
                actions: CsvList::from_optional(args.actions),
                files_read: CsvList::from_optional(args.files_read),
                files_changed: CsvList::from_optional(args.files_changed),
                decisions: CsvList::from_optional(args.decisions),
                errors: CsvList::from_optional(args.errors),
            })?;
            println!("Trace #{id} recorded.");
            let result = service.score_trace(Some(id))?;
            print_trace_score(&result, false);
            println!("Reminder: Record any human corrections with: harness-cli intervention add");
            if let Some(story_id) = story_id {
                print_story_verify_warning(&service, &story_id)?;
            }
        }
        Command::ScoreTrace(args) => {
            let id = parse_optional_integer("score-trace: --id", args.id)?;
            let result = service.score_trace(id)?;
            print_trace_score(&result, id.is_none());
            if !result.meets_requirement {
                std::process::exit(1);
            }
        }
        Command::ScoreContext { trace_id } => {
            let id = parse_optional_integer("score-context: trace-id", Some(trace_id))?
                .expect("value provided");
            print_context_score(&service.score_context(id)?);
        }
        Command::Audit(args) => match args.action {
            Some(AuditAction::Disposition(args)) => match args.action {
                AuditDispositionAction::Add(args) => {
                    let id = service.add_audit_disposition(AuditDispositionAddInput {
                        finding_key: args.finding_key,
                        entity_id: args.entity_id,
                        rationale: args.rationale,
                        provenance: args.provenance,
                        approval_task_id: args.approval_task,
                        approval_source: args.approval_source,
                        actor: args.actor,
                        expires_at: args.expires_at,
                    })?;
                    if args.json {
                        println!("{{\"ok\":true,\"id\":{id},\"status\":\"accepted\"}}");
                    } else {
                        println!("Audit disposition #{id} accepted.");
                    }
                }
                AuditDispositionAction::List { json } => {
                    let dispositions = service.list_audit_dispositions()?;
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string(&serde_json::json!({
                                "ok": true,
                                "dispositions": dispositions,
                            }))
                            .expect("audit dispositions serialize")
                        );
                    } else {
                        print_audit_dispositions(&dispositions);
                    }
                }
                AuditDispositionAction::Revoke(args) => {
                    let id = parse_optional_integer(
                        "audit disposition revoke: --id",
                        Some(args.id),
                    )?
                    .expect("value provided");
                    service.revoke_audit_disposition(AuditDispositionRevokeInput {
                        id,
                        actor: args.actor,
                        reason: args.reason,
                    })?;
                    if args.json {
                        println!("{{\"ok\":true,\"id\":{id},\"status\":\"revoked\"}}");
                    } else {
                        println!("Audit disposition #{id} revoked.");
                    }
                }
            },
            None => {
                let result = service.audit()?;
                if args.json {
                    let code = if result.strict_passes() {
                        "AUDIT_CLEAR"
                    } else {
                        "AUDIT_DEBT"
                    };
                    println!(
                        "{}",
                        serde_json::to_string(&serde_json::json!({
                            "ok": result.strict_passes(),
                            "code": code,
                            "message": if result.strict_passes() {
                                "All implemented audit checks passed with no unknown coverage."
                            } else {
                                "Audit debt or unknown coverage remains."
                            },
                            "audit": result,
                        }))
                        .expect("audit result serializes")
                    );
                } else {
                    print_audit(&result);
                }
                if args.strict && !result.strict_passes() {
                    std::process::exit(6);
                }
            }
        },
        Command::Propose(args) => print_proposals(&service.propose(args.commit)?),
        Command::Query(args) => match args.view {
            QueryView::Matrix(args) => print_matrix(&service.query_matrix()?, args.numeric),
            QueryView::Backlog(args) => {
                print_backlog(&service.query_backlog(backlog_filter(&args))?)
            }
            QueryView::Decisions => print_decisions(&service.query_decisions()?),
            QueryView::Intakes => print_intakes(&service.query_intakes()?),
            QueryView::Traces => print_traces(&service.query_traces()?),
            QueryView::Friction => print_friction(&service.query_friction()?),
            QueryView::Tools(args) => {
                let responsibility = args
                    .responsibility
                    .map(|value| validate_responsibility(&value))
                    .transpose()?;
                let capability = args
                    .capability
                    .as_deref()
                    .map(normalize_capability)
                    .transpose()?;
                let mut tools = service.query_tools(responsibility, capability)?;
                if let Some(status) = args.status.as_deref() {
                    let normalized = status.trim().to_lowercase();
                    tools.retain(|tool| tool.status == normalized);
                }
                if args.json {
                    print_tools_json(&tools);
                } else {
                    print_tools_summary(&tools);
                }
            }
            QueryView::Interventions(args) => {
                let trace_id = parse_optional_integer("query interventions: --trace", args.trace)?;
                print_interventions(&service.query_interventions(InterventionFilter {
                    trace_id,
                    story_id: args.story,
                    intervention_type: args.intervention_type,
                })?);
            }
            QueryView::Stats => print_stats(&service.query_stats()?),
            QueryView::Sql { query } => {
                if query.is_empty() {
                    return Err(InterfaceError::EmptySql);
                }
                print_query_table(&service.query_sql(&query.join(" "))?);
            }
        },
    }

    Ok(())
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn json_numbers(values: &[i64]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_strings(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn context_entries_json(values: &[crate::infrastructure::WorkflowContextEntry]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|entry| format!(
                "{{\"path\":\"{}\",\"reason\":\"{}\"}}",
                json_escape(&entry.path),
                json_escape(&entry.reason)
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn print_trace_score(result: &TraceScoreResult, latest: bool) {
    if latest {
        println!("Trace #{} (latest):", result.trace_id);
    } else {
        println!("Trace #{}:", result.trace_id);
    }
    println!(
        "  Tier achieved: {} ({}/3)",
        result.achieved.label(),
        result.achieved.score()
    );

    match (&result.risk_lane, result.required) {
        (Some(lane), Some(required)) => {
            println!(
                "  Lane: {} -> required tier: {} ({}/3)",
                lane,
                required.label(),
                required.score()
            );
            if result.meets_requirement {
                println!("  MEETS REQUIREMENT");
            } else {
                println!("  BELOW REQUIREMENT");
            }
        }
        _ => {
            println!("  Lane: unknown (no linked intake)");
        }
    }

    print_missing_fields(
        "minimal",
        TraceQualityTier::Minimal,
        &result.missing_minimal,
    );
    print_missing_fields(
        "standard",
        TraceQualityTier::Standard,
        &result.missing_standard,
    );
    print_missing_fields(
        "detailed",
        TraceQualityTier::Detailed,
        &result.missing_detailed,
    );
}

fn print_story_verify_all(result: &StoryVerifyAllResult) {
    for item in &result.items {
        match item.result.as_str() {
            "skipped" => println!("Story {}: skipped (no verify_command)", item.id),
            status => {
                println!("Story {}: {status}", item.id);
                if !item.stdout.is_empty() {
                    print!("{}", item.stdout);
                }
                if !item.stderr.is_empty() {
                    print!("{}", item.stderr);
                }
            }
        }
    }
    println!(
        "{} stories verified: {} passed, {} failed, {} skipped (no verify_command)",
        result.items.len(),
        result.passed(),
        result.failed(),
        result.skipped()
    );
}

fn print_context_score(result: &ContextScoreResult) {
    println!(
        "Trace #{} | Lane: {} | Phase: {}",
        result.trace_id, result.lane, result.phase
    );
    println!();
    let must_met = result.must.iter().filter(|item| item.met).count();
    println!("Must-read compliance: {must_met}/{}", result.must.len());
    for item in &result.must {
        println!(
            "  {} {} ({})",
            if item.met { "OK" } else { "MISSING" },
            item.label,
            item.target
        );
    }
    let should_met = result.should.iter().filter(|item| item.met).count();
    println!(
        "Should-read compliance: {should_met}/{}",
        result.should.len()
    );
    for item in &result.should {
        println!(
            "  {} {} ({})",
            if item.met { "OK" } else { "MISSING" },
            item.label,
            item.target
        );
    }
    println!("Over-reading: {} item(s)", result.over_read.len());
    for item in &result.over_read {
        println!("  - {item}");
    }
}

fn print_audit(result: &crate::domain::AuditResult) {
    println!("=== Harness Drift Audit ===");
    println!("Doctor health: {}", result.health_scope);
    println!();
    println!(
        "Accepted historical findings: {}",
        result.accepted_findings.len()
    );
    for finding in &result.accepted_findings {
        println!(
            "  - {}:{} via disposition #{} (approval task {}, actor {})",
            finding.finding_key,
            finding.entity_id,
            finding.disposition_id,
            finding.approval_task_id,
            finding.actor
        );
        println!("    title: {}", finding.title);
        println!("    rationale: {}", finding.rationale);
        println!("    provenance: {}", finding.provenance);
        println!("    approval: {}", finding.approval_source);
    }
    print_audit_category(
        "Orphaned stories (planned/in-progress, no traces)",
        &result.orphaned_stories,
    );
    print_audit_category(
        "Terminal tasks without final traces",
        &result.terminal_tasks_without_traces,
    );
    print_audit_category("Traces without task roots", &result.unrooted_traces);
    print_audit_category(
        "Completed normal/high-risk tasks below closure gates",
        &result.completed_tasks_below_gates,
    );
    print_audit_category("Unverified stories", &result.unverified_stories);
    print_audit_category("Unverified decisions", &result.unverified_decisions);
    print_audit_category(
        "Open backlog without outcomes",
        &result.backlog_without_outcomes,
    );
    print_audit_category("Stale stories", &result.stale_stories);
    print_audit_category("Broken tools", &result.broken_tools);
    print_audit_category(
        "Material friction without observed outcome",
        &result.friction_without_outcomes,
    );
    println!("Coverage checked: {}.", result.coverage.join(", "));
    println!("Named coverage checks: {}", result.coverage_checks.len());
    for check in &result.coverage_checks {
        println!(
            "  - {}@{}: {} (proof {:?}, freshness head={:?} branch={:?} dirty={:?} output={:?})",
            check.check_id,
            check.version,
            check.state,
            check.proof_run_id,
            check.freshness.head,
            check.freshness.branch,
            check.freshness.dirty,
            check.freshness.output,
        );
        if !check.measured_counts.is_empty() {
            println!("    counts: {:?}", check.measured_counts);
        }
        for remediation in &check.remediation {
            println!("    remediation: {remediation}");
        }
    }
    println!(
        "Coverage unknown: {}. Zero findings means no debt in checked coverage only.",
        if result.unknown_coverage.is_empty() {
            "none".to_owned()
        } else {
            result.unknown_coverage.join(", ")
        }
    );
    println!(
        "Entropy score: {}/100 (lower is better)",
        result.entropy_score()
    );
    print_maturity_report(&result.maturity);
}

fn print_maturity_report(report: &crate::domain::MaturityReport) {
    println!();
    println!("=== Outcome-derived Maturity Report ===");
    println!("Basis: {}", report.basis);
    println!(
        "H5 status: {} (measured improvements {}/{})",
        report.h5_status,
        report.measured_improvements.observed,
        report.measured_improvements.required
    );
    println!(
        "Observed terminal tasks: {}/{} (tiny {}/{}, normal {}/{}, high-risk {}/{})",
        report.evidence_backed_terminal_tasks.observed,
        report.evidence_backed_terminal_tasks.required,
        report.tiny_tasks.observed,
        report.tiny_tasks.required,
        report.normal_tasks.observed,
        report.normal_tasks.required,
        report.high_risk_tasks.observed,
        report.high_risk_tasks.required,
    );
    println!(
        "Required scenarios: blocked/resumed {}/{}, fresh-clone/rebuild {}/{}, installer-upgrade {}/{}",
        report.blocked_resumed_tasks.observed,
        report.blocked_resumed_tasks.required,
        report.fresh_clone_rebuild_tasks.observed,
        report.fresh_clone_rebuild_tasks.required,
        report.installer_upgrade_tasks.observed,
        report.installer_upgrade_tasks.required,
    );
    println!(
        "Completed normal/high-risk tasks meeting closure gates: {}/{}",
        report.completed_expanded_tasks_meeting_gates, report.completed_expanded_tasks
    );
    for gap in &report.gaps {
        println!("  gap: {gap}");
    }
}

fn print_audit_category(label: &str, findings: &[crate::domain::AuditFinding]) {
    println!();
    println!("{label}: {}", findings.len());
    for finding in findings {
        println!("  - {}: {}", finding.id, finding.title);
    }
}

fn print_audit_dispositions(dispositions: &[AuditDispositionRecord]) {
    println!("Audit dispositions: {}", dispositions.len());
    for disposition in dispositions {
        println!(
            "  #{} {}:{} [{}] approval={} actor={}",
            disposition.id,
            disposition.finding_key,
            disposition.entity_id,
            disposition.status,
            disposition.approval_task_id,
            disposition.actor
        );
        println!("    rationale: {}", disposition.rationale);
        println!("    provenance: {}", disposition.provenance);
        if let Some(expires_at) = &disposition.expires_at {
            println!("    expires_at: {expires_at}");
        }
        if let Some(reason) = &disposition.revocation_reason {
            println!("    revocation: {reason}");
        }
    }
}

fn print_proposals(proposals: &[ImprovementProposal]) {
    println!("=== Improvement Proposals ===");
    if proposals.is_empty() {
        println!("No proposals generated.");
        return;
    }
    for (index, proposal) in proposals.iter().enumerate() {
        println!();
        println!(
            "Proposal {} ({} confidence):",
            index + 1,
            proposal.confidence
        );
        println!("  Title: {}", proposal.title);
        println!("  Component: {}", proposal.component);
        println!("  Evidence: {}", proposal.evidence);
        println!("  Predicted impact: {}", proposal.predicted_impact);
        println!("  Risk: {}", proposal.risk);
        println!("  Suggested action: {}", proposal.suggested_action);
        println!("  Validation: {}", proposal.validation_plan);
        if let Some(id) = proposal.committed_backlog_id {
            println!("  Created backlog item #{id}");
        }
    }
    println!();
    println!(
        "{} proposals generated. Use --commit to create backlog items.",
        proposals.len()
    );
}

fn print_story_verify_warning(
    service: &HarnessService,
    story_id: &str,
) -> Result<(), InterfaceError> {
    let status = service.story_verify_status(story_id)?;
    let has_command = status
        .verify_command
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    if has_command && status.last_verified_result.as_deref() != Some("pass") {
        println!();
        println!(
            "Warning: Story {} has verify_command but verification has not passed.",
            status.id
        );
        println!("Run: harness-cli story verify {}", status.id);
    }
    Ok(())
}

fn print_missing_fields(label: &str, tier: TraceQualityTier, fields: &[String]) {
    if fields.is_empty() {
        return;
    }
    println!();
    println!("  Missing for {label}:");
    for field in fields {
        println!("    - {field}");
    }
    if tier == TraceQualityTier::Detailed {
        println!();
    }
}

fn backlog_filter(args: &BacklogQueryArgs) -> BacklogFilter {
    if args.open {
        BacklogFilter::Open
    } else if args.closed {
        BacklogFilter::Closed
    } else {
        BacklogFilter::All
    }
}

fn print_brownfield_import_result(result: BrownfieldImportResult) {
    println!("Brownfield import complete.");
    println!("Stories imported or updated: {}", result.stories);
    println!("Decisions imported or updated: {}", result.decisions);
    println!("Backlog items discovered: {}", result.backlog_items);
}

fn parse_optional_bool(
    label: &str,
    value: Option<String>,
) -> Result<Option<BoolFlag>, InterfaceError> {
    value
        .map(|inner| BoolFlag::parse(label, &inner))
        .transpose()
        .map_err(InterfaceError::from)
}

#[derive(Debug)]
struct BehaviorBearingSelection {
    mode: String,
    value: bool,
    reasons: Vec<String>,
}

fn classify_behavior_bearing(
    value: &str,
    input_type: &InputType,
    risk_flags: &[String],
    story_id: Option<&str>,
) -> Result<BehaviorBearingSelection, InterfaceError> {
    match normalize_token(value).as_str() {
        "yes" => Ok(BehaviorBearingSelection {
            mode: "explicit".to_owned(),
            value: true,
            reasons: vec!["explicit:yes".to_owned()],
        }),
        "no" => Ok(BehaviorBearingSelection {
            mode: "explicit".to_owned(),
            value: false,
            reasons: vec!["explicit:no".to_owned()],
        }),
        "auto" => {
            let normalized_flags = risk_flags
                .iter()
                .map(|flag| normalize_token(flag))
                .collect::<Vec<_>>();
            let explicitly_non_behavioral = normalized_flags.iter().any(|flag| {
                matches!(
                    flag.as_str(),
                    "non_behavioral" | "read_only" | "docs_only" | "documentation_only"
                )
            });
            let (behavior_bearing, mut reasons) = if story_id.is_some() {
                (true, vec!["linked-story".to_owned()])
            } else if explicitly_non_behavioral {
                (
                    false,
                    normalized_flags
                        .iter()
                        .filter(|flag| {
                            matches!(
                                flag.as_str(),
                                "non_behavioral"
                                    | "read_only"
                                    | "docs_only"
                                    | "documentation_only"
                            )
                        })
                        .map(|flag| format!("explicit-flag:{flag}"))
                        .collect(),
                )
            } else {
                let typed_behavior = !matches!(input_type, InputType::Maintenance);
                (
                    typed_behavior || !normalized_flags.is_empty(),
                    vec![format!("typed-input:{}", input_type.as_db_value())],
                )
            };
            if !normalized_flags.is_empty() && !explicitly_non_behavioral {
                reasons.extend(
                    normalized_flags
                        .iter()
                        .map(|flag| format!("explicit-flag:{flag}")),
                );
            }
            Ok(BehaviorBearingSelection {
                mode: "auto".to_owned(),
                value: behavior_bearing,
                reasons,
            })
        }
        _ => Err(InterfaceError::Usage(
            "task start: --behavior-bearing must be auto, yes, or no; auto uses typed intake, explicit flags, and linked story only"
                .to_owned(),
        )),
    }
}

fn resolve_start_identity(
    owner: Option<String>,
    session: Option<String>,
) -> Result<(String, String, String), InterfaceError> {
    match (owner, session) {
        (Some(owner), Some(session)) => {
            Ok((owner, session, "explicit-owner-session".to_owned()))
        }
        (None, None) => {
            let key = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            Ok((
                "harness-cli".to_owned(),
                format!("auto-{key}-{}", std::process::id()),
                "auto-assigned:harness-cli-ephemeral-session".to_owned(),
            ))
        }
        _ => Err(InterfaceError::Usage(
            "task start: --owner and --session must be supplied together, or both omitted for an explicit harness-cli auto-assignment"
                .to_owned(),
        )),
    }
}

fn task_start_contract_json(
    task: &crate::application::TaskStatusRecord,
    policy: &crate::infrastructure::WorkflowPolicy,
    behavior: &BehaviorBearingSelection,
    lane_reasons: &[String],
    policy_gates: &[String],
    ownership_reason: &str,
) -> serde_json::Value {
    let lane_policy = match task.risk_lane.as_str() {
        "high_risk" => &policy.lanes.high_risk,
        "normal" => &policy.lanes.normal,
        _ => &policy.lanes.tiny,
    };
    let story_required = lane_policy.story == "required"
        || (lane_policy.story == "when_behavior_bearing" && behavior.value);
    let normalized_flags = task
        .risk_flags
        .iter()
        .map(|flag| normalize_token(flag))
        .collect::<Vec<_>>();
    let approval_gates = policy
        .approvals
        .required_for
        .iter()
        .filter(|gate| normalized_flags.contains(&normalize_token(gate)))
        .cloned()
        .collect::<Vec<_>>();
    let decision_required = normalized_flags
        .iter()
        .any(|flag| matches!(flag.as_str(), "architecture_direction" | "source_hierarchy"));
    let owner = task.owner.as_deref().unwrap_or("<missing>");
    let session = task.session_id.as_deref().unwrap_or("<missing>");
    let first_context = task
        .context_manifest
        .get("must_read")
        .and_then(serde_json::Value::as_array)
        .and_then(|entries| entries.first())
        .and_then(|entry| entry.get("path"))
        .and_then(serde_json::Value::as_str);
    let next_command = if story_required && task.story_id.is_none() {
        format!(
            "_harness/bin/harness-cli task link-story --id {} --story <STORY> --role primary --owner {} --session {} --json",
            task.id, owner, session
        )
    } else if let Some(path) = first_context {
        format!(
            "_harness/bin/harness-cli task context acknowledge --id {} --read '{}' --actor {} --json",
            task.id,
            path.replace('\'', "'\\''"),
            owner
        )
    } else {
        format!(
            "_harness/bin/harness-cli proof run --task {} --layer {} -- <COMMAND>",
            task.id,
            lane_policy
                .proof
                .first()
                .map(String::as_str)
                .unwrap_or("required")
        )
    };
    let completion_gates = [
        ("story", story_required, task.story_id.is_some()),
        (
            "context",
            task.context_required > 0,
            task.context_acknowledged >= task.context_required,
        ),
        ("approval", !approval_gates.is_empty(), approval_gates.is_empty()),
        ("proof", true, false),
        ("trace", true, false),
        ("capsule", task.capsule_required, !task.capsule_required),
        ("friction", true, true),
    ]
    .into_iter()
    .map(|(gate, required, satisfied)| {
        serde_json::json!({
            "gate": gate,
            "required": required,
            "state": if !required { "not_applicable" } else if satisfied { "satisfied" } else { "unmet" },
        })
    })
    .collect::<Vec<_>>();
    serde_json::json!({
        "ok": true,
        "task_id": task.id,
        "status": task.status,
        "lane": task.risk_lane,
        "lane_reasons": lane_reasons,
        "behavior": {
            "mode": behavior.mode,
            "bearing": behavior.value,
            "reasons": behavior.reasons,
            "summary_inspected": false,
        },
        "ownership": {
            "owner": task.owner,
            "session": task.session_id,
            "reason": ownership_reason,
        },
        "lease": {
            "expires_at": task.lease_expires_at,
            "state": task.lease_state,
        },
        "requirements": {
            "story": {"required": story_required, "id": task.story_id},
            "decision": {"required": decision_required, "ids": []},
            "approval": {"required": !approval_gates.is_empty(), "gates": approval_gates},
        },
        "context": task.context_manifest,
        "proof_gates": lane_policy.proof,
        "completion_gates": completion_gates,
        "policy_gates": policy_gates,
        "relevant_tools": [],
        "stop_condition": task.context_manifest.get("stop_condition"),
        "next_command": next_command,
    })
}

fn print_task_start_contract_human(
    task: &crate::application::TaskStatusRecord,
    contract: &serde_json::Value,
) {
    println!("task: {}", task.id);
    println!("status: {}", task.status);
    println!("lane: {}", task.risk_lane);
    println!("lane reasons: {}", contract["lane_reasons"]);
    println!("behavior: {}", contract["behavior"]);
    println!("ownership: {}", contract["ownership"]);
    println!("requirements: {}", contract["requirements"]);
    println!("context: {}", contract["context"]);
    println!("proof gates: {}", contract["proof_gates"]);
    println!("completion gates: {}", contract["completion_gates"]);
    println!("relevant tools: {}", contract["relevant_tools"]);
    println!("stop condition: {}", contract["stop_condition"]);
    println!(
        "next: {}",
        contract["next_command"].as_str().unwrap_or("<none>")
    );
}

fn task_status_contract_json(
    service: &HarnessService,
    task: &crate::application::TaskStatusRecord,
) -> Result<serde_json::Value, InterfaceError> {
    let policy = service.workflow_policy()?;
    let lane_policy = match task.risk_lane.as_str() {
        "high_risk" => &policy.lanes.high_risk,
        "normal" => &policy.lanes.normal,
        _ => &policy.lanes.tiny,
    };
    let escaped_id = task.id.replace('\'', "''");
    let stories = query_table_objects(service.query_sql(&format!(
        "SELECT task_story.story_id AS id, task_story.role, story.status, story.title \
         FROM task_story JOIN story ON story.id=task_story.story_id \
         WHERE task_story.task_id='{escaped_id}' ORDER BY task_story.role, task_story.story_id"
    ))?);
    let approvals = query_table_objects(service.query_sql(&format!(
        "SELECT gate, source, evidence, COALESCE(scope,'') AS scope, created_at \
         FROM task_approval WHERE task_id='{escaped_id}' ORDER BY gate, created_at"
    ))?);
    let friction = query_table_objects(service.query_sql(&format!(
        "SELECT fingerprint, category, severity, disposition, status, summary, \
                COALESCE(actual_outcome,'') AS actual_outcome \
         FROM friction WHERE task_id='{escaped_id}' ORDER BY id"
    ))?);
    let traces = query_table_objects(service.query_sql(&format!(
        "SELECT trace.id, trace.outcome, COALESCE(trace.agent,''), trace.created_at \
         FROM trace JOIN task ON task.intake_id=trace.intake_id \
         WHERE task.id='{escaped_id}' ORDER BY trace.id"
    ))?);
    let proofs = service.query_proofs(&task.id)?;
    let proof_layers = proofs
        .iter()
        .map(|proof| {
            serde_json::json!({
                "story_id": proof.story_id, "layer": proof.layer, "state": proof.state,
                "executable": proof.executable,
                "argv": proof.argv_json.as_deref().and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok()),
                "exit_code": proof.exit_code, "head_commit": proof.head_commit,
                "branch": proof.branch, "dirty_fingerprint": proof.dirty_fingerprint,
                "cli_version": proof.cli_version, "platform": proof.platform,
                "command_digest": proof.command_digest,
                "stdout": {"path": proof.stdout_path, "hash": proof.stdout_hash},
                "stderr": {"path": proof.stderr_path, "hash": proof.stderr_hash},
                "artifact": {"path": proof.artifact_path, "hash": proof.artifact_hash},
            })
        })
        .collect::<Vec<_>>();
    let approval_gate_names = approvals
        .iter()
        .filter_map(|approval| approval.get("gate").and_then(serde_json::Value::as_str))
        .map(normalize_token)
        .collect::<Vec<_>>();
    let normalized_flags = task
        .risk_flags
        .iter()
        .map(|flag| normalize_token(flag))
        .collect::<Vec<_>>();
    let mut required_approvals = policy
        .approvals
        .required_for
        .iter()
        .filter(|gate| normalized_flags.contains(&normalize_token(gate)))
        .cloned()
        .collect::<Vec<_>>();
    if task.risk_lane == "high_risk" && required_approvals.is_empty() {
        required_approvals.push("risk-policy".to_owned());
    }
    let story_required = lane_policy.story == "required"
        || (lane_policy.story == "when_behavior_bearing" && task.behavior_bearing);
    let story_satisfied = !stories.is_empty();
    let context_satisfied = task.context_acknowledged >= task.context_required;
    let proof_freshness = serde_json::json!({
        "head": task.latest_proof_head_fresh, "branch": task.latest_proof_branch_fresh,
        "dirty": task.latest_proof_dirty_fresh, "output": task.latest_proof_output_fresh,
        "artifact": task.latest_proof_artifact_fresh,
    });
    let proof_satisfied = task.latest_proof_state.as_deref() == Some("pass")
        && task.latest_proof_head_fresh == Some(true)
        && task.latest_proof_branch_fresh == Some(true)
        && task.latest_proof_dirty_fresh == Some(true)
        && task.latest_proof_output_fresh == Some(true)
        && task.latest_proof_artifact_fresh != Some(false);
    let approvals_satisfied = required_approvals
        .iter()
        .all(|gate| approval_gate_names.contains(&normalize_token(gate)));
    let unresolved_friction = friction
        .iter()
        .filter(|item| {
            item.get("disposition").and_then(serde_json::Value::as_str) != Some("not-friction")
                && !matches!(
                    item.get("status").and_then(serde_json::Value::as_str),
                    Some("validated" | "ineffective" | "reverted")
                )
        })
        .cloned()
        .collect::<Vec<_>>();
    let friction_satisfied = unresolved_friction.is_empty();
    let staged_capsules = staged_capsules_for_task(&task.worktree, &task.id);
    let capsule_candidates = capsule_candidates_for_task(&task.worktree, &task.id);
    let capsule_exists = task
        .capsule_path
        .as_deref()
        .is_some_and(|path| std::path::Path::new(&task.worktree).join(path).is_file());
    let effective_capsule_path = task
        .capsule_path
        .clone()
        .or_else(|| capsule_candidates.first().cloned());
    let capsule_state = if task.status == "completed" && capsule_exists {
        "final"
    } else if !staged_capsules.is_empty() {
        "staged"
    } else if !capsule_candidates.is_empty() {
        "orphaned"
    } else if task.capsule_required {
        "missing"
    } else {
        "not_required"
    };
    let capsule_satisfied =
        !task.capsule_required || capsule_exists || capsule_candidates.len() == 1;
    let qualifying_trace_ids = traces
        .iter()
        .filter_map(|trace| trace.get("id").and_then(serde_json::Value::as_str))
        .filter_map(|id| id.parse::<i64>().ok())
        .filter(|id| {
            service
                .score_trace(Some(*id))
                .is_ok_and(|score| score.meets_requirement)
        })
        .collect::<Vec<_>>();
    let trace_satisfied = qualifying_trace_ids.len() == 1;

    let owner = task.owner.as_deref().unwrap_or("<OWNER>");
    let session = task.session_id.as_deref().unwrap_or("<SESSION>");
    let mut remediation = Vec::new();
    let mut gates = Vec::new();
    let mut add_gate = |name: &str, required: bool, satisfied: bool, commands: Vec<String>| {
        if required && !satisfied {
            remediation.extend(commands.iter().cloned());
        }
        gates.push(serde_json::json!({
                "gate": name, "required": required,
                "state": if !required { "not_applicable" } else if satisfied { "satisfied" } else { "unmet" },
                "remediation": if required && !satisfied { commands } else { Vec::<String>::new() },
            }));
    };
    add_gate(
        "story",
        story_required,
        story_satisfied,
        vec![format!(
            "_harness/bin/harness-cli task link-story --id {} --story <STORY> --role primary --owner {} --session {} --json",
            task.id, owner, session
        )],
    );
    let context_commands = task
        .context_manifest
        .get("must_read")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.get("path").and_then(serde_json::Value::as_str))
        .filter(|path| !task.context_acknowledged_paths.iter().any(|read| read == path))
        .map(|path| format!(
            "_harness/bin/harness-cli task context acknowledge --id {} --read '{}' --actor {} --json",
            task.id, path.replace('\'', "'\\''"), owner
        ))
        .collect::<Vec<_>>();
    add_gate(
        "context",
        task.context_required > 0,
        context_satisfied,
        context_commands,
    );
    let approval_commands = required_approvals
        .iter()
        .filter(|gate| !approval_gate_names.contains(&normalize_token(gate)))
        .map(|gate| format!(
            "_harness/bin/harness-cli task approve --id {} --gate {} --source <SOURCE> --evidence <EVIDENCE> --scope <SCOPE> --json",
            task.id, gate
        ))
        .collect::<Vec<_>>();
    add_gate(
        "approval",
        !required_approvals.is_empty(),
        approvals_satisfied,
        approval_commands,
    );
    add_gate(
        "proof",
        true,
        proof_satisfied,
        lane_policy
            .proof
            .iter()
            .map(|layer| {
                format!(
                    "_harness/bin/harness-cli proof run --task {} --layer {} -- <COMMAND>",
                    task.id, layer
                )
            })
            .collect(),
    );
    add_gate(
        "friction",
        true,
        friction_satisfied,
        unresolved_friction
            .iter()
            .filter_map(|item| item.get("fingerprint").and_then(serde_json::Value::as_str))
            .map(|fingerprint| format!(
                "_harness/bin/harness-cli friction resolve --fingerprint {} --status validated --actual-outcome <OUTCOME> --json",
                fingerprint
            ))
            .collect(),
    );
    let trace_commands = if qualifying_trace_ids.is_empty() {
        vec![format!(
            "_harness/bin/harness-cli trace --summary '<SUMMARY>' --intake <INTAKE_ID> --story {} --agent {} --outcome completed --actions '<ACTIONS>' --read '<FILES>' --changed '<FILES>' --decisions '<DECISIONS>' --errors '<ERRORS>' --notes '<NOTES>'",
            task.story_id.as_deref().unwrap_or("<STORY>"), owner
        )]
    } else {
        vec![format!(
            "_harness/bin/harness-cli task finish --id {} --owner {} --session {} --trace <ONE_OF:{}> --outcome completed --friction none{} --json",
            task.id,
            owner,
            session,
            qualifying_trace_ids
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(","),
            effective_capsule_path
                .as_deref()
                .map(|path| format!(" --capsule {path}"))
                .unwrap_or_default()
        )]
    };
    add_gate("trace", true, trace_satisfied, trace_commands);
    add_gate(
        "capsule",
        task.capsule_required,
        capsule_satisfied,
        vec![format!(
            "_harness/bin/harness-cli memory capsule render --id {} --date \"$(date +%F)\" --lane {} --outcome completed --summary '<SUMMARY>' --json",
            task.id, task.risk_lane.replace('_', "-")
        )],
    );
    if task.status == "in_progress" && remediation.is_empty() {
        remediation.push(format!(
            "_harness/bin/harness-cli task finish --id {} --owner {} --session {} --outcome completed --friction none{} --json",
            task.id,
            owner,
            session,
            effective_capsule_path
                .as_deref()
                .map(|path| format!(" --capsule {path}"))
                .unwrap_or_default()
        ));
    }
    let mut context = task.context_manifest.clone();
    if let Some(object) = context.as_object_mut() {
        object.insert(
            "required".to_owned(),
            serde_json::json!(task.context_required),
        );
        object.insert(
            "acknowledged".to_owned(),
            serde_json::json!(task.context_acknowledged),
        );
        object.insert(
            "acknowledged_paths".to_owned(),
            serde_json::json!(task.context_acknowledged_paths),
        );
    }
    Ok(serde_json::json!({
        "ok": true, "task_id": task.id, "status": task.status, "lane": task.risk_lane,
        "summary": task.summary, "input_type": task.input_type,
        "behavior_bearing": task.behavior_bearing,
        "ownership": {"owner": task.owner, "state": if task.owner.is_some() { "assigned" } else { "unowned" }},
        "session": {"id": task.session_id}, "worktree": task.worktree,
        "lease": {"expires_at": task.lease_expires_at, "state": task.lease_state},
        "transitions": task.allowed_next,
        "links": {
            "stories": stories, "decisions": [], "approvals": approvals,
            "backlog_items": [], "traces": traces,
        },
        "friction": {"all": friction, "unresolved": unresolved_friction},
        "gates": gates,
        "proof": {
            "runs": task.proof_runs, "required_layers": lane_policy.proof,
            "latest_state": task.latest_proof_state, "layers": proof_layers,
            "freshness": proof_freshness,
        },
        "context": context,
        "capsule": {
            "required": task.capsule_required, "state": capsule_state,
            "path": effective_capsule_path, "checksum": task.capsule_checksum,
            "omission_reason": task.capsule_omission_reason,
            "staged": staged_capsules, "candidates": capsule_candidates,
            "orphaned": (task.capsule_path.is_some() && !capsule_exists)
                || (task.capsule_path.is_none() && effective_capsule_path.is_some()),
        },
        "remediation": remediation,
        "next_command": remediation.first(),
    }))
}

fn query_table_objects(table: crate::application::QueryTable) -> Vec<serde_json::Value> {
    table
        .rows
        .into_iter()
        .map(|row| {
            serde_json::Value::Object(
                table
                    .headers
                    .iter()
                    .cloned()
                    .zip(row)
                    .map(|(key, value)| (key, serde_json::Value::String(value)))
                    .collect(),
            )
        })
        .collect()
}

fn staged_capsules_for_task(worktree: &str, task_id: &str) -> Vec<String> {
    fn visit(
        root: &std::path::Path,
        current: &std::path::Path,
        task_id: &str,
        out: &mut Vec<String>,
    ) {
        let Ok(entries) = fs::read_dir(current) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                visit(root, &path, task_id, out);
            } else if file_type.is_file()
                && path.extension().is_some_and(|extension| extension == "tmp")
                && path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().contains(task_id))
            {
                out.push(
                    path.strip_prefix(root)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
    }
    let root = std::path::Path::new(worktree).join("docs/tasks");
    let mut paths = Vec::new();
    visit(&root, &root, task_id, &mut paths);
    paths.sort();
    paths
}

fn capsule_candidates_for_task(worktree: &str, task_id: &str) -> Vec<String> {
    fn visit(
        root: &std::path::Path,
        current: &std::path::Path,
        task_id: &str,
        out: &mut Vec<String>,
    ) {
        let Ok(entries) = fs::read_dir(current) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                visit(root, &path, task_id, out);
            } else if file_type.is_file()
                && path.extension().is_some_and(|extension| extension == "md")
                && path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().contains(task_id))
            {
                out.push(
                    path.strip_prefix(worktree_root(root))
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
    }
    fn worktree_root(tasks_root: &std::path::Path) -> &std::path::Path {
        tasks_root
            .parent()
            .and_then(std::path::Path::parent)
            .unwrap_or(tasks_root)
    }
    let root = std::path::Path::new(worktree).join("docs/tasks");
    let mut paths = Vec::new();
    visit(&root, &root, task_id, &mut paths);
    paths.sort();
    paths
}

fn print_task_status_contract_human(contract: &serde_json::Value) {
    for key in [
        "task_id",
        "status",
        "lane",
        "summary",
        "input_type",
        "behavior_bearing",
        "ownership",
        "session",
        "worktree",
        "lease",
        "transitions",
        "links",
        "friction",
        "gates",
        "proof",
        "context",
        "capsule",
        "remediation",
        "next_command",
    ] {
        println!(
            "{key}: {}",
            contract.get(key).unwrap_or(&serde_json::Value::Null)
        );
    }
}

fn print_task_transition(task: &crate::application::TaskStatusRecord, json: bool) {
    if json {
        println!(
            "{{\"ok\":true,\"task_id\":\"{}\",\"status\":\"{}\"}}",
            json_escape(&task.id),
            json_escape(&task.status)
        );
    } else {
        println!("Task {} is now {}.", task.id, task.status);
    }
}

fn print_init_result(result: InitResult) {
    match result {
        InitResult::Created { db_path } => {
            println!("Creating harness database at {}", db_path.display());
            println!("Schema applied.");
        }
        InitResult::Existing { db_path, version } => {
            println!("Database already exists at {}", db_path.display());
            println!("Current schema version: {version}");
        }
        InitResult::MigratedExisting { db_path } => {
            println!("Database already exists at {}", db_path.display());
            println!("No schema version found. Applying schema.");
            println!("Schema applied.");
        }
    }
}

fn print_migrate_result(result: MigrateResult) {
    println!("Current schema version: {}", result.current_version);
    if result.applied.is_empty() {
        println!("Already up to date.");
    } else {
        for version in &result.applied {
            println!("Applying migration {version}...");
        }
        println!("Applied {} migration(s).", result.applied.len());
    }
}

fn workflow_parity(
    repo_root: &std::path::Path,
    policy: &crate::infrastructure::WorkflowPolicy,
    commands: &[String],
) -> Result<PolicyParityResult, InterfaceError> {
    let fixture_path = repo_root.join("_harness/tests/policy-parity-cases.toml");
    let fixture_text = fs::read_to_string(&fixture_path).map_err(|error| {
        InterfaceError::WorkflowParity(format!("cannot read {}: {error}", fixture_path.display()))
    })?;
    let fixture: PolicyParityFixture = toml::from_str(&fixture_text).map_err(|error| {
        InterfaceError::WorkflowParity(format!("cannot parse {}: {error}", fixture_path.display()))
    })?;
    if fixture.schema_version != 1 {
        return Err(InterfaceError::WorkflowParity(format!(
            "unsupported fixture schema version {}",
            fixture.schema_version
        )));
    }

    let mut result = PolicyParityResult::default();
    let source_policy = repo_root.join(&fixture.source_policy);
    if source_policy.is_file() {
        result
            .checked
            .push(format!("source policy {}", fixture.source_policy));
    } else {
        result
            .failures
            .push(format!("missing source policy {}", fixture.source_policy));
    }
    if policy.mode == "shadow" {
        result.checked.push("workflow mode shadow".to_owned());
    } else {
        result
            .failures
            .push(format!("workflow mode is {}, expected shadow", policy.mode));
    }

    let tracked_manifest_path = repo_root.join("_harness/command-manifest.txt");
    match fs::read_to_string(&tracked_manifest_path) {
        Ok(contents) => {
            let tracked = contents
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(str::to_owned)
                .collect::<Vec<_>>();
            if tracked == commands {
                result.checked.push("compiled command manifest".to_owned());
            } else {
                result.failures.push(
                    "compiled command manifest differs from _harness/command-manifest.txt"
                        .to_owned(),
                );
            }
        }
        Err(error) => result.failures.push(format!(
            "cannot read {}: {error}",
            tracked_manifest_path.display()
        )),
    }

    for case in fixture.classification_cases {
        if case.comparison != "accepted" {
            result.failures.push(format!(
                "classification case {} has unsupported comparison {}",
                case.id, case.comparison
            ));
            continue;
        }
        let (actual_lane, _) = policy.classify(&case.flags);
        if actual_lane == case.expected_lane {
            result.checked.push(format!("classification:{}", case.id));
        } else {
            result.failures.push(format!(
                "classification:{} expected {}, got {}",
                case.id, case.expected_lane, actual_lane
            ));
        }
    }

    for case in fixture.context_cases {
        let manifest =
            policy.context_manifest(&case.lane, &case.phase, &case.paths, &case.flags, &[]);
        let contains = |entries: &[crate::infrastructure::WorkflowContextEntry], expected: &str| {
            entries.iter().any(|entry| entry.path.starts_with(expected))
        };
        let mut missing = Vec::new();
        for expected in &case.must_include {
            if !contains(&manifest.must_read, expected) {
                missing.push(format!("must_read {expected}"));
            }
        }
        for expected in &case.should_include {
            if !contains(&manifest.should_read, expected) {
                missing.push(format!("should_read {expected}"));
            }
        }
        for expected in &case.skip_include {
            if !contains(&manifest.skip, expected) {
                missing.push(format!("skip {expected}"));
            }
        }
        if missing.is_empty() {
            result.checked.push(format!("context:{}", case.id));
        } else {
            result.failures.push(format!(
                "context:{} missing {}",
                case.id,
                missing.join(", ")
            ));
        }
    }

    for delta in fixture.intentional_deltas {
        let decision_path = repo_root
            .join("docs/decisions")
            .join(format!("{}.md", delta.decision));
        match fs::read_to_string(&decision_path) {
            Ok(contents) if contents.contains("## Status\n\nAccepted") => {
                result.deltas.push(format!(
                    "{}: {} ({}; {})",
                    delta.id, delta.shadow_behavior, delta.decision, delta.disposition
                ))
            }
            Ok(_) => result.failures.push(format!(
                "intentional delta:{} decision {} is not accepted",
                delta.id, delta.decision
            )),
            Err(_) => result.failures.push(format!(
                "intentional delta:{} missing decision {}",
                delta.id, delta.decision
            )),
        }
        if delta.current_markdown.trim().is_empty() {
            result.failures.push(format!(
                "intentional delta:{} has empty current_markdown",
                delta.id
            ));
        }
    }
    Ok(result)
}

#[derive(Debug)]
struct ArtifactCheckResult {
    checked: Vec<String>,
    legacy: Vec<String>,
    errors: Vec<String>,
    artifacts: Vec<ArtifactRecord>,
}

#[derive(Clone, Debug)]
struct ArtifactRecord {
    kind: String,
    id: String,
    path: String,
    checksum: String,
    schema: String,
    status: String,
    title: String,
    lane: Option<String>,
    content: String,
    components: Vec<String>,
    capsule: Option<CapsuleProjection>,
}

#[derive(Clone, Debug)]
struct CapsuleProjection {
    date: String,
    summary: String,
    story_ids_json: String,
    trace_ids_json: String,
    proof_summaries_json: String,
    unknown_fields_json: String,
    content_checksum: String,
}

fn project_artifact_index(
    database: &std::path::Path,
    artifacts: &[ArtifactRecord],
    preserve_operational: bool,
) -> Result<usize, InterfaceError> {
    let mut connection =
        Connection::open(database).map_err(crate::infrastructure::HarnessInfraError::from)?;
    let transaction = connection
        .transaction()
        .map_err(crate::infrastructure::HarnessInfraError::from)?;
    transaction
        .execute("DELETE FROM artifact_index;", [])
        .map_err(crate::infrastructure::HarnessInfraError::from)?;
    transaction
        .execute("DELETE FROM portable_task_summary;", [])
        .map_err(crate::infrastructure::HarnessInfraError::from)?;
    for artifact in artifacts {
        transaction.execute(
            "INSERT INTO artifact_index(artifact_type, artifact_id, path, checksum, schema_version, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                artifact.kind,
                artifact.id,
                artifact.path,
                artifact.checksum,
                artifact.schema,
                artifact.status
            ],
        ).map_err(crate::infrastructure::HarnessInfraError::from)?;
        match artifact.kind.as_str() {
            "story" => {
                let lane = artifact
                    .lane
                    .clone()
                    .unwrap_or_else(|| "high_risk".to_owned());
                let story_status = match artifact.status.as_str() {
                    "completed" => "implemented",
                    "ready" => "planned",
                    value
                        if matches!(
                            value,
                            "planned" | "in_progress" | "implemented" | "changed" | "retired"
                        ) =>
                    {
                        value
                    }
                    _ => "planned",
                };
                let evidence = artifact
                    .content
                    .split("## Evidence")
                    .nth(1)
                    .map(|body| body.split("\n## ").next().unwrap_or(body).trim())
                    .filter(|body| !body.is_empty());
                let sql = if preserve_operational {
                    "INSERT INTO story(id, title, risk_lane, status, evidence, notes)
                     VALUES (?1, ?2, ?3, ?4, ?5, 'Refreshed from canonical artifact by memory rebuild apply.')
                     ON CONFLICT(id) DO UPDATE SET title=excluded.title, risk_lane=excluded.risk_lane,
                       status=excluded.status, evidence=excluded.evidence"
                } else {
                    "INSERT INTO story(id, title, risk_lane, status, evidence, notes)
                     VALUES (?1, ?2, ?3, ?4, ?5, 'Rebuilt from canonical artifact by memory rebuild dry-run.')"
                };
                transaction
                    .execute(
                        sql,
                        params![artifact.id, artifact.title, lane, story_status, evidence],
                    )
                    .map_err(crate::infrastructure::HarnessInfraError::from)?;
            }
            "decision" => {
                let decision_status = match artifact.status.as_str() {
                    value
                        if matches!(value, "proposed" | "accepted" | "superseded" | "rejected") =>
                    {
                        value
                    }
                    _ => "proposed",
                };
                let sql = if preserve_operational {
                    "INSERT INTO decision(id, title, status, doc_path, notes)
                     VALUES (?1, ?2, ?3, ?4, 'Refreshed from canonical artifact by memory rebuild apply.')
                     ON CONFLICT(id) DO UPDATE SET title=excluded.title, status=excluded.status,
                       doc_path=excluded.doc_path"
                } else {
                    "INSERT INTO decision(id, title, status, doc_path, notes)
                     VALUES (?1, ?2, ?3, ?4, 'Rebuilt from canonical artifact by memory rebuild dry-run.')"
                };
                transaction
                    .execute(
                        sql,
                        params![artifact.id, artifact.title, decision_status, artifact.path],
                    )
                    .map_err(crate::infrastructure::HarnessInfraError::from)?;
            }
            "capsule" => {
                let capsule = artifact.capsule.as_ref().ok_or_else(|| {
                    InterfaceError::WorkflowParity(format!(
                        "{}: validated capsule projection is missing",
                        artifact.path
                    ))
                })?;
                transaction
                    .execute(
                        "INSERT INTO portable_task_summary(
                        task_id, capsule_path, capsule_schema, task_date, risk_lane,
                        outcome, summary, story_ids_json, trace_ids_json,
                        proof_summaries_json, unknown_fields_json, content_checksum
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                        params![
                            artifact.id,
                            artifact.path,
                            artifact.schema,
                            capsule.date,
                            artifact.lane.as_deref().unwrap_or("high_risk"),
                            artifact.status,
                            capsule.summary,
                            capsule.story_ids_json,
                            capsule.trace_ids_json,
                            capsule.proof_summaries_json,
                            capsule.unknown_fields_json,
                            capsule.content_checksum,
                        ],
                    )
                    .map_err(crate::infrastructure::HarnessInfraError::from)?;
            }
            _ => {}
        }
    }
    let count = artifacts.len();
    transaction
        .commit()
        .map_err(crate::infrastructure::HarnessInfraError::from)?;
    Ok(count)
}

fn candidate_schema_version(database: &std::path::Path) -> Result<i64, InterfaceError> {
    let connection =
        Connection::open(database).map_err(crate::infrastructure::HarnessInfraError::from)?;
    connection
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .map_err(crate::infrastructure::HarnessInfraError::from)
        .map_err(InterfaceError::from)
}

fn rebuild_parity_report(
    database: &std::path::Path,
    artifacts: &[ArtifactRecord],
) -> Result<serde_json::Value, InterfaceError> {
    let connection =
        Connection::open(database).map_err(crate::infrastructure::HarnessInfraError::from)?;
    let mut projected = std::collections::BTreeMap::new();
    let mut statement = connection
        .prepare(
            "SELECT artifact_type, artifact_id, path, checksum, schema_version, status
             FROM artifact_index ORDER BY artifact_type, artifact_id",
        )
        .map_err(crate::infrastructure::HarnessInfraError::from)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .map_err(crate::infrastructure::HarnessInfraError::from)?;
    for row in rows {
        let (kind, id, path, checksum, schema, status) =
            row.map_err(crate::infrastructure::HarnessInfraError::from)?;
        projected.insert((kind, id), (path, checksum, schema, status));
    }
    let mut mismatches = Vec::new();
    for artifact in artifacts {
        let key = (artifact.kind.clone(), artifact.id.clone());
        let expected = (
            artifact.path.clone(),
            artifact.checksum.clone(),
            artifact.schema.clone(),
            artifact.status.clone(),
        );
        match projected.remove(&key) {
            Some(actual) if actual == expected => {}
            Some(actual) => mismatches.push(serde_json::json!({
                "kind": artifact.kind, "id": artifact.id,
                "expected": expected, "actual": actual
            })),
            None => mismatches.push(serde_json::json!({
                "kind": artifact.kind, "id": artifact.id, "error": "missing projection"
            })),
        }
    }
    for ((kind, id), actual) in projected {
        mismatches.push(serde_json::json!({
            "kind": kind, "id": id, "actual": actual, "error": "unexpected projection"
        }));
    }
    let source_counts = ["story", "decision", "capsule"]
        .into_iter()
        .map(|kind| {
            (
                kind,
                artifacts
                    .iter()
                    .filter(|artifact| artifact.kind == kind)
                    .count(),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let portable_capsules: i64 = connection
        .query_row("SELECT COUNT(*) FROM portable_task_summary", [], |row| {
            row.get(0)
        })
        .map_err(crate::infrastructure::HarnessInfraError::from)?;
    Ok(serde_json::json!({
        "check_id": "semantic-memory-parity",
        "version": 1,
        "state": if mismatches.is_empty() && portable_capsules as usize == source_counts["capsule"] { "pass" } else { "fail" },
        "schema_version": candidate_schema_version(database)?,
        "source_counts": source_counts,
        "projected_count": artifacts.len(),
        "portable_task_summaries": portable_capsules,
        "artifacts": artifacts.iter().map(|artifact| serde_json::json!({
            "kind": artifact.kind,
            "id": artifact.id,
            "path": artifact.path,
            "status": artifact.status,
            "checksum": artifact.checksum,
            "schema": artifact.schema,
            "components": artifact.components,
            "story_ids": artifact.capsule.as_ref().and_then(|capsule| serde_json::from_str::<serde_json::Value>(&capsule.story_ids_json).ok()),
            "trace_ids": artifact.capsule.as_ref().and_then(|capsule| serde_json::from_str::<serde_json::Value>(&capsule.trace_ids_json).ok()),
            "proof_summaries": artifact.capsule.as_ref().and_then(|capsule| serde_json::from_str::<serde_json::Value>(&capsule.proof_summaries_json).ok()),
            "unknown_fields": artifact.capsule.as_ref().and_then(|capsule| serde_json::from_str::<serde_json::Value>(&capsule.unknown_fields_json).ok()),
        })).collect::<Vec<_>>(),
        "mismatches": mismatches,
    }))
}

fn rebuild_logical_digest(database: &std::path::Path) -> Result<String, InterfaceError> {
    let connection =
        Connection::open(database).map_err(crate::infrastructure::HarnessInfraError::from)?;
    let mut values = Vec::new();
    for query in [
        "SELECT artifact_type || '|' || artifact_id || '|' || path || '|' || checksum || '|' || schema_version || '|' || status FROM artifact_index ORDER BY artifact_type, artifact_id",
        "SELECT id || '|' || title || '|' || risk_lane || '|' || status || '|' || COALESCE(evidence, '') FROM story ORDER BY id",
        "SELECT id || '|' || title || '|' || status || '|' || doc_path FROM decision ORDER BY id",
    ] {
        let mut statement = connection.prepare(query).map_err(crate::infrastructure::HarnessInfraError::from)?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0)).map_err(crate::infrastructure::HarnessInfraError::from)?;
        for row in rows { values.push(row.map_err(crate::infrastructure::HarnessInfraError::from)?); }
    }
    Ok(format!(
        "{:x}",
        Sha256::digest(values.join("\n").as_bytes())
    ))
}

fn rebuild_apply_state_allowed(code: &str, recover_foreign: bool) -> bool {
    matches!(code, "HEALTHY" | "DB_MISSING")
        || (recover_foreign && matches!(code, "DB_UNHEALTHY" | "DB_AHEAD_OF_SOURCE"))
}

fn checkpoint_rebuild_database(database: &std::path::Path) -> Result<(), InterfaceError> {
    let connection =
        Connection::open(database).map_err(crate::infrastructure::HarnessInfraError::from)?;
    connection
        .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(crate::infrastructure::HarnessInfraError::from)?;
    Ok(())
}

fn render_capsule(
    repo_root: &std::path::Path,
    database: &std::path::Path,
    id: &str,
    date: &str,
    lane: &str,
    outcome: &str,
    summary: &str,
) -> Result<String, InterfaceError> {
    if id.is_empty()
        || !id
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || value == '-')
        || !date
            .chars()
            .all(|value| value.is_ascii_digit() || value == '-')
        || !matches!(lane, "tiny" | "normal" | "high-risk" | "high_risk")
        || outcome.is_empty()
    {
        return Err(InterfaceError::WorkflowParity(
            "invalid capsule id, date, lane, or outcome".to_owned(),
        ));
    }
    let redacted = redact_capsule_text(summary);
    let body = format!("# Outcome\n\n{}\n", redacted);
    let checksum = format!("{:x}", Sha256::digest(body.as_bytes()));
    let connection =
        Connection::open(database).map_err(crate::infrastructure::HarnessInfraError::from)?;
    let task_exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM task WHERE id=?1)",
            params![id],
            |row| row.get(0),
        )
        .map_err(crate::infrastructure::HarnessInfraError::from)?;
    if !task_exists {
        return Err(InterfaceError::WorkflowParity(format!(
            "cannot render portable capsule for unknown task {id}"
        )));
    }
    let collect_strings = |sql: &str| -> Result<Vec<String>, InterfaceError> {
        let mut statement = connection
            .prepare(sql)
            .map_err(crate::infrastructure::HarnessInfraError::from)?;
        let rows = statement
            .query_map(params![id], |row| row.get::<_, String>(0))
            .map_err(crate::infrastructure::HarnessInfraError::from)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(crate::infrastructure::HarnessInfraError::from)
            .map_err(InterfaceError::from)
    };
    let story_ids =
        collect_strings("SELECT story_id FROM task_story WHERE task_id=?1 ORDER BY story_id")?;
    let trace_ids = collect_strings(
        "SELECT CAST(trace.id AS TEXT) FROM trace
         JOIN task ON task.intake_id=trace.intake_id
         WHERE task.id=?1 ORDER BY trace.id",
    )?
    .into_iter()
    .filter_map(|value| value.parse::<i64>().ok())
    .collect::<Vec<_>>();
    let mut proof_statement = connection
        .prepare(
            "SELECT layer, state, head_commit, branch, dirty_fingerprint,
                    stdout_hash, stderr_hash, artifact_path, artifact_hash
             FROM proof_run WHERE task_id=?1 ORDER BY id",
        )
        .map_err(crate::infrastructure::HarnessInfraError::from)?;
    let proof_rows = proof_statement
        .query_map(params![id], |row| {
            Ok(serde_json::json!({
                "layer": row.get::<_, String>(0)?,
                "state": row.get::<_, String>(1)?,
                "head_commit": row.get::<_, Option<String>>(2)?,
                "branch": row.get::<_, Option<String>>(3)?,
                "dirty_fingerprint": row.get::<_, Option<String>>(4)?,
                "stdout_hash": row.get::<_, Option<String>>(5)?,
                "stderr_hash": row.get::<_, Option<String>>(6)?,
                "artifact_path": row.get::<_, Option<String>>(7)?,
                "artifact_hash": row.get::<_, Option<String>>(8)?,
            }))
        })
        .map_err(crate::infrastructure::HarnessInfraError::from)?;
    let proof_summaries = proof_rows
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(crate::infrastructure::HarnessInfraError::from)?;
    let story_ids_json = serde_json::to_string(&story_ids)
        .map_err(|error| InterfaceError::WorkflowParity(error.to_string()))?;
    let trace_ids_json = serde_json::to_string(&trace_ids)
        .map_err(|error| InterfaceError::WorkflowParity(error.to_string()))?;
    let proof_summaries_json = serde_json::to_string(&proof_summaries)
        .map_err(|error| InterfaceError::WorkflowParity(error.to_string()))?;
    let month = date.get(0..7).ok_or_else(|| {
        InterfaceError::WorkflowParity("capsule date must be YYYY-MM-DD".to_owned())
    })?;
    let directory = repo_root
        .join("docs/tasks")
        .join(&date[..4])
        .join(&month[5..]);
    let slug = redacted
        .to_lowercase()
        .chars()
        .map(|value| {
            if value.is_ascii_alphanumeric() {
                value
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .take(6)
        .collect::<Vec<_>>()
        .join("-");
    let path = directory.join(format!(
        "{}-{}.md",
        id,
        if slug.is_empty() { "task" } else { &slug }
    ));
    if path.exists() {
        return Err(InterfaceError::WorkflowParity(format!(
            "capsule already exists at {}",
            path.display()
        )));
    }
    fs::create_dir_all(&directory).map_err(crate::infrastructure::HarnessInfraError::from)?;
    let content = format!(
        "---\nschema: harness/task-capsule/v2\ntask_id: {id}\ndate: {date}\nlane: {}\noutcome: {outcome}\nstory_ids: {story_ids_json}\ntrace_ids: {trace_ids_json}\nproof_summaries: {proof_summaries_json}\nunknown_fields: []\ncontent_checksum: sha256:{checksum}\n---\n{body}",
        lane.replace('-', "_")
    );
    let temporary = directory.join(format!(".{}-{}.tmp", id, std::process::id()));
    fs::write(&temporary, content).map_err(crate::infrastructure::HarnessInfraError::from)?;
    fs::rename(&temporary, &path).map_err(crate::infrastructure::HarnessInfraError::from)?;
    Ok(path
        .strip_prefix(repo_root)
        .unwrap_or(&path)
        .to_string_lossy()
        .into_owned())
}

fn redact_capsule_text(value: &str) -> String {
    let mut redact_next = false;
    value
        .split_whitespace()
        .map(|word| {
            let lower = word.to_lowercase();
            let sensitive_key =
                lower.contains("password") || lower.contains("token") || lower.contains("secret");
            let redact =
                redact_next || sensitive_key || word.starts_with('/') || word.starts_with("C:\\");
            redact_next = sensitive_key;
            if redact {
                "[redacted]".to_owned()
            } else {
                word.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn capsule_check(repo_root: &std::path::Path) -> Result<Vec<String>, InterfaceError> {
    Ok(artifact_check(repo_root, Some("capsule"), None).errors)
}

fn artifact_frontmatter(
    content: &str,
) -> Option<(std::collections::BTreeMap<String, String>, &str)> {
    let (head, body) = content.strip_prefix("---\n")?.split_once("\n---\n")?;
    let fields = head
        .lines()
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| {
            (
                key.trim().to_owned(),
                value.trim().trim_matches('"').to_owned(),
            )
        })
        .collect();
    Some((fields, body))
}

fn markdown_heading(content: &str) -> Option<&str> {
    content
        .lines()
        .find_map(|line| line.strip_prefix("# "))
        .map(str::trim)
}

fn markdown_status(content: &str) -> Option<String> {
    content
        .lines()
        .find_map(|line| line.strip_prefix("Status:").map(str::trim))
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .or_else(|| {
            content
                .split("## Status")
                .nth(1)
                .and_then(|rest| rest.lines().map(str::trim).find(|line| !line.is_empty()))
                .map(str::to_owned)
        })
}

fn markdown_lane(content: &str) -> Option<String> {
    content
        .lines()
        .find_map(|line| {
            line.trim()
                .trim_start_matches("- ")
                .strip_prefix("Lane:")
                .map(str::trim)
        })
        .filter(|value| !value.is_empty())
        .map(|value| value.replace('-', "_"))
}

fn aggregate_artifact_checksum(
    repo_root: &std::path::Path,
    components: &[String],
) -> Result<String, String> {
    let mut entries = Vec::new();
    for component in components {
        let bytes =
            fs::read(repo_root.join(component)).map_err(|error| format!("{component}: {error}"))?;
        entries.push(format!(
            "{component}\0{:x}",
            Sha256::digest(bytes.as_slice())
        ));
    }
    Ok(format!(
        "{:x}",
        Sha256::digest(entries.join("\n").as_bytes())
    ))
}

fn parse_artifact_record(
    repo_root: &std::path::Path,
    kind: &str,
    path: String,
    components: Vec<String>,
    packet: bool,
) -> Result<(ArtifactRecord, bool), String> {
    let content =
        fs::read_to_string(repo_root.join(&path)).map_err(|error| format!("{path}: {error}"))?;
    let parsed = artifact_frontmatter(&content);
    let fields = parsed.as_ref().map(|(fields, _)| fields);
    let body = parsed.as_ref().map(|(_, body)| *body).unwrap_or(&content);
    let id_key = if kind == "capsule" { "task_id" } else { "id" };
    let id = fields
        .and_then(|fields| fields.get(id_key).cloned())
        .or_else(|| {
            markdown_heading(&content)
                .and_then(|heading| heading.split_whitespace().next())
                .map(str::to_owned)
        })
        .ok_or_else(|| format!("{path}: artifact id is missing"))?;
    if id.is_empty() || id.contains('/') || id.contains('\\') || id == "." || id == ".." {
        return Err(format!("{path}: unsafe or empty {kind} id"));
    }
    let status = fields
        .and_then(|fields| {
            fields
                .get(if kind == "capsule" {
                    "outcome"
                } else {
                    "status"
                })
                .cloned()
        })
        .or_else(|| markdown_status(&content))
        .ok_or_else(|| format!("{path}: {kind} status is missing"))?;
    let lane = fields
        .and_then(|fields| fields.get("lane").cloned())
        .or_else(|| markdown_lane(&content))
        .map(|lane| lane.replace('-', "_"));
    if lane
        .as_deref()
        .is_some_and(|lane| !matches!(lane, "tiny" | "normal" | "high_risk"))
    {
        return Err(format!("{path}: invalid {kind} lane"));
    }
    let heading = markdown_heading(&content).unwrap_or(&id);
    let mut title = fields
        .and_then(|fields| fields.get("title").cloned())
        .unwrap_or_else(|| {
            heading
                .strip_prefix(&id)
                .map(str::trim)
                .filter(|title| !title.is_empty())
                .unwrap_or(heading)
                .to_owned()
        });
    if packet && title.eq_ignore_ascii_case("overview") {
        if let Some(packet_name) = std::path::Path::new(&path)
            .parent()
            .and_then(std::path::Path::file_name)
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_prefix(&format!("{id}-")))
        {
            title = packet_name
                .split('-')
                .filter(|part| !part.is_empty())
                .map(|part| {
                    let mut characters = part.chars();
                    characters
                        .next()
                        .map(|first| first.to_ascii_uppercase().to_string() + characters.as_str())
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
                .join(" ");
        }
    }
    let (schema, legacy) = if packet {
        ("harness/story-packet/v1".to_owned(), parsed.is_none())
    } else if let Some(fields) = fields {
        let schema = fields.get("schema").cloned().unwrap_or_default();
        let valid_schema = match kind {
            "story" => schema == "harness/story/v1",
            "decision" => schema == "harness/decision/v1",
            "capsule" => matches!(
                schema.as_str(),
                "harness/task-capsule/v1" | "harness/task-capsule/v2"
            ),
            _ => false,
        };
        if !valid_schema {
            return Err(format!("{path}: invalid {kind} schema {schema}"));
        }
        (schema, false)
    } else {
        if kind == "capsule" || markdown_heading(&content).is_none() {
            return Err(format!(
                "{path}: legacy {kind} requires valid frontmatter or a title and Status section"
            ));
        }
        ("legacy".to_owned(), true)
    };
    if kind == "story" {
        if fields.is_some() && lane.is_none() {
            return Err(format!("{path}: story lane is missing"));
        }
        if fields.is_some() {
            let references = content
                .strip_prefix("---\n")
                .and_then(|rest| rest.split_once("\n---\n").map(|(head, _)| head))
                .into_iter()
                .flat_map(str::lines)
                .skip_while(|line| !line.starts_with("product_docs:"))
                .skip(1)
                .take_while(|line| line.trim_start().starts_with('-'))
                .filter_map(|line| line.trim().strip_prefix('-'))
                .map(str::trim)
                .collect::<Vec<_>>();
            if references.iter().any(|reference| {
                std::path::Path::new(reference).is_absolute()
                    || reference.split('/').any(|part| part == "..")
                    || !repo_root.join(reference).is_file()
            }) {
                return Err(format!("{path}: invalid story product reference"));
            }
        }
    }
    let capsule = if kind == "capsule" {
        let fields = fields.expect("capsules require frontmatter");
        for required in ["task_id", "date", "lane", "outcome", "content_checksum"] {
            if !fields.contains_key(required) {
                return Err(format!(
                    "{path}: capsule required field {required} is missing"
                ));
            }
        }
        let expected = fields["content_checksum"]
            .strip_prefix("sha256:")
            .ok_or_else(|| format!("{path}: capsule checksum prefix is invalid"))?;
        let actual = format!("{:x}", Sha256::digest(body.as_bytes()));
        if expected != actual {
            return Err(format!(
                "{path}: capsule content checksum mismatch for {id}"
            ));
        }
        let summary = body
            .strip_prefix("# Outcome\n\n")
            .unwrap_or(body)
            .trim()
            .to_owned();
        let v2 = schema == "harness/task-capsule/v2";
        let json_field = |key: &str, fallback: &str| -> Result<String, String> {
            let value = fields.get(key).map(String::as_str).unwrap_or(fallback);
            let parsed = serde_json::from_str::<serde_json::Value>(value)
                .map_err(|error| format!("{path}: capsule {key} is invalid JSON: {error}"))?;
            if !parsed.is_array() {
                return Err(format!("{path}: capsule {key} must be a JSON array"));
            }
            Ok(value.to_owned())
        };
        let unknown = if v2 {
            json_field("unknown_fields", "[]")?
        } else {
            serde_json::to_string(&["story_ids", "trace_ids", "proof_summaries"])
                .map_err(|error| error.to_string())?
        };
        Some(CapsuleProjection {
            date: fields["date"].clone(),
            summary,
            story_ids_json: if v2 {
                json_field("story_ids", "[]")?
            } else {
                "[]".to_owned()
            },
            trace_ids_json: if v2 {
                json_field("trace_ids", "[]")?
            } else {
                "[]".to_owned()
            },
            proof_summaries_json: if v2 {
                json_field("proof_summaries", "[]")?
            } else {
                "[]".to_owned()
            },
            unknown_fields_json: unknown,
            content_checksum: expected.to_owned(),
        })
    } else {
        None
    };
    let checksum = if packet {
        aggregate_artifact_checksum(repo_root, &components)?
    } else {
        format!("{:x}", Sha256::digest(content.as_bytes()))
    };
    Ok((
        ArtifactRecord {
            kind: kind.to_owned(),
            id,
            path,
            checksum,
            schema,
            status,
            title,
            lane,
            content,
            components,
            capsule,
        },
        legacy,
    ))
}

fn safe_recursive_files(
    repo_root: &std::path::Path,
    root: &std::path::Path,
    errors: &mut Vec<String>,
) -> Vec<std::path::PathBuf> {
    fn visit(
        repo_root: &std::path::Path,
        current: &std::path::Path,
        files: &mut Vec<std::path::PathBuf>,
        errors: &mut Vec<String>,
    ) {
        let mut entries = match fs::read_dir(current) {
            Ok(entries) => entries.filter_map(Result::ok).collect::<Vec<_>>(),
            Err(error) => {
                errors.push(format!("{}: {error}", current.display()));
                return;
            }
        };
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let relative = path
                .strip_prefix(repo_root)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(error) => {
                    errors.push(format!("{relative}: {error}"));
                    continue;
                }
            };
            if file_type.is_symlink() {
                errors.push(format!("unsafe symlink artifact path {relative}"));
            } else if file_type.is_dir() {
                visit(repo_root, &path, files, errors);
            } else if file_type.is_file() {
                files.push(path);
            } else {
                errors.push(format!("unsafe artifact file type {relative}"));
            }
        }
    }
    let mut files = Vec::new();
    if root.is_dir() {
        visit(repo_root, root, &mut files, errors);
    }
    files.sort();
    files
}

fn artifact_check(
    repo_root: &std::path::Path,
    only_kind: Option<&str>,
    requested_path: Option<String>,
) -> ArtifactCheckResult {
    let mut result = ArtifactCheckResult {
        checked: Vec::new(),
        legacy: Vec::new(),
        errors: Vec::new(),
        artifacts: Vec::new(),
    };
    let mut seen_ids = std::collections::BTreeMap::<(String, String), String>::new();
    let mut seen_folded_ids =
        std::collections::BTreeMap::<(String, String), (String, String)>::new();
    let mut seen_folded_paths = std::collections::BTreeMap::<String, String>::new();
    let kinds: Vec<(&str, &str)> = match only_kind {
        Some("story") => vec![("story", "docs/stories")],
        Some("decision") => vec![("decision", "docs/decisions")],
        Some("capsule") => vec![("capsule", "docs/tasks")],
        _ => vec![
            ("story", "docs/stories"),
            ("decision", "docs/decisions"),
            ("capsule", "docs/tasks"),
        ],
    };
    for (kind, directory) in kinds {
        let root = repo_root.join(directory);
        let mut candidates = Vec::<(String, Vec<String>, bool)>::new();
        if let Some(path) = requested_path.as_ref() {
            if std::path::Path::new(path).is_absolute() || path.split('/').any(|part| part == "..")
            {
                result.errors.push(format!("unsafe artifact path {path}"));
                continue;
            }
            let requested = repo_root.join(path);
            match fs::symlink_metadata(&requested) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    result
                        .errors
                        .push(format!("unsafe symlink artifact path {path}"));
                    continue;
                }
                Ok(metadata) if metadata.is_dir() && kind == "story" => {
                    let overview = requested.join("overview.md");
                    let files = safe_recursive_files(repo_root, &requested, &mut result.errors);
                    let components = files
                        .iter()
                        .filter(|file| file.extension().is_some_and(|ext| ext == "md"))
                        .map(|file| {
                            file.strip_prefix(repo_root)
                                .unwrap_or(file)
                                .to_string_lossy()
                                .into_owned()
                        })
                        .collect::<Vec<_>>();
                    candidates.push((
                        overview
                            .strip_prefix(repo_root)
                            .unwrap_or(&overview)
                            .to_string_lossy()
                            .into_owned(),
                        components,
                        true,
                    ));
                }
                Ok(metadata) if metadata.is_file() => {
                    candidates.push((path.clone(), vec![path.clone()], false));
                }
                Ok(_) => result
                    .errors
                    .push(format!("unsafe artifact file type {path}")),
                Err(error) => result.errors.push(format!("{path}: {error}")),
            }
        } else {
            let files = safe_recursive_files(repo_root, &root, &mut result.errors);
            match kind {
                "story" => {
                    let mut packet_files = std::collections::BTreeMap::<String, Vec<String>>::new();
                    for file in files {
                        let relative = file
                            .strip_prefix(repo_root)
                            .unwrap_or(&file)
                            .to_string_lossy()
                            .into_owned();
                        let within = file.strip_prefix(&root).unwrap_or(&file);
                        if within.components().count() == 1 {
                            if file.file_name().is_some_and(|name| name == "README.md") {
                                continue;
                            }
                            if file.extension().is_some_and(|ext| ext == "md") {
                                candidates.push((relative.clone(), vec![relative], false));
                            } else {
                                result
                                    .errors
                                    .push(format!("unsafe story artifact file {relative}"));
                            }
                        } else {
                            let packet = within
                                .components()
                                .next()
                                .unwrap()
                                .as_os_str()
                                .to_string_lossy()
                                .into_owned();
                            packet_files.entry(packet).or_default().push(relative);
                        }
                    }
                    for (_packet, mut components) in packet_files {
                        components.sort();
                        let allowed = ["overview.md", "design.md", "execplan.md", "validation.md"];
                        for component in &components {
                            let name = std::path::Path::new(component)
                                .file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or("");
                            if !allowed.contains(&name) {
                                result.errors.push(format!(
                                    "unsupported story packet component {component}"
                                ));
                            }
                        }
                        let overview = components
                            .iter()
                            .find(|path| path.ends_with("/overview.md"))
                            .cloned();
                        match overview {
                            Some(overview) => candidates.push((overview, components, true)),
                            None => result.errors.push(format!(
                                "story packet is missing canonical overview.md: {}",
                                components.first().cloned().unwrap_or_default()
                            )),
                        }
                    }
                }
                _ => {
                    for file in files {
                        let relative = file
                            .strip_prefix(repo_root)
                            .unwrap_or(&file)
                            .to_string_lossy()
                            .into_owned();
                        if file.file_name().is_some_and(|name| name == "README.md") {
                            continue;
                        }
                        if file.extension().is_some_and(|ext| ext == "md") {
                            candidates.push((relative.clone(), vec![relative], false));
                        } else {
                            result
                                .errors
                                .push(format!("unsafe {kind} artifact file {relative}"));
                        }
                    }
                }
            }
        }
        candidates.sort_by(|left, right| left.0.cmp(&right.0));
        for (path, components, packet) in candidates {
            let folded_path = path.to_ascii_lowercase();
            if let Some(existing) = seen_folded_paths.insert(folded_path, path.clone()) {
                if existing != path {
                    result.errors.push(format!(
                        "case-colliding artifact paths: {existing} and {path}"
                    ));
                }
            }
            match parse_artifact_record(repo_root, kind, path.clone(), components, packet) {
                Err(error) => result.errors.push(error),
                Ok((artifact, legacy)) => {
                    let key = (kind.to_owned(), artifact.id.clone());
                    if let Some(existing) = seen_ids.insert(key, path.clone()) {
                        result.errors.push(format!(
                            "duplicate {kind} id {}: {existing} and {path}",
                            artifact.id
                        ));
                    }
                    let folded_key = (kind.to_owned(), artifact.id.to_ascii_lowercase());
                    if let Some((existing_id, existing_path)) =
                        seen_folded_ids.insert(folded_key, (artifact.id.clone(), path.clone()))
                    {
                        if existing_id != artifact.id {
                            result.errors.push(format!(
                                "case-colliding {kind} ids {existing_id} and {}: {existing_path} and {path}",
                                artifact.id
                            ));
                        }
                    }
                    result.checked.push(path.clone());
                    if legacy {
                        result.legacy.push(path);
                    }
                    result.artifacts.push(artifact);
                }
            }
        }
    }
    result.checked.sort();
    result.legacy.sort();
    result
        .artifacts
        .sort_by(|left, right| left.path.cmp(&right.path));
    result
}

fn print_artifact_check(result: ArtifactCheckResult, json: bool) {
    if json {
        println!(
            "{{\"ok\":{},\"checked\":{},\"legacy\":{},\"errors\":{}}}",
            result.errors.is_empty(),
            json_strings(&result.checked),
            json_strings(&result.legacy),
            json_strings(&result.errors)
        );
    } else {
        for path in &result.checked {
            println!("ok: {path}");
        }
        for path in &result.legacy {
            println!("legacy: {path}");
        }
        for error in &result.errors {
            println!("error: {error}");
        }
    }
    if !result.errors.is_empty() {
        std::process::exit(6);
    }
}

fn resolve_context() -> Result<HarnessContext, InterfaceError> {
    let repo_root = match env::var_os("HARNESS_REPO_ROOT") {
        Some(path) => validate_explicit_repo_root(PathBuf::from(path))?,
        None => find_repository_root(env::current_dir().map_err(InterfaceError::CurrentDir)?)?,
    };
    let db_path = env::var_os("HARNESS_DB")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join("harness.db"));

    let schema_dir = resolve_schema_dir(&repo_root);

    Ok(HarnessContext {
        repo_root,
        db_path,
        schema_dir,
    })
}

fn validate_explicit_repo_root(path: PathBuf) -> Result<PathBuf, InterfaceError> {
    let root = path.canonicalize().map_err(|error| {
        InterfaceError::RepositoryRoot(format!("HARNESS_REPO_ROOT is not accessible: {error}"))
    })?;
    if !root.join(".git").exists() {
        return Err(InterfaceError::RepositoryRoot(
            "HARNESS_REPO_ROOT must point at a repository root containing .git".to_owned(),
        ));
    }
    Ok(root)
}

fn find_repository_root(start: PathBuf) -> Result<PathBuf, InterfaceError> {
    let start = start.canonicalize().map_err(|error| {
        InterfaceError::RepositoryRoot(format!("cannot canonicalize current directory: {error}"))
    })?;
    let roots = start
        .ancestors()
        .filter(|path| path.join(".git").exists())
        .collect::<Vec<_>>();
    match roots.as_slice() {
        [] => Err(InterfaceError::RepositoryRoot(
            "no repository root found from the current directory".to_owned(),
        )),
        [root] => Ok((*root).to_path_buf()),
        _ => Err(InterfaceError::RepositoryRoot(
            "ambiguous nested repository roots; set HARNESS_REPO_ROOT explicitly".to_owned(),
        )),
    }
}

fn resolve_schema_dir(repo_root: &std::path::Path) -> PathBuf {
    let harness_relative = repo_root.join("_harness/scripts/schema");
    if harness_relative.exists() {
        return harness_relative;
    }

    repo_root.join("scripts/schema")
}

fn print_matrix(records: &[StoryMatrixRecord], numeric: bool) {
    let rows = records
        .iter()
        .map(|record| {
            vec![
                record.id.clone(),
                record.title.clone(),
                record.status.clone(),
                proof_display(record.unit, numeric),
                proof_display(record.integration, numeric),
                proof_display(record.e2e, numeric),
                proof_display(record.platform, numeric),
                record.evidence.clone().unwrap_or_default(),
            ]
        })
        .collect::<Vec<_>>();
    print_table(
        &[
            "id", "title", "status", "unit", "integ", "e2e", "plat", "evidence",
        ],
        &rows,
    );
}

fn print_backlog(records: &[BacklogRecord]) {
    let rows = records
        .iter()
        .map(|record| {
            vec![
                record.id.to_string(),
                record.title.clone(),
                record.status.clone(),
                record.risk.clone().unwrap_or_default(),
                record.predicted_impact.clone().unwrap_or_default(),
                record.actual_outcome.clone().unwrap_or_default(),
            ]
        })
        .collect::<Vec<_>>();
    print_table(
        &[
            "id",
            "title",
            "status",
            "risk",
            "predicted_impact",
            "actual_outcome",
        ],
        &rows,
    );
}

fn print_decisions(records: &[DecisionRecord]) {
    let rows = records
        .iter()
        .map(|record| {
            vec![
                record.id.clone(),
                record.title.clone(),
                record.status.clone(),
                record.last_verified_at.clone().unwrap_or_default(),
                record.last_verified_result.clone().unwrap_or_default(),
            ]
        })
        .collect::<Vec<_>>();
    print_table(
        &[
            "id",
            "title",
            "status",
            "last_verified_at",
            "last_verified_result",
        ],
        &rows,
    );
}

fn print_intakes(records: &[IntakeRecord]) {
    let rows = records
        .iter()
        .map(|record| {
            vec![
                record.id.to_string(),
                record.created_at.clone(),
                record.input_type.clone(),
                record.risk_lane.clone(),
                record.summary.clone(),
            ]
        })
        .collect::<Vec<_>>();

    print_table(
        &["id", "created_at", "input_type", "risk_lane", "summary"],
        &rows,
    );
}

fn print_traces(records: &[TraceRecord]) {
    let rows = records
        .iter()
        .map(|record| {
            vec![
                record.id.to_string(),
                record.created_at.clone(),
                record.outcome.clone().unwrap_or_default(),
                record.task_summary.clone(),
                record.harness_friction.clone().unwrap_or_default(),
            ]
        })
        .collect::<Vec<_>>();
    print_table(
        &[
            "id",
            "created_at",
            "outcome",
            "task_summary",
            "harness_friction",
        ],
        &rows,
    );
}

fn print_friction(records: &[FrictionRecord]) {
    let rows = records
        .iter()
        .map(|record| {
            vec![
                record.id.to_string(),
                record.created_at.clone(),
                record.risk_lane.clone().unwrap_or_else(|| "-".to_owned()),
                record.input_type.clone().unwrap_or_else(|| "-".to_owned()),
                record.task_summary.clone(),
                record.harness_friction.clone(),
            ]
        })
        .collect::<Vec<_>>();
    print_table(
        &[
            "id",
            "created_at",
            "risk_lane",
            "input_type",
            "task_summary",
            "harness_friction",
        ],
        &rows,
    );
}

fn print_tools_summary(records: &[ToolEntry]) {
    let rows = records
        .iter()
        .map(|record| {
            vec![
                record.name.clone(),
                record.kind.clone(),
                record.capability.clone().unwrap_or_else(|| "-".to_owned()),
                record.responsibility.clone(),
                record.status.clone(),
                record.source.clone(),
            ]
        })
        .collect::<Vec<_>>();
    print_table(
        &[
            "name",
            "kind",
            "capability",
            "responsibility",
            "status",
            "source",
        ],
        &rows,
    );
}

fn print_tools_json(records: &[ToolEntry]) {
    println!("[");
    for (index, record) in records.iter().enumerate() {
        let comma = if index + 1 == records.len() { "" } else { "," };
        println!("  {{");
        println!("    \"provider\": \"{}\",", json_escape(&record.provider));
        println!("    \"name\": \"{}\",", json_escape(&record.name));
        println!("    \"command\": \"{}\",", json_escape(&record.command));
        println!(
            "    \"description\": \"{}\",",
            json_escape(&record.description)
        );
        println!("    \"args\": [");
        for (arg_index, arg) in record.args.iter().enumerate() {
            let arg_comma = if arg_index + 1 == record.args.len() {
                ""
            } else {
                ","
            };
            println!(
                "      {{\"name\":\"{}\",\"type\":\"{}\",\"required\":{},\"help\":\"{}\"}}{}",
                json_escape(&arg.name),
                json_escape(&arg.arg_type),
                arg.required,
                json_escape(arg.help.as_deref().unwrap_or("")),
                arg_comma
            );
        }
        println!("    ],");
        println!(
            "    \"responsibility\": \"{}\",",
            json_escape(&record.responsibility)
        );
        println!("    \"source\": \"{}\",", json_escape(&record.source));
        println!("    \"since\": \"{}\",", json_escape(&record.since));
        println!("    \"kind\": \"{}\",", json_escape(&record.kind));
        println!(
            "    \"capability\": {},",
            json_optional(record.capability.as_deref())
        );
        println!(
            "    \"scan_target\": {},",
            json_optional(record.scan_target.as_deref())
        );
        println!("    \"status\": \"{}\",", json_escape(&record.status));
        println!(
            "    \"checked_at\": {}",
            json_optional(record.checked_at.as_deref())
        );
        println!("  }}{comma}");
    }
    println!("]");
}

fn print_tool_check_summary(records: &[ToolCheckResult]) {
    if records.is_empty() {
        println!("No external tools registered. Optional tool capabilities are inactive.");
        return;
    }

    let rows = records
        .iter()
        .map(|record| {
            vec![
                record.name.clone(),
                record.kind.clone(),
                record.capability.clone().unwrap_or_else(|| "-".to_owned()),
                record.status.clone(),
                record.detail.clone(),
            ]
        })
        .collect::<Vec<_>>();
    print_table(&["name", "kind", "capability", "status", "detail"], &rows);
}

fn print_tool_check_json(records: &[ToolCheckResult]) {
    println!("[");
    for (index, record) in records.iter().enumerate() {
        let comma = if index + 1 == records.len() { "" } else { "," };
        println!("  {{");
        println!("    \"name\": \"{}\",", json_escape(&record.name));
        println!("    \"kind\": \"{}\",", json_escape(&record.kind));
        println!(
            "    \"capability\": {},",
            json_optional(record.capability.as_deref())
        );
        println!("    \"status\": \"{}\",", json_escape(&record.status));
        println!("    \"detail\": \"{}\"", json_escape(&record.detail));
        println!("  }}{comma}");
    }
    println!("]");
}

fn json_optional(value: Option<&str>) -> String {
    match value {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    }
}

fn proof_record_json(proof: &ProofRecord) -> String {
    format!(
        "{{\"story_id\":{},\"layer\":\"{}\",\"state\":\"{}\",\"executable\":{},\"argv_json\":{},\"exit_code\":{},\"head_commit\":{},\"branch\":{},\"dirty_fingerprint\":{},\"cli_version\":{},\"platform\":{},\"command_digest\":{},\"stdout_path\":{},\"stdout_hash\":{},\"stderr_path\":{},\"stderr_hash\":{},\"artifact_path\":{},\"artifact_hash\":{},\"summary\":{}}}",
        json_optional(proof.story_id.as_deref()),
        json_escape(&proof.layer),
        json_escape(&proof.state),
        json_optional(proof.executable.as_deref()),
        json_optional(proof.argv_json.as_deref()),
        proof
            .exit_code
            .map(|value| value.to_string())
            .unwrap_or_else(|| "null".to_owned()),
        json_optional(proof.head_commit.as_deref()),
        json_optional(proof.branch.as_deref()),
        json_optional(proof.dirty_fingerprint.as_deref()),
        json_optional(proof.cli_version.as_deref()),
        json_optional(proof.platform.as_deref()),
        json_optional(proof.command_digest.as_deref()),
        json_optional(proof.stdout_path.as_deref()),
        json_optional(proof.stdout_hash.as_deref()),
        json_optional(proof.stderr_path.as_deref()),
        json_optional(proof.stderr_hash.as_deref()),
        json_optional(proof.artifact_path.as_deref()),
        json_optional(proof.artifact_hash.as_deref()),
        json_optional(proof.summary.as_deref()),
    )
}

fn print_interventions(records: &[InterventionRecord]) {
    let rows = records
        .iter()
        .map(|record| {
            vec![
                record.id.to_string(),
                record.created_at.clone(),
                record
                    .trace_id
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                record.story_id.clone().unwrap_or_default(),
                record.intervention_type.clone(),
                record.source.clone(),
                record.description.clone(),
                record.impact.clone().unwrap_or_default(),
            ]
        })
        .collect::<Vec<_>>();
    print_table(
        &[
            "id",
            "created_at",
            "trace",
            "story",
            "type",
            "source",
            "description",
            "impact",
        ],
        &rows,
    );
}

fn print_stats(stats: &HarnessStats) {
    println!("=== Harness Stats ===");
    print_table(
        &["intakes", "stories", "decisions", "backlog_items", "traces"],
        &[vec![
            stats.intakes.to_string(),
            stats.stories.to_string(),
            stats.decisions.to_string(),
            stats.backlog_items.to_string(),
            stats.traces.to_string(),
        ]],
    );
}

fn print_query_table(table: &QueryTable) {
    let headers = table.headers.iter().map(String::as_str).collect::<Vec<_>>();
    print_table(&headers, &table.rows);
}

fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    let widths = headers
        .iter()
        .enumerate()
        .map(|(index, header)| {
            rows.iter()
                .filter_map(|row| row.get(index))
                .map(String::len)
                .chain(std::iter::once(header.len()))
                .max()
                .unwrap_or(header.len())
        })
        .collect::<Vec<_>>();

    print_row(
        &headers
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>(),
        &widths,
    );
    print_row(
        &widths
            .iter()
            .map(|width| "-".repeat(*width))
            .collect::<Vec<_>>(),
        &widths,
    );
    for row in rows {
        print_row(row, &widths);
    }
}

fn print_row(values: &[String], widths: &[usize]) {
    for (index, width) in widths.iter().enumerate() {
        if index > 0 {
            print!("  ");
        }
        let value = values.get(index).map(String::as_str).unwrap_or("");
        print!("{value:<width$}");
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn task_finish_gate_has_shared_structured_result_and_exit_five() {
        let expected = StructuredErrorResult::new(
            "TASK_PROOF_MISSING",
            "no proof run recorded",
            ["Run proof run before retrying task finish."],
        )
        .with_detail("gate", "TASK_PROOF_MISSING");
        let error = InterfaceError::Infrastructure(
            crate::infrastructure::HarnessInfraError::TaskFinishGate(expected.clone()),
        );

        assert_eq!(error.exit_code(), 5);
        assert_eq!(error.structured_result(), expected);
        let json = serde_json::to_value(error.structured_result()).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["code"], "TASK_PROOF_MISSING");
        assert!(json["remediation"]
            .as_array()
            .is_some_and(|items| !items.is_empty()));
    }

    #[test]
    fn foreign_rebuild_requires_an_explicit_recovery_switch() {
        assert!(rebuild_apply_state_allowed("HEALTHY", false));
        assert!(rebuild_apply_state_allowed("DB_MISSING", false));
        assert!(!rebuild_apply_state_allowed("DB_UNHEALTHY", false));
        assert!(!rebuild_apply_state_allowed("DB_AHEAD_OF_SOURCE", false));
        assert!(rebuild_apply_state_allowed("DB_UNHEALTHY", true));
        assert!(rebuild_apply_state_allowed("DB_AHEAD_OF_SOURCE", true));
        assert!(!rebuild_apply_state_allowed("DB_UNREADABLE", true));
    }

    #[test]
    fn story_help_documents_proof_command_shape() {
        let mut command = Cli::command();
        let story = command.find_subcommand_mut("story").unwrap();

        let update_help = story
            .find_subcommand_mut("update")
            .unwrap()
            .render_long_help()
            .to_string();
        assert!(update_help.contains("--unit <0|1>"));
        assert!(update_help.contains("--integration <0|1>"));
        assert!(update_help.contains("Proof flags use numeric booleans"));

        let verify_help = story
            .find_subcommand_mut("verify")
            .unwrap()
            .render_long_help()
            .to_string();
        assert!(verify_help.contains("story verify only accepts the story id"));
        assert!(verify_help.contains("Configure proof with story add/update --verify"));
    }

    #[test]
    fn command_help_documents_lane_values_and_version() {
        let mut command = Cli::command();
        let root_help = command.render_long_help().to_string();
        assert!(root_help.contains("Usage: _harness/bin/harness-cli <COMMAND>"));
        assert!(root_help.contains("--version"));

        let intake_help = command
            .find_subcommand_mut("intake")
            .unwrap()
            .render_long_help()
            .to_string();
        assert!(intake_help.contains("--lane <tiny|normal|high-risk>"));
        assert!(intake_help.contains("Use tiny instead of low"));

        let story_add_help = command
            .find_subcommand_mut("story")
            .unwrap()
            .find_subcommand_mut("add")
            .unwrap()
            .render_long_help()
            .to_string();
        assert!(story_add_help.contains("--lane <tiny|normal|high-risk>"));

        let task_start_help = command
            .find_subcommand_mut("task")
            .unwrap()
            .find_subcommand_mut("start")
            .unwrap()
            .render_long_help()
            .to_string();
        assert!(task_start_help.contains("--session <SESSION>"));
        assert!(task_start_help.contains("--lease-seconds <LEASE_SECONDS>"));

        let task_handoff_help = command
            .find_subcommand_mut("task")
            .unwrap()
            .find_subcommand_mut("handoff")
            .unwrap()
            .render_long_help()
            .to_string();
        assert!(task_handoff_help.contains("--from-session <FROM_SESSION>"));
        assert!(task_handoff_help.contains("--to-session <TO_SESSION>"));

        let backlog_add_help = command
            .find_subcommand_mut("backlog")
            .unwrap()
            .find_subcommand_mut("add")
            .unwrap()
            .render_long_help()
            .to_string();
        assert!(backlog_add_help.contains("--risk <tiny|normal|high-risk>"));
        assert!(backlog_add_help.contains("Accepted lanes"));

        let matrix_help = command
            .find_subcommand_mut("query")
            .unwrap()
            .find_subcommand_mut("matrix")
            .unwrap()
            .render_long_help()
            .to_string();
        assert!(matrix_help.contains("--numeric"));

        let audit_help = command
            .find_subcommand_mut("audit")
            .unwrap()
            .render_long_help()
            .to_string();
        assert!(audit_help.contains("--strict"));
        assert!(audit_help.contains("--json"));
        assert!(audit_help.contains("disposition"));

        let disposition_add_help = command
            .find_subcommand_mut("audit")
            .unwrap()
            .find_subcommand_mut("disposition")
            .unwrap()
            .find_subcommand_mut("add")
            .unwrap()
            .render_long_help()
            .to_string();
        assert!(disposition_add_help.contains("--finding-key"));
        assert!(disposition_add_help.contains("--approval-task"));
        assert!(disposition_add_help.contains("--provenance"));
    }

    #[test]
    fn audit_json_mode_is_detected_for_structured_errors() {
        let cli = Cli::try_parse_from(["harness-cli", "audit", "--strict", "--json"]).unwrap();
        assert!(cli.requests_json());
        let cli =
            Cli::try_parse_from(["harness-cli", "audit", "disposition", "list", "--json"]).unwrap();
        assert!(cli.requests_json());
    }

    #[test]
    fn compiled_command_manifest_matches_tracked_snapshot() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf();
        let expected = std::fs::read_to_string(repo_root.join("_harness/command-manifest.txt"))
            .unwrap()
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .map(str::to_owned)
            .collect::<Vec<_>>();
        assert_eq!(compiled_command_manifest(), expected);
    }

    #[test]
    fn workflow_parity_accepts_tracked_shadow_fixture() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf();
        let policy =
            crate::infrastructure::parse_workflow_policy(&repo_root.join("_harness/workflow.toml"))
                .unwrap();
        let result = workflow_parity(&repo_root, &policy, &compiled_command_manifest()).unwrap();
        assert!(result.failures.is_empty(), "{:?}", result.failures);
        assert!(result
            .deltas
            .iter()
            .any(|delta| delta.starts_with("one-flag-code-impact:")));
    }

    #[test]
    fn artifact_check_reports_duplicate_v1_ids_without_writing_files() {
        let temp = tempfile::tempdir().unwrap();
        let stories = temp.path().join("docs/stories");
        let product = temp.path().join("docs/product");
        std::fs::create_dir_all(&stories).unwrap();
        std::fs::create_dir_all(&product).unwrap();
        std::fs::write(product.join("sample.md"), "# Sample\n").unwrap();
        let content = "---\nschema: harness/story/v1\nid: US-900\ntitle: Duplicate\nstatus: planned\nlane: normal\nproduct_docs:\n  - docs/product/sample.md\n---\n# Duplicate\n";
        std::fs::write(stories.join("one.md"), content).unwrap();
        std::fs::write(stories.join("two.md"), content).unwrap();
        let result = artifact_check(temp.path(), Some("story"), None);
        assert!(result
            .errors
            .iter()
            .any(|error| error.contains("duplicate story id US-900")));
        assert_eq!(
            std::fs::read_to_string(stories.join("one.md")).unwrap(),
            content
        );
    }

    #[test]
    fn artifact_check_rejects_v1_story_with_invalid_lane_or_missing_product_doc() {
        let temp = tempfile::tempdir().unwrap();
        let stories = temp.path().join("docs/stories");
        std::fs::create_dir_all(&stories).unwrap();
        std::fs::write(stories.join("bad.md"), "---\nschema: harness/story/v1\nid: US-901\ntitle: Bad\nstatus: planned\nlane: sideways\nproduct_docs:\n  - docs/product/missing.md\n---\n# Bad\n").unwrap();
        let result = artifact_check(temp.path(), Some("story"), None);
        assert!(result
            .errors
            .iter()
            .any(|error| error.contains("invalid story lane")));
    }

    #[test]
    fn repository_root_discovery_accepts_one_root_and_rejects_nested_roots() {
        let temp = tempfile::Builder::new()
            .prefix("harness-cli-root-")
            .tempdir_in("/dev/shm")
            .unwrap();
        let root = temp.path().join("repo");
        let child = root.join("src/nested");
        std::fs::create_dir_all(root.join(".git")).unwrap();
        std::fs::create_dir_all(&child).unwrap();

        assert_eq!(find_repository_root(child.clone()).unwrap(), root);

        std::fs::create_dir_all(child.join(".git")).unwrap();
        let error = find_repository_root(child).unwrap_err();
        assert!(error
            .to_string()
            .contains("ambiguous nested repository roots"));
    }
}
