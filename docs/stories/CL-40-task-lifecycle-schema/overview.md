# CL-40 Overview

## Status

completed

Migration 006 created inert task/proof tables. CL-40 makes transition rules
explicit and testable. The schema stays at canonical migration `006`; no
source migration was added because the retained local DB is ahead (`001..008`)
and must remain a rejected recovery input.
