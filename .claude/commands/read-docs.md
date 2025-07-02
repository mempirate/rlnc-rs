---
allowed-tools: Bash(cargo:*), Bash(ls:*)
description: Read the documentation for the given crate. Optionally compile the docs first.
---

## Prerequisites

- Crate: $ARGUMENTS
- Check if the docs are compiled: !`ls target/doc/crate`
- If not, compile the docs: !`cargo doc --workspace --all-features`
- Check all documentation files in the crate docs: !`ls target/doc/crate`

## Task
Once the docs are compiled, read the documentation for the given crate (in target/doc/$ARGUMENTS). Give the user a one-line summary of the documentation to show that you've read it.