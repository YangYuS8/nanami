# Nanami

Nanami is a desktop-native AI companion and development control surface for OpenClaw.

Nanami is not a standalone AI agent runtime. Nanami is the desktop interaction layer for OpenClaw. It provides a Live2D companion UI, chat surface, task panel, permission control, desktop integration, and development workflow visualization.

## Project Goal

Nanami turns OpenClaw into a desktop companion with real development capabilities.

It should be able to:

- Chat with the user through a desktop companion interface.
- Display OpenClaw agent activity in a structured way.
- Visualize skills, tool calls, sandbox execution, task progress, and errors.
- Ask for explicit permission before risky desktop operations.
- Use CubeSandbox for isolated code and shell execution.
- Help with real development workflows such as debugging, testing, patch generation, and project analysis.

## Non-Goals

Nanami MUST NOT become a separate agent runtime.

Nanami MUST NOT duplicate OpenClaw memory, skills, hooks, or long-term agent logic.

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

The current goal is Nanami 0.8:

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
