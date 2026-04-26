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

Current 0.5b phase: OpenClaw sandbox event mapping. This phase maps structured OpenClaw sandbox events into Nanami `sandbox.started` / `sandbox.updated` / `sandbox.output` / `sandbox.artifact` / `sandbox.completed` events on `/tasks/openclaw/stream`. It does not call the real CubeSandbox API, execute commands, mount host directories, read or write artifact files, or use network access.

Current 0.5c phase: structured Sandbox View + permission/audit link. This phase upgrades `nanami-ui` to maintain structured sandbox view state and derive display text from that state, while clarifying that any future real mount/network capability must go through `PermissionManager`. It does not call the real CubeSandbox API, execute commands, mount host directories, read or write artifact files, or use network access.

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
User runs a mock sandbox stream or receives mapped OpenClaw sandbox events.
Nanami shows sandbox ID, template, network policy, mounts, output, artifacts, completion state, and related permission/audit guidance without performing any real sandbox operation.
```

## Version 0.6: Desktop Companion Experience

Goal: make Nanami feel like a companion.

Current 0.6a phase: companion shell + persona state mock foundation. This phase adds `persona.state` protocol events, a mock persona SSE endpoint, and a placeholder pet view in `nanami-ui`. It does not integrate a real Live2D renderer, model assets, tray, notifications, or advanced window behavior.

Current 0.6b phase: tray + notifications + basic window behavior. This phase adds a minimal system tray menu, tray-backed notifications, and show/hide/toggle main window behavior. It does not integrate a real Live2D renderer, transparent or always-on-top window behavior, dragging, or platform-specific hacks.

Current 0.6c phase: Live2D renderer adapter boundary + placeholder renderer abstraction. This phase adds a UI-side renderer controller that receives persona state and exposes renderer-facing state to QML, without integrating a real Live2D SDK, model assets, or complex animation.

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
User runs a mock persona stream, toggles the main window from the tray, triggers a mock notification, and sees renderer-facing placeholder state update with persona changes.
Nanami displays a placeholder companion shell while exposing a clear renderer adapter boundary without advanced pet window effects.
```

## Version 0.7: Development Workflow

Goal: support a real assisted development loop.

Current 0.7a phase: development workflow protocol + mock workflow stream + UI skeleton. This phase adds structured workflow events, a mock-only workflow SSE endpoint, and a lightweight workflow panel in `nanami-ui`. It does not read real project files, execute commands, call real CubeSandbox, write files, or apply patches.

Current 0.7b phase: mock project metadata + structured workflow state. This phase adds a mock project metadata endpoint, a lightweight project panel in `nanami-ui`, and an internal structured workflow view state. It still does not read real project files, detect project manifests, execute commands, call real CubeSandbox, write files, or apply patches.

Current 0.7c phase: sandbox test result visualization + patch proposal view. This phase extends mock workflow visualization with richer test result and patch proposal metadata such as command preview, duration, failed test names, and patch risk level. It still does not read real project files, execute commands, call real CubeSandbox, write files, or apply patches.

Current 0.7d phase: permission-gated apply patch mock flow. This phase adds a mock apply-patch request endpoint and a UI path to request apply-patch permission for visualization only. It still does not read real project files, write files, apply real patches, execute commands, or call real CubeSandbox.

Current 0.7e phase: OpenClaw workflow event mapping. This phase maps structured OpenClaw workflow JSON into Nanami workflow events on `/tasks/openclaw/stream`. It still does not read real project files, execute commands, call real CubeSandbox, write files, or apply patches.

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
User selects a project folder or loads a mock project, then runs a mock workflow or receives structured OpenClaw workflow events.
Nanami shows minimal project metadata and workflow visualization without reading source content or touching the real project state.
```

## Version 0.8: Real Project Selection and Trust

Goal: let the user explicitly choose a real project boundary before later trust and workflow expansion.

Current 0.8a phase: explicit project selection + manifest-only project metadata. This phase allows the user to explicitly choose a project directory and lets `nanami-core` detect only top-level manifest filenames to produce minimal metadata. It does not read source content, manifest contents, recursively scan the project, execute commands, call CubeSandbox, write files, or apply patches.

Current 0.8b phase: project trust confirmation + in-memory trust state. This phase lets the user explicitly confirm trust for the currently selected project, but stores that trust in memory only. It still does not read source content, manifest contents, recursively scan the project, execute commands, call CubeSandbox, write files, or apply patches.

Current 0.8c phase: read-only project structure summary. This phase exposes a shallow first-level project structure summary for the currently selected trusted project. It does not read source content, manifest contents, recursively scan the project, execute commands, call CubeSandbox, write files, or apply patches.

Current 0.8d phase: connect selected project context to mock workflow. This phase allows mock workflow generation to reference selected project metadata and shallow structure summary data. It still does not read source content, manifest contents, recursively scan the project, execute commands, call CubeSandbox, write files, or apply patches.

## Version 0.9: Permission-Gated Manifest Introspection

Goal: allow a narrow, explicit, audited read of the selected trusted project's top-level manifest without expanding into source analysis or execution.

Current 0.9a phase: permission-gated manifest preview only. This phase allows `nanami-core` to request explicit L2 `filesystem.read` approval for the currently selected trusted project, then read only one supported top-level manifest file (`Cargo.toml`, `package.json`, or `pyproject.toml`) and return a capped preview of at most 8 KB.

Current 0.9b phase: permission-gated manifest summary extraction only. This phase allows `nanami-core` to reuse the same approved top-level manifest read, still capped to 8 KB, and derive a structured summary from that manifest content only. It may extract package name, version, dependency count, script count, and a short summary text, but it does not widen the read scope beyond the same top-level manifest.

Current 0.9c phase: core module split cleanup only. This phase reorganizes `nanami-core` internals into smaller modules such as routes, services, state, error handling, and mock workflow helpers. It does not change public API paths, protocol fields, permission semantics, or runtime capability boundaries.

Current 0.9d phase: QML panel split cleanup only. This phase reorganizes `nanami-ui` by splitting `Main.qml` into focused panel components such as status, pet, chat, task, permission, sandbox, project, and workflow panels. It does not change public API paths, protocol fields, controller interfaces, permission semantics, or runtime capability boundaries.

Current 0.9e phase: core test module split cleanup only. This phase reorganizes `nanami-core` tests into focused modules such as health, openclaw, chat, tasks, permissions, sandbox, persona, workflow, projects, and manifest, plus shared support helpers. It does not change public API paths, protocol fields, permission semantics, runtime capability boundaries, or the intended meaning of the test assertions.

Current 0.9f phase: protocol module split cleanup only. This phase reorganizes `nanami-protocol` into focused modules such as chat, error, session, openclaw, task, tool, sandbox, persona, project, manifest, workflow, permission, audit, and event, while preserving the same public re-exports from `lib.rs`. It does not change public API paths, protocol fields, serde shapes, event type strings, permission semantics, or runtime capability boundaries.

Current 0.9g phase: OpenClaw adapter module split cleanup only. This phase reorganizes `nanami-openclaw` into focused modules such as client, config, error, status, chat, SSE parsing, agent stream handling, and mapping helpers for tool, sandbox, and workflow events. It does not change public API paths, protocol fields, event mapping behavior, serde shapes, error semantics, or runtime capability boundaries.

Current 0.9h phase: OpenClaw adapter test module split cleanup only. This phase reorganizes `nanami-openclaw` agent event mapping tests into focused native, tool, sandbox, and workflow test modules with shared support helpers. It does not change public API paths, protocol fields, event mapping behavior, serde shapes, error semantics, runtime capability boundaries, or the intended meaning of the test assertions.

0.9 runtime capability boundary remains unchanged across 0.9a through 0.9h: Nanami does not read source content, does not recursively scan the project, does not execute commands, does not call CubeSandbox, does not write files, and does not download dependencies.

Expected demo:

```text
User explicitly selects a project folder, confirms trust, requests manifest preview permission, approves the L2 read, then loads both a top-level manifest preview and a structured manifest summary.
Nanami derives summary fields only from the same capped top-level manifest content and does not read source files, execute commands, or use CubeSandbox.
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
