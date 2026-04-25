# Nanami Development Rules

Nanami is a desktop-native AI companion and development control surface for OpenClaw.

## Architecture

- nanami-ui: Qt/QML + C++ Live2D renderer.
- nanami-core: Rust backend daemon.
- OpenClaw: agent runtime, skills, hooks, memory.
- CubeSandbox: secure code execution.

## Rules

- Do not put business logic in UI.
- Do not execute local commands from UI.
- Do not bypass PermissionManager.
- Prefer CubeSandbox for untrusted code execution.
- Always update protocol types before UI event handling.
- Always add tests for Rust core logic.
- Always run verification before claiming completion.
