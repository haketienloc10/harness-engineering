# Architecture Guide

## Project Architecture

Describe the intended architecture here.

Recommended v0 default:

```txt
Modular Monolith
```

For frontend-only projects, describe the app structure instead.

## Module Rules

- Keep module boundaries explicit.
- Avoid direct cross-module data access.
- Prefer public application/service interfaces between modules.
- Do not introduce a new module without clear reason.
- Do not introduce a new dependency without justification.

## Layering Rules

Recommended backend layering:

```txt
Controller / API Layer
  -> Application Service / Use Case Layer
  -> Domain / Business Logic
  -> Repository / Persistence
```

Recommended frontend layering:

```txt
Page / Route
  -> Feature Component
  -> Shared Component
  -> State / Hook
  -> Utility
```

## Frontend State Rules

For small apps:
- keep local state simple
- use localStorage only when persistence is required
- avoid global state libraries unless necessary
- avoid external UI libraries unless requested or justified

## Migration to Microservices

Do not optimize for microservices too early.

If future extraction is needed, prefer:
- explicit module boundaries
- stable public ports/interfaces
- isolated data ownership where practical
- integration tests around module contracts
