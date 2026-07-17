# CL-31 Overview

## Current Behavior

CL-30 can validate canonical artifacts but does not reconstruct SQLite state.

## Target Behavior

`memory rebuild --dry-run` reports exactly what canonical artifacts would
project into a newly initialized temporary DB. A later explicit apply path may
switch only after backup and validation.

The explicit apply path is fail-closed for an ahead/foreign database by
default. A reviewed recovery may add `--recover-foreign`; the command still
requires a healthy rebuilt candidate, checkpoints and backs up the quarantine
input, then atomically replaces the active DB.

## Status

completed
