# Run Manifest

## Execution Mode

- mode: template_subagents_required
- fallback_allowed: false
- subagent_runtime_available: false
- run_status: running

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
