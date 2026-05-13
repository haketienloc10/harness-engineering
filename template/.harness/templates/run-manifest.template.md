# Run Manifest

## Execution Mode

- mode: codex_project_subagents_required
- dispatch_mode: codex_project_scoped
- fallback_allowed: false
- subagent_runtime_available: unknown
- run_status: created_pending_executor_check
- coordinator_source_edits_allowed: false
- coordinator_role_work_allowed: false

## Required Role Instances

- planner: pending
- contract_reviewer: pending
- generator: pending
- evaluator: pending

## Role Codex Agent Files

- planner_agent_name: harness_planner
- planner_agent_file: .codex/agents/harness-planner.toml
- contract_reviewer_agent_name: harness_contract_reviewer
- contract_reviewer_agent_file: .codex/agents/harness-contract-reviewer.toml
- generator_agent_name: harness_generator
- generator_agent_file: .codex/agents/harness-generator.toml
- evaluator_agent_name: harness_evaluator
- evaluator_agent_file: .codex/agents/harness-evaluator.toml

## Required Block Codes

- required_codex_agent_unavailable: BLOCKED_REQUIRED_CODEX_AGENT_UNAVAILABLE
- required_subagent_unavailable: BLOCKED_REQUIRED_SUBAGENT_UNAVAILABLE
- required_generator_unavailable: BLOCKED_REQUIRED_GENERATOR_UNAVAILABLE
- coordinator_context_over_budget: BLOCKED_COORDINATOR_CONTEXT_OVER_BUDGET
