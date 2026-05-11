# Testing and Evaluator Policy

## Evaluator Separation

Evaluator must be separated from Generator in written artifacts.

Evaluator checks implementation against:

- original input
- planner brief
- implementation contract
- acceptance criteria
- real verification evidence

Evaluator must not approve only by reading code.

## Required Evidence

`05-evaluator-report.md` must include:

- command executed
- result summary
- pass/fail
- error output if failed
- runtime/API/browser evidence when relevant

Copied command output, logs, compiler errors, API responses, and stack traces must be preserved exactly.

## Default Verification

Run when possible:

```bash
bash .harness/scripts/verify.sh
```

If the app has runtime UI or API:

```bash
bash .harness/scripts/smoke.sh
```

For Vite:

```bash
APP_URL=http://localhost:5173 bash .harness/scripts/smoke.sh
```

## E2E Test Policy

E2E test is required when the implementation changes user-visible behaviour that cannot be reliably verified by unit tests, build checks, or static inspection.

For UI tasks, prefer E2E tests when the task includes any of these behaviours:

- form validation
- create/update/delete flow
- filtering or search
- navigation or routing
- modal/dialog interaction
- multi-step user flow
- state transition
- persistence across reloads
- authentication/authorization flow
- error state or empty state
- integration between UI and API
- regression-prone behaviour already covered by previous bugs

If the repository already has an E2E framework such as Playwright, Cypress, Selenium, or equivalent, the agent should add or update E2E tests for the changed behaviour.

If the repository does not have an E2E framework, the evaluator must still collect behaviour-level evidence by browser/manual/runtime checks, and the run should add a concrete Harness backlog proposal when E2E coverage would improve future verification.

Do not introduce a new E2E framework unless the implementation contract explicitly includes it or the user asks for it.

For small visual-only changes, copy changes, static text updates, or layout-only adjustments, E2E test is optional. In those cases, evaluator may use browser/manual evidence, screenshot evidence, or smoke checks if they directly verify the changed behaviour.

Evaluator must explain the E2E decision in `05-evaluator-report.md`:

- E2E required: yes/no
- Reason
- E2E command executed, if applicable
- Alternative verification method, if E2E was not used
- Backlog proposal created, if E2E was useful but unavailable

## UI Strictness

For UI tasks, build success, static checks, and curl smoke are insufficient.

Evaluator may approve only when each required UI behaviour has behaviour-level evidence.

Behaviour-level evidence may come from:

- E2E test result
- browser automation result
- manual browser verification
- runtime/API-equivalent verification
- screenshot evidence, when the change is visual or layout-related

For UI behaviour that affects user flow or state, E2E test is preferred.

Examples of required behaviours:

- form validation
- create/update/delete flow
- filtering
- navigation
- state transition
- persistence
- error state
- empty state

For each required behaviour, record:

- expected behaviour
- verification method
- evidence type
- concrete evidence
- pass/fail result

If evidence is missing for any required behaviour, mark the run `Fail`, `Needs Fix`, or `Blocked`, not `Pass`.