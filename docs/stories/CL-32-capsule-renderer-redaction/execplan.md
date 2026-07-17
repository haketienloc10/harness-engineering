# CL-32 Exec Plan: Capsule Renderer and Redaction

## Goal

Render versioned task capsules through an explicit CLI command with safe path
generation, bounded content, redaction and checksum validation.

## Risk

High-risk: portable records, privacy/redaction and future closure invariants.

## Scope

Renderer/parser, atomic staging/rename, secret and absolute-path redaction,
collision refusal and orphan detection. Task-finish integration stays CL-43.
