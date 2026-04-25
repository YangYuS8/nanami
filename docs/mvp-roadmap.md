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

Current 0.2c phase: true token streaming. `nanami-openclaw` parses OpenClaw Gateway streaming responses, `nanami-core` exposes `POST /chat/stream` as SSE, and `nanami-ui` displays assistant text incrementally.

Required:

- OpenClaw Gateway configuration.
- Connection status.
- Reconnect behavior.
- Auth/pairing error state.
- Basic chat request forwarding.
- Streaming message display.
- Structured error events.

Deferred within 0.2:

- OpenClaw tool call visualization, which belongs to 0.3.

Expected demo:

```text
User sends a message from Nanami.
OpenClaw responds.
Nanami displays the assistant response incrementally through SSE.
```

## Version 0.3: Task and Tool Call Visualization

Goal: show agent activity as structured tasks.

Current 0.3a phase: Task/Tool visualization foundation only. This phase adds protocol types, a mock task stream endpoint, and a UI task panel skeleton. Real OpenClaw tool call parsing is deferred to 0.3b.

Current 0.3b phase: OpenClaw tool event mapping. This phase maps OpenClaw streaming tool call JSON into Nanami task/tool events for visualization only. 0.3c will upgrade the Task Panel from simple text timeline to a structured data model.

Current 0.3c phase completed: structured Task Panel state. `TaskController` now maintains structured in-memory task/tool state and regenerates the timeline from it. A richer structured Task Panel view can build on this without changing the transport protocol.

Required:

- Task state machine.
- Tool call event model.
- Task panel UI.
- stdout/stderr display.
- tool started/output/completed events.
- Persona state mapping.

Expected demo:

```text
User runs an OpenClaw task stream.
Nanami displays mapped task timeline and tool status without executing tools.
```

## Version 0.4: Permission System

Goal: enforce explicit user approval.

Current 0.4a phase: permission protocol + mock permission flow. This phase adds structured permission events, a mock core permission stream, and a UI permission dialog skeleton.

Current 0.4b phase: dangerous tool request interception visibility. This phase classifies mapped OpenClaw tool requests and inserts `permission.requested` events into task streams, but does not execute tools or return decisions back to OpenClaw.

Current 0.4c phase: permission decision flow + in-memory audit log. This phase records permission decisions in memory, exposes query endpoints, and keeps an in-memory audit trail without executing any real operation.

Required:

- PermissionManager.
- Permission request events.
- Permission dialog.
- allow_once / allow_for_task / deny.
- Audit records.
- Sensitive value redaction.

Expected demo:

```text
Mock permission request appears in Nanami.
User chooses allow_once / allow_for_task / deny.
Nanami records the mock decision.
```

CubeSandbox remains part of 0.5. Real tool execution is still out of scope.

## Version 0.5: CubeSandbox Integration View

Goal: visualize isolated execution.

Current 0.5a phase: sandbox protocol + mock sandbox stream + UI skeleton. This phase adds structured sandbox events, a mock-only sandbox SSE endpoint, and a basic Sandbox View. It does not call the real CubeSandbox API, execute commands, mount host directories, read or write artifact files, use network access, or consume real OpenClaw cube-sandbox events.

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
User runs a mock sandbox stream.
Nanami shows sandbox ID, template, network policy, mounts, output, artifacts, and completion state without performing any real sandbox operation.
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
