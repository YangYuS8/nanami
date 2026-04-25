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

The current goal is Nanami 0.4:

- 0.4a in progress: permission protocol + mock permission flow.
- `nanami-core` provides mock permission request and resolve endpoints.
- `nanami-ui` displays a mock permission dialog skeleton.
- Decisions are recorded only; no dangerous action is executed.
- Real OpenClaw dangerous tool interception is not implemented yet.

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
