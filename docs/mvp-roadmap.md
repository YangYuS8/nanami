# Nanami MVP Roadmap

This document defines staged development goals.

Agents MUST avoid implementing later-stage features before the earlier stage has a working, verifiable baseline.

## Version 0.1: Process Skeleton

Goal: establish the clean project foundation.

Required:

- Repository layout.
- `nanami-core` Rust daemon.
- `nanami-ui` Qt/QML app skeleton.
- Local connection between UI and core.
- Health check endpoint.
- Basic event stream.
- Basic settings placeholder.
- Basic build instructions.

Verification:

```bash
cargo check
cargo test
cmake -S . -B build -G Ninja
cmake --build build
```

Expected demo:

```text
Start nanami-core.
Start nanami-ui.
UI shows core connection status.
```

## Version 0.2: OpenClaw Connection

Goal: connect Nanami to OpenClaw.

Current first phase: OpenClaw Gateway connection status only. This phase establishes configuration, reachability checks, auth/pairing/scope status mapping, and UI status display. It is not the full chat or streaming implementation.

Current 0.2b phase: basic chat request forwarding and complete response display. `nanami-ui` sends user input to `nanami-core`, `nanami-core` forwards through `nanami-openclaw`, and the UI displays the complete assistant response. True token streaming belongs to 0.2c or a later small step.

Required:

- OpenClaw Gateway configuration.
- Connection status.
- Reconnect behavior.
- Auth/pairing error state.
- Basic chat request forwarding.
- Streaming message display.
- Structured error events.

Deferred within 0.2:

- True token streaming, which belongs to 0.2c or a later small step.
- OpenClaw tool call visualization, which belongs to 0.3.

Expected demo:

```text
User sends a message from Nanami.
OpenClaw responds.
Nanami displays the complete assistant response.
```

## Version 0.3: Task and Tool Call Visualization

Goal: show agent activity as structured tasks.

Required:

- Task state machine.
- Tool call event model.
- Task panel UI.
- stdout/stderr display.
- tool started/output/completed events.
- Persona state mapping.

Expected demo:

```text
OpenClaw starts a task.
Nanami displays task timeline and tool status.
```

## Version 0.4: Permission System

Goal: enforce explicit user approval.

Required:

- PermissionManager.
- Permission request events.
- Permission dialog.
- allow_once / allow_for_task / deny.
- Audit records.
- Sensitive value redaction.

Expected demo:

```text
OpenClaw requests project file read.
Nanami asks permission.
User approves.
Task continues.
```

## Version 0.5: CubeSandbox Integration View

Goal: visualize isolated execution.

Required:

- CubeSandbox/E2B adapter boundary.
- Sandbox event model.
- Sandbox execution status.
- Network policy display.
- Mount display.
- stdout/stderr/artifacts display.
- No silent fallback to host execution.

Expected demo:

```text
OpenClaw uses cube-sandbox skill.
Nanami shows sandbox ID, command, output, and result.
```

## Version 0.6: Desktop Companion Experience

Goal: make Nanami feel like a companion.

Required:

- Pet window.
- Live2D renderer placeholder or real renderer.
- Persona state mapping.
- Tray menu.
- Desktop notifications.
- Chat panel polish.
- Basic TTS placeholder or interface.

Expected demo:

```text
Nanami reacts visually to idle, thinking, tool_call, waiting_permission, success, and error states.
```

## Version 0.7: Development Workflow

Goal: support a real assisted development loop.

Required:

- Open project.
- Analyze project.
- Run tests in sandbox.
- Show result.
- Generate patch.
- Ask before writing.
- Apply patch after approval.
- Re-run verification.

Expected demo:

```text
User asks Nanami to fix a small project issue.
Nanami analyzes, tests in sandbox, proposes diff, applies after approval, and verifies.
```

## Out of Scope for MVP

Do not implement these before the core loop works:

- Full plugin marketplace.
- Persistent autonomous background agent.
- Complex long-term memory system outside OpenClaw.
- Multi-user collaboration.
- Cloud sync.
- Mobile app.
- Direct IDE replacement.
- Unrestricted host automation.
