---
## Why

Provide command-line interface following BWA-MEM conventions for easy adoption by bioinformatics users.

## What Changes

- Implement `index` subcommand for building FM-index
- Implement `align` subcommand for aligning reads to existing index
- Implement `mem` subcommand for one-step index and align
- Add configurable alignment parameters
- Support multi-threading via `-t` option
- Support output redirection via `-o` option

## Capabilities

### New Capabilities

- `cli`: Command-line interface for index and alignment

### Modified Capabilities

None (initial implementation)

## Impact

- `src/main.rs` - CLI implementation with clap
- CLI documentation and examples
