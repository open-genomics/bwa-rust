# OpenSpec Specifications

## Overview

bwa-rust uses [OpenSpec](https://github.com/LessUp/openspec) for capability-driven development. Each specification defines a distinct capability with clear boundaries.

## Specification Structure

Specifications are located in `openspec/specs/`:

```
openspec/specs/
├── 001-core-types.md
├── 002-fm-index.md
├── 003-seeding.md
├── 004-chaining.md
├── 005-alignment.md
├── 006-sam-output.md
├── 007-parallelism.md
├── 008-cli.md
├── 009-validation.md
├── 010-memory-safety.md
└── 011-error-handling.md
```

## Capability Overview

| Spec | Capability | Status |
|------|------------|:------:|
| 001 | Core Types (DNA, Contig, Read) | <span class="status-badge delivered">✓</span> |
| 002 | FM-index Construction & Query | <span class="status-badge delivered">✓</span> |
| 003 | SMEM Seeding | <span class="status-badge delivered">✓</span> |
| 004 | Chain Building | <span class="status-badge delivered">✓</span> |
| 005 | Smith-Waterman Alignment | <span class="status-badge delivered">✓</span> |
| 006 | SAM Output (CIGAR, MAPQ, Tags) | <span class="status-badge delivered">✓</span> |
| 007 | Rayon Parallelism | <span class="status-badge delivered">✓</span> |
| 008 | CLI Interface | <span class="status-badge delivered">✓</span> |
| 009 | Validation & Testing | <span class="status-badge delivered">✓</span> |
| 010 | Memory Safety (no unsafe) | <span class="status-badge delivered">✓</span> |
| 011 | Error Handling | <span class="status-badge delivered">✓</span> |

## Specification Format

Each specification follows this structure:

```markdown
# Spec: [Capability Name]

## Intent
What problem does this capability solve?

## Scope
What is included and excluded?

## Interface
Public API and types.

## Behavior
Expected behavior and invariants.

## Verification
How correctness is verified.
```

## Reading Specifications

To understand a capability:

1. Read the spec in `openspec/specs/`
2. Check the implementation in `src/`
3. Review tests in `src/*/tests.rs`

## Development Workflow

When adding or modifying capabilities:

1. **Propose**: Create or update spec
2. **Implement**: Write code matching spec
3. **Verify**: Add tests for spec requirements
4. **Document**: Update architecture docs

---

[Next: Validation →](/en/specs/validation)
