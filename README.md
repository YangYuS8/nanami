# Nanami

Nanami is a desktop-native AI companion and local visual companion client for OpenClaw.

Nanami is not a standalone AI agent runtime. Nanami is the local visual desktop client for OpenClaw. It provides a Live2D companion UI, chat client surface, task and tool event visualization, sandbox event visualization, permission interaction, and desktop notifications.

## Project Goal

Nanami turns OpenClaw into a local visual desktop companion client without replacing the OpenClaw runtime.

It should be able to:

- Chat with the user through a desktop companion interface.
- Display OpenClaw agent activity in a structured way.
- Visualize skills, tool calls, sandbox execution, task progress, and errors.
- Ask for explicit permission before risky desktop operations.
- Use CubeSandbox for isolated code and shell execution.
- Present OpenClaw task, tool, and sandbox activity through a local desktop companion experience.

## Non-Goals

Nanami MUST NOT become a separate agent runtime.

Nanami MUST NOT duplicate OpenClaw memory, skills, hooks, or long-term agent logic.

Nanami MUST NOT replicate OpenClaw planning or other agent-runtime orchestration responsibilities.

Nanami MUST NOT execute local commands directly from the UI.

Nanami MUST NOT bypass the PermissionManager.

Nanami MUST NOT silently read or write user project files.

## Architecture Summary

```text
nanami-ui
    Qt/QML + C++ Live2D renderer
    Desktop windows, tray, chat panel, task panel, permission dialog

nanami-core
    Rust daemon
    Session manager, task state machine, permission manager,
    OpenClaw adapter, CubeSandbox/E2B adapter, desktop bridge

OpenClaw
    Agent runtime, skills, hooks, memory, tool calling

CubeSandbox
    Secure isolated execution environment for code and shell commands
```

## Repository Layout

```text
nanami/
├── apps/
│   ├── nanami-ui/
│   └── nanami-core/
├── crates/
│   ├── nanami-protocol/
│   ├── nanami-openclaw/
│   ├── nanami-sandbox/
│   ├── nanami-permission/
│   └── nanami-storage/
├── docs/
│   ├── architecture.md
│   ├── protocol.md
│   ├── permission-model.md
│   ├── security.md
│   ├── mvp-roadmap.md
│   └── agent-workflow.md
├── assets/
├── packaging/
└── .opencode/skills/
```

The layout may evolve, but agents MUST keep the boundary between UI, core, OpenClaw, and CubeSandbox.

## Development Rules for Agents

Agents working on Nanami MUST follow these rules:

1. Read `AGENTS.md` before making changes.
2. Use project skills from `.opencode/skills/` when relevant.
3. Update protocol types before implementing UI event handling.
4. Add tests for Rust core logic.
5. Do not put business logic in UI.
6. Do not bypass PermissionManager.
7. Prefer CubeSandbox for untrusted code execution.
8. Do not introduce assets or SDK binaries without checking license rules.
9. Run verification before claiming completion.

## Current Priority

The current goal is Nanami 0.9:

Product positioning note for 0.10a: Nanami is now described as the local visual companion client for OpenClaw, not as a development control surface. OpenClaw remains the agent runtime, and CubeSandbox remains the safe execution environment for dangerous development operations.

- 0.4a completed: permission protocol + mock permission flow.
- 0.4b completed: dangerous tool request interception visibility.
- 0.4c completed: permission decision flow + in-memory audit log.
- 0.5a completed: sandbox protocol + mock sandbox stream + UI skeleton.
- 0.5b completed: OpenClaw sandbox event mapping.
- 0.5c completed: structured sandbox view + permission/audit link.
- 0.6a completed: companion shell + persona state mock foundation.
- 0.6b completed: tray + notifications + basic window behavior.
- 0.6c completed: Live2D renderer adapter boundary + placeholder renderer abstraction.
- 0.7a completed: development workflow protocol + mock workflow stream + UI skeleton.
- 0.7b completed: mock project metadata + structured workflow state.
- 0.7c completed: sandbox test result visualization + patch proposal view.
- 0.7d completed: permission-gated apply patch mock flow.
- 0.7e completed: OpenClaw workflow event mapping.
- 0.8a completed: explicit project selection + manifest-only project metadata.
- 0.8b completed: project trust confirmation + in-memory trust state.
- 0.8c completed: read-only project structure summary.
- 0.8d completed: connect selected project context to mock workflow.
- 0.9a completed: permission-gated manifest preview.
- 0.9b completed: permission-gated manifest summary extraction.
- 0.9c completed: core module split cleanup.
- 0.9d completed: QML panel split cleanup.
- 0.9e completed: core test module split cleanup.
- 0.9f completed: protocol module split cleanup.
- 0.9g completed: OpenClaw adapter module split cleanup.
- 0.9h completed: OpenClaw adapter test module split cleanup.
- 0.9i completed: UI controller networking/SSE cleanup.
- `nanami-core` provides mock permission request, decision, audit, and sandbox stream endpoints.
- `nanami-core` also provides a mock persona state stream endpoint.
- `nanami-ui` displays mock permission and sandbox visualization skeletons with structured sandbox view state, plus a placeholder pet view.
- 0.5a is mock sandbox visualization only.
- 0.5a does not call the real CubeSandbox API, does not execute commands, does not mount host directories, does not read or write artifact files, does not use network access, and does not consume real OpenClaw cube-sandbox events.
- 0.5b maps OpenClaw structured sandbox events into Nanami sandbox events through `/tasks/openclaw/stream` only.
- 0.5b still does not call the real CubeSandbox API, execute commands, mount host directories, read or write artifact files, or enable network access.
- 0.5c upgrades the Sandbox View to structured UI state and clearer permission/audit guidance only.
- Real sandbox mount/network capability still belongs to future PermissionManager-gated phases and is not executed in 0.5c.
- 0.6a adds persona state protocol and a mock companion shell only.
- Real Live2D renderer, tray integration, notifications, and advanced window behavior remain deferred to 0.6b/0.6c.
- 0.6b adds a system tray, basic notifications, and show/hide/toggle window behavior only.
- Real Live2D renderer and advanced desktop window behavior such as transparency, always-on-top, dragging, and other pet-specific effects remain deferred to 0.6c or later.
- 0.6c adds a placeholder renderer abstraction and renderer adapter boundary only.
- Real Live2D SDK integration, model assets, complex animation, transparency, always-on-top, dragging, and other advanced pet effects remain deferred to later phases.
- 0.7a adds development workflow mock visualization only.
- It does not read real project files, execute commands, call real CubeSandbox, write files, or apply patches.
- 0.7b adds mock project metadata and structured workflow UI state only.
- It still does not read real project files, execute commands, call real CubeSandbox, write files, apply patches, or turn permission approvals into real writes.
- 0.7c improves mock test result and patch proposal visualization only.
- It still does not read real project files, execute commands, call real CubeSandbox, write files, apply patches, or turn permission approvals into real writes.
- 0.7d adds a permission-gated apply patch mock flow only.
- It still does not read real project files, write files, apply real patches, execute commands, call real CubeSandbox, or turn permission approvals into real writes.
- 0.7e maps structured OpenClaw workflow JSON into Nanami workflow events through `/tasks/openclaw/stream` only.
- It still does not read real project files, execute commands, call real CubeSandbox, write files, apply patches, or infer workflow state from natural language.
- 0.8a adds explicit user-triggered project folder selection and manifest-only metadata detection.
- It does not read source content, does not read manifest contents, does not recursively scan the project, does not execute commands, and does not call CubeSandbox.
- 0.8b adds user-confirmed in-memory trust state for the selected project only.
- It does not allow automatic source reads, file writes, command execution, or CubeSandbox usage, and still does not read source content or manifest contents.
- 0.8c adds a shallow read-only project structure summary only.
- It does not read source content, does not read manifest contents, does not recursively scan the project, does not execute commands, and does not call CubeSandbox.
- 0.8d connects selected project metadata and shallow structure summary to mock workflow only.
- It does not perform real project analysis, does not read source content, does not execute commands, does not call CubeSandbox, and does not write files.
- 0.9a adds explicit L2 permission-gated top-level manifest preview for the currently selected trusted project only.
- It reads only one top-level manifest file (`Cargo.toml`, `package.json`, or `pyproject.toml`) after explicit approval, returns at most an 8 KB preview, and is not source analysis.
- It does not read source content, does not recursively scan the project, does not execute commands, does not call CubeSandbox, and does not write files.
- 0.9b adds structured manifest summary extraction from the same permission-gated top-level manifest read only.
- It extracts package metadata and dependency/script counts from the top-level manifest content already allowed by the same L2 permission, still capped to 8 KB, and still does not perform source analysis.
- It does not read source content, does not recursively scan the project, does not execute commands, does not call CubeSandbox, does not write files, and does not download dependencies.
- 0.9c is a core module split cleanup only.
- It reorganizes `nanami-core` implementation into smaller route/service/state modules without changing API paths, protocol shapes, UI behavior, permission semantics, manifest read limits, or runtime capability boundaries.
- 0.9d is a QML panel split cleanup only.
- It reorganizes `Main.qml` into smaller panel components under `qml/components/` while keeping the same context properties, controller behavior, endpoint usage, permission flow, and runtime capability boundaries.
- 0.9e is a core test module split cleanup only.
- It reorganizes `nanami-core` crate tests into focused modules and shared support helpers without changing API behavior, protocol shapes, permission semantics, runtime capabilities, or test assertion intent.
- 0.9f is a protocol module split cleanup only.
- It reorganizes `nanami-protocol` into focused module files with re-exports from `lib.rs`, without changing public type names, serde shapes, event type strings, protocol fields, or runtime capability boundaries.
- 0.9g is an OpenClaw adapter module split cleanup only.
- It reorganizes `nanami-openclaw` into focused client/config/error/chat/SSE/agent/mapping modules without changing its public API, event mapping behavior, serde shapes, error semantics, or runtime capability boundaries.
- 0.9h is an OpenClaw adapter test module split cleanup only.
- It reorganizes `nanami-openclaw` agent event mapping tests into focused native/tool/sandbox/workflow modules and shared support helpers without changing public API, event mapping behavior, serde shapes, error semantics, runtime capability boundaries, or test assertion intent.
- 0.9i is a UI controller networking/SSE cleanup only.
- It introduces lightweight shared helpers for HTTP JSON requests, JSON object parsing, network error strings, and SSE frame extraction across `nanami-ui` controllers without changing QML-visible properties, controller invokables, endpoint usage, permission flow, or runtime capability boundaries.

## Development

Install Rust stable, CMake, Ninja, and Qt 6 Quick development packages.

Check the Rust workspace:

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy -- -D warnings
```

Configure and build the Qt/CMake skeleton:

```bash
cmake -S . -B build -G Ninja
cmake --build build
```

## Run

Start `nanami-core`:

```bash
cargo run -p nanami-core
```

Check the health endpoint:

```bash
curl http://127.0.0.1:17878/health
```

Check the OpenClaw Gateway connection status through `nanami-core`:

```bash
curl http://127.0.0.1:17878/openclaw/status
```

Send a basic chat request through `nanami-core`:

```bash
curl -X POST http://127.0.0.1:17878/chat \
  -H 'content-type: application/json' \
  -d '{"message":"Hello Nanami"}'
```

Stream a chat response through `nanami-core` SSE:

```bash
curl -N -X POST http://127.0.0.1:17878/chat/stream \
  -H 'content-type: application/json' \
  -d '{"message":"Hello Nanami"}'
```

`POST /chat` remains a non-streaming fallback. `POST /chat/stream` is the 0.2c true token streaming path used by the UI, with upstream chunks forwarded incrementally instead of buffering the whole response first.

Configure the OpenClaw Gateway URL before starting `nanami-core`:

```bash
NANAMI_OPENCLAW_GATEWAY_URL=http://127.0.0.1:18789 cargo run -p nanami-core
```

`NANAMI_OPENCLAW_TOKEN` may be set when the gateway requires authentication. Do not commit real tokens.

`NANAMI_OPENCLAW_CHAT_PATH` may override the 0.2b/0.2c placeholder chat path. The default is `/chat` and is centralized in `crates/nanami-openclaw`; it may change when the OpenClaw Gateway chat API stabilizes.

Run `nanami-ui` after building:

```bash
./build/apps/nanami-ui/nanami-ui
```

The current UI displays `nanami-core` health, OpenClaw Gateway connection status, and a streaming chat form through `nanami-core` only. Nanami 0.2c now performs true incremental assistant streaming rather than emitting a single buffered SSE body; tool call visualization is not implemented yet.

Run the 0.3a mock task stream through `nanami-core`:

```bash
curl -N http://127.0.0.1:17878/tasks/mock/stream
```

Nanami 0.3a provides a mock Task/Tool visualization foundation only. It does not parse real OpenClaw tool calls yet.

Run the 0.3b OpenClaw task/tool event mapping stream through `nanami-core`:

```bash
curl -N -X POST http://127.0.0.1:17878/tasks/openclaw/stream \
  -H 'content-type: application/json' \
  -d '{"message":"Run project check"}'
```

Nanami 0.3b maps OpenClaw streaming tool events into Nanami task/tool events for visualization only. It does not execute tools.

Run the 0.4a mock permission request stream through `nanami-core`:

```bash
curl -N http://127.0.0.1:17878/permissions/mock/stream
```

Resolve a mock permission request:

```bash
curl -X POST http://127.0.0.1:17878/permissions/resolve \
  -H 'content-type: application/json' \
  -d '{"permission_id":"perm_mock_read_project","decision":"allow_once"}'
```

Query the current in-memory decision for a permission:

```bash
curl http://127.0.0.1:17878/permissions/decision/perm_mock_read_project
```

Read the in-memory permission audit log:

```bash
curl http://127.0.0.1:17878/permissions/audit
```

The current audit log only exists in memory and is not persisted.

Run the 0.5a mock sandbox stream through `nanami-core`:

```bash
curl -N http://127.0.0.1:17878/sandbox/mock/stream
```

Nanami 0.5a provides mock sandbox visualization only. It does not call real CubeSandbox, does not execute commands, does not mount host directories, does not read or write artifact files, and does not enable network access.

Nanami 0.5b adds OpenClaw structured sandbox event mapping to `/tasks/openclaw/stream`. This is event mapping only: it does not call real CubeSandbox, does not execute commands, does not mount host directories, does not read or write artifact files, and does not enable network access.

Nanami 0.5c upgrades the Sandbox View to structured UI state derived from sandbox events and links the view more clearly to permission/audit guidance. It still does not call real CubeSandbox, execute commands, mount host directories, read or write artifact files, or enable network access.

Run the 0.6a mock persona stream through `nanami-core`:

```bash
curl -N http://127.0.0.1:17878/persona/mock/stream
```

Nanami 0.6a provides a companion shell and mock persona state foundation only. It does not integrate a real Live2D SDK, does not load model assets, and does not implement tray, notification, transparent window, always-on-top, or drag behaviors yet.

Nanami 0.6b adds a basic tray menu, tray-backed notifications, and show/hide/toggle window behavior. It still does not integrate a real Live2D SDK, model assets, transparent window behavior, always-on-top, dragging, or platform-specific desktop hacks.

Nanami 0.6c adds a UI-side renderer adapter boundary and placeholder renderer abstraction for persona-driven pet display. It still does not integrate a real Live2D SDK, load model assets, implement complex animation, or enable advanced pet window behavior.

Run the 0.7a mock workflow stream through `nanami-core`:

```bash
curl -N http://127.0.0.1:17878/workflow/mock/stream
```

Nanami 0.7a provides development workflow mock visualization only. It does not read real project files, execute commands, call real CubeSandbox, write files, apply patches, or turn permission approvals into real writes.

Nanami 0.7b adds mock project metadata and a structured workflow UI state layer. It still does not read real project files, detect project files, execute commands, call real CubeSandbox, write files, apply patches, or trigger real writes from permission approvals.

Nanami 0.7c improves mock test result and patch proposal visualization with command preview, failed test names, duration, and patch risk level. It still does not read real project files, execute commands, call real CubeSandbox, write files, or apply patches.

Nanami 0.7d adds a permission-gated apply patch mock flow for visualization only. It records mock permission requests and mock apply status without reading real project files, writing files, or applying real patches.

Nanami 0.7e adds OpenClaw structured workflow event mapping into Nanami workflow events on `/tasks/openclaw/stream`. It still does not read real project files, execute commands, call real CubeSandbox, write files, or apply patches.

Nanami 0.8a adds explicit project selection and manifest-only project metadata detection. It only checks top-level manifest filenames and still does not read source content, manifest contents, execute commands, call CubeSandbox, write files, or apply patches.

Nanami 0.8b adds a user-confirmed in-memory trust state for the currently selected project. This trust state does not by itself allow automatic reads, writes, command execution, or CubeSandbox usage.

Nanami 0.8c adds a shallow read-only project structure summary for the currently selected trusted project. It lists only first-level entries and still does not read source content, manifest contents, execute commands, call CubeSandbox, write files, or apply patches.

Nanami 0.8d connects selected project metadata and shallow structure summary to a mock current-project workflow stream. It still does not perform real project analysis, read source content, execute commands, call CubeSandbox, write files, or apply patches.

Nanami 0.9a adds permission-gated manifest preview only. The user must first explicitly select and trust a project, then explicitly approve an L2 `filesystem.read` request before `nanami-core` reads a single top-level manifest file and returns a capped preview. This is not source analysis: Nanami does not read source files, does not recursively scan the project, does not execute commands, does not call CubeSandbox, and does not write files.

Nanami 0.9b adds permission-gated manifest summary extraction only. After the same explicit L2 `filesystem.read` approval used for manifest preview, `nanami-core` may read the same top-level manifest content, still capped to 8 KB, and derive structured fields such as package name, version, dependency count, script count, and a short summary text. This still is not source analysis: Nanami does not read source files, does not recursively scan the project, does not execute commands, does not call CubeSandbox, does not write files, and does not download dependencies.

Nanami 0.9c is a pure internal cleanup phase for `nanami-core`. It splits the oversized `apps/nanami-core/src/lib.rs` into focused route, service, state, error, and mock modules for maintainability only. It does not change endpoint paths, protocol fields, permission behavior, manifest preview/summary limits, source read scope, command execution, CubeSandbox usage, or file writes.

Nanami 0.9d is a pure internal cleanup phase for `nanami-ui`. It splits the oversized `apps/nanami-ui/qml/Main.qml` into panel components such as status, pet, chat, task, permission, sandbox, project, and workflow panels under `qml/components/`. It does not change controller interfaces, endpoint usage, permission behavior, source read scope, command execution, CubeSandbox usage, or file writes.

Nanami 0.9e is a pure internal cleanup phase for `nanami-core` tests. It splits the oversized `apps/nanami-core/src/tests.rs` into focused test modules and shared support helpers for maintainability only. It does not change endpoint paths, protocol fields, permission behavior, runtime capabilities, or the semantics of the existing test assertions.

Nanami 0.9f is a pure internal cleanup phase for `nanami-protocol`. It splits the oversized `crates/nanami-protocol/src/lib.rs` into focused modules such as chat, task, tool, sandbox, persona, project, manifest, workflow, permission, audit, and event, while preserving the same public re-exports from `nanami_protocol`. It does not change any public type names, serde shapes, event type strings, protocol fields, or runtime capability boundaries.

Nanami 0.9g is a pure internal cleanup phase for `nanami-openclaw`. It splits the oversized `crates/nanami-openclaw/src/lib.rs` into focused modules such as client, config, error, status, chat, SSE parsing, agent stream handling, and mapping helpers for tool, sandbox, and workflow events. It does not change the adapter public API, event mapping behavior, serde shapes, error semantics, or runtime capability boundaries.

Nanami 0.9h is a pure internal cleanup phase for `nanami-openclaw` tests. It splits the mixed `crates/nanami-openclaw/tests/agent_events.rs` coverage into focused native, tool, sandbox, and workflow test modules with shared support helpers. It does not change the adapter public API, event mapping behavior, serde shapes, error semantics, runtime capability boundaries, or the meaning of existing test assertions.

Nanami 0.9i is a pure internal cleanup phase for `nanami-ui` controllers. It introduces lightweight shared helpers so controllers no longer each reimplement the same HTTP JSON request setup, JSON object parsing, network error mapping, and SSE data frame extraction logic. It does not change QML-visible properties, controller invokables, core API usage, permission behavior, source read scope, command execution, CubeSandbox usage, or file writes.

## Verification

Run all current project checks:

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy -- -D warnings
cmake -S . -B build -G Ninja
cmake --build build
```
