# Nanami Architecture

This document defines Nanami's architecture for both humans and agents.

Agents MUST follow this document when modifying module boundaries, process communication, task flow, permission logic, or integration with OpenClaw and CubeSandbox.

## Core Principle

Nanami is the local visual desktop client for OpenClaw.

Nanami provides desktop interaction, chat entry, event visualization, permission interaction, notification delivery, and companion presentation.

OpenClaw provides the agent runtime.

CubeSandbox provides the safe execution environment.

Nanami does not own agent reasoning, memory, skills, hooks, or planning. It visualizes and mediates those runtime capabilities for the user on the local desktop.

## High-Level Architecture

```text
┌─────────────────────────────────────────────┐
│                 nanami-ui                   │
│                                             │
│ Qt/QML + C++ Live2D Renderer                │
│                                             │
│ Responsibilities:                           │
│ - Pet window                                │
│ - Chat panel                                │
│ - Task panel                                │
│ - Permission dialog                         │
│ - Settings page                             │
│ - Tray menu                                 │
│ - Desktop notifications                     │
└─────────────────────┬───────────────────────┘
                      │
                      │ Local WebSocket / HTTP / IPC
                      ▼
┌─────────────────────────────────────────────┐
│                nanami-core                  │
│                                             │
│ Rust daemon                                 │
│                                             │
│ Responsibilities:                           │
│ - Session management                        │
│ - Task state machine                        │
│ - Permission manager                        │
│ - OpenClaw adapter                          │
│ - CubeSandbox/E2B adapter                   │
│ - Desktop capability bridge                 │
│ - Event bus                                 │
│ - Local storage                             │
└───────────────┬─────────────────────┬───────┘
                │                     │
                ▼                     ▼
┌─────────────────────────┐   ┌─────────────────────────┐
│        OpenClaw          │   │       CubeSandbox        │
│                         │   │                         │
│ Agent runtime            │   │ KVM MicroVM sandbox      │
│ Skills                   │   │ E2B-compatible API       │
│ Hooks                    │   │ Code execution           │
│ Memory                   │   │ Shell execution          │
│ Tool calling             │   │ File I/O                 │
└─────────────────────────┘   └─────────────────────────┘
```

## Responsibility Boundaries

### Nanami

Nanami is the local client and companion UI layer around OpenClaw.

Nanami is responsible for:

- Providing the local chat entry point.
- Rendering the Live2D companion and desktop presence.
- Visualizing chat, task, tool, workflow, and sandbox events.
- Presenting permission requests, consent choices, and audit visibility.
- Surfacing notifications, tray interactions, and user-facing status.

Nanami is not responsible for:

- Agent reasoning.
- Planning.
- Memory.
- Skills.
- Hooks.
- Re-implementing OpenClaw runtime logic.

### OpenClaw

OpenClaw is the agent runtime.

OpenClaw owns:

- Agent reasoning.
- Planning.
- Skills.
- Hooks.
- Memory.
- Tool calling.
- Persona and runtime behavior.

Nanami MUST NOT duplicate these systems.

### CubeSandbox

CubeSandbox is the safe execution environment for dangerous code and shell operations initiated through OpenClaw-approved flows.

CubeSandbox is responsible for:

- Isolated code execution.
- Isolated shell execution.
- File I/O inside the sandbox boundary.
- Execution policy constraints such as network and mount controls.

Nanami MUST NOT silently fall back to host execution when sandbox execution fails.

### Permission UI

The permission UI is the user-facing consent and visibility layer.

It is responsible for:

- Showing what action is being requested.
- Showing the relevant scope and risk.
- Collecting explicit allow or deny decisions.
- Giving the user visibility into what happened afterward.

It is not a separate policy engine. Policy enforcement still belongs to `nanami-core` and its `PermissionManager`.

## Module Boundaries

### nanami-ui

`nanami-ui` is responsible for presentation and user interaction.

It MAY:

- Render the Live2D companion.
- Present a dedicated pet window as part of the desktop companion layer.
- Display chat messages.
- Display task state.
- Display tool call progress.
- Display permission requests.
- Send user input to `nanami-core`.
- Display desktop notifications.
- Show settings.

It MUST NOT:

- Execute shell commands.
- Read or write project files directly.
- Call CubeSandbox directly.
- Call model providers directly.
- Store permanent permissions by itself.
- Implement agent reasoning logic.

Live2D renderer notes:

- The UI renderer adapter MAY expose multiple backends such as `placeholder` and `live2d`.
- The pet window belongs to the `nanami-ui` presentation layer and MUST remain a UI-only shell around renderer/controller state.
- Real Live2D SDK integration, model resources, packaging, and platform support MUST be handled separately from the adapter foundation.
- When the Live2D SDK or model resources are unavailable, the UI MUST fall back safely to the placeholder renderer.

### nanami-core

`nanami-core` is responsible for local client orchestration and policy enforcement.

It MAY:

- Connect to OpenClaw Gateway.
- Translate OpenClaw runtime events into Nanami client events.
- Manage tasks and sessions.
- Manage permissions.
- Call CubeSandbox through approved adapters.
- Expose a local API to `nanami-ui`.
- Record task history and audit logs.
- Bridge desktop capabilities after permission approval.

It MUST:

- Enforce PermissionManager decisions.
- Use structured events.
- Record dangerous operations.
- Apply timeouts to external calls.
- Avoid leaking secrets in logs.

### OpenClaw and CubeSandbox Usage

OpenClaw remains the system that decides what the agent is trying to do.

CubeSandbox SHOULD be used for:

- Running untrusted code.
- Running shell commands generated by AI.
- Testing code safely.
- Executing temporary scripts.
- Installing dependencies in isolation.
- Running network-restricted experiments.

Nanami does not restrict OpenClaw capability by redefining runtime ownership, but it also does not bypass OpenClaw or CubeSandbox to perform dangerous operations on its own.

## Event Flow

A typical task flow:

```text
User input
    ↓
nanami-ui sends message to nanami-core
    ↓
nanami-core forwards request to OpenClaw
    ↓
OpenClaw starts reasoning and emits tool calls
    ↓
nanami-core maps tool calls into task events
    ↓
PermissionManager requests approval when needed
    ↓
nanami-ui displays permission dialog
    ↓
User approves or denies
    ↓
nanami-core continues or aborts operation
    ↓
OpenClaw/CubeSandbox returns result
    ↓
nanami-ui displays final response and task summary
```

## Agent Implementation Rules

Agents MUST:

1. Decide which module owns a change before editing code.
2. Keep protocol changes in protocol definitions.
3. Keep permission checks in `nanami-core`.
4. Keep rendering logic in `nanami-ui`.
5. Keep OpenClaw-specific logic in OpenClaw adapter modules.
6. Keep CubeSandbox-specific logic in sandbox adapter modules.
7. Add tests for Rust core behavior.
8. Update docs when architecture changes.

Agents MUST NOT:

1. Add shortcuts that bypass permissions.
2. Put business logic in QML.
3. Parse free text to infer tool state when structured events are available.
4. Store secrets in plain logs.
5. Commit unlicensed assets.
