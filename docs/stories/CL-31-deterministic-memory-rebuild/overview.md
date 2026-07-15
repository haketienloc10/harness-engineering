# CL-31 Overview

## Current Behavior

CL-30 can validate canonical artifacts but does not reconstruct SQLite state.

## Target Behavior

`memory rebuild --dry-run` reports exactly what canonical artifacts would
project into a newly initialized temporary DB. A later explicit apply path may
switch only after backup and validation.

The explicit apply path is now available and fail-closed for an ahead/foreign
database; it is not used against this repository's quarantine input.

## Status

completed
