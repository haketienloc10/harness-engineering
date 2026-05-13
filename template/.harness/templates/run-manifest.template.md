# Run Manifest

## Execution Mode

- mode: template_subagents_required
- dispatch_mode: template_based
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

## Role Template Sources

- planner_template: .harness/subagents/planner.md
- contract_reviewer_template: .harness/subagents/contract-reviewer.md
- generator_template: .harness/subagents/generator.md
- evaluator_template: .harness/subagents/evaluator.md

## Required Block Codes

- required_subagent_template_unavailable: BLOCKED_REQUIRED_SUBAGENT_TEMPLATE_UNAVAILABLE
- required_subagent_unavailable: BLOCKED_REQUIRED_SUBAGENT_UNAVAILABLE
- required_generator_unavailable: BLOCKED_REQUIRED_GENERATOR_UNAVAILABLE
- coordinator_context_over_budget: BLOCKED_COORDINATOR_CONTEXT_OVER_BUDGET
