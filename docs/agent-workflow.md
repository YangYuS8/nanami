# Nanami Agent Workflow

This document describes how coding agents should work inside this repository.

Agents MUST read this document before making non-trivial changes.

## Required Reading

Before editing code, read:

1. `AGENTS.md`
2. `README.md`
3. `docs/architecture.md`
4. `docs/protocol.md`
5. `docs/permission-model.md`
6. Relevant `.opencode/skills/*/SKILL.md`

## Skill Selection

Agents SHOULD load relevant skills before work.

Examples:

### Architecture work

Use:

```text
nanami-architecture
nanami-protocol
```

### Rust core work

Use:

```text
nanami-rust-core
nanami-permission-review
```

### UI work

Use:

```text
nanami-ui
nanami-desktop-integration
```

### OpenClaw integration

Use:

```text
nanami-openclaw-integration
nanami-protocol
nanami-permission-review
```

For Nanami 0.2, agents MUST keep OpenClaw integration inside `nanami-core` and `crates/nanami-openclaw`. The UI may only call `nanami-core` endpoints such as `/openclaw/status`; it must not call OpenClaw Gateway directly. Do not implement CubeSandbox or tool-call visualization as part of OpenClaw connection status work.

For Nanami 0.2b chat forwarding, agents MUST keep OpenClaw Gateway request/response parsing inside `crates/nanami-openclaw`. `nanami-core` owns `/chat`, validation, and error mapping. `nanami-ui` may only call `nanami-core` and must not parse OpenClaw-specific response shapes.

For Nanami 0.3a, agents MUST keep task/tool state structured in protocol types and core-produced SSE. UI may render mock task/tool events from `nanami-core`, but it must not infer tool state from free text and must not call OpenClaw, CubeSandbox, or system tools directly.

For Nanami 0.3b, agents MUST map OpenClaw tool-call related JSON into Nanami `EventEnvelope` values inside `crates/nanami-openclaw` or `nanami-core`. UI must only render structured events from `nanami-core`, and tool arguments may only be displayed as text/log output, never executed.

For Nanami 0.3c, agents SHOULD keep Task Panel state structured inside UI controllers. Even if the current QML still renders a text timeline, that timeline should be derived from structured task/tool state rather than from append-only free text updates.

### CubeSandbox integration

Use:

```text
nanami-cubesandbox-integration
nanami-permission-review
nanami-protocol
```

### Assets or third-party code

Use:

```text
nanami-license-assets
```

### Completion check

Use:

```text
nanami-release-check
```

## Standard Development Flow

Agents SHOULD follow this flow:

```text
1. Understand requirement.
2. Identify affected modules.
3. Load relevant skills.
4. Update or confirm protocol.
5. Write or update tests.
6. Implement smallest useful change.
7. Run verification.
8. Update docs.
9. Summarize evidence.
```

## Planning Rules

For non-trivial changes, agents MUST produce a short plan before editing.

The plan MUST include:

- affected files
- module boundary
- permission impact
- protocol impact
- verification commands

## Implementation Rules

Agents MUST:

- Keep changes focused.
- Prefer small commits.
- Avoid unrelated refactors.
- Add tests for core logic.
- Keep UI reactive to structured events.
- Avoid hardcoded local paths.
- Avoid hardcoded secrets.
- Use timeouts for external calls.
- Redact sensitive logs.

Agents MUST NOT:

- Add host command execution without permission design.
- Add UI features that depend on parsing natural language state.
- Bypass `nanami-core`.
- Commit generated binaries.
- Commit unlicensed models, fonts, or SDK files.
- Claim completion without verification.

## Verification Rules

Before claiming completion, agents MUST run relevant checks.

Rust:

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy -- -D warnings
```

Qt/C++:

```bash
cmake -S . -B build -G Ninja
cmake --build build
```

Documentation-only changes:

```text
Check links, headings, and consistency with architecture rules.
```

## Response Format for Agents

When reporting completion, agents SHOULD include:

```text
Summary:
- What changed

Verification:
- Commands run
- Results

Security:
- Permission impact
- Secret handling
- Sandbox/host execution impact

Docs:
- Documents updated or not needed
```

## When Unsure

Agents MUST choose the safer option.

Examples:

- Ask for permission rather than assuming approval.
- Use CubeSandbox rather than host execution.
- Add protocol type rather than parsing text.
- Redact logs rather than printing raw output.
- Update docs rather than relying on hidden assumptions.
