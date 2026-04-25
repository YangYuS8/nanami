# Nanami Protocol

This document defines the event and API protocol between `nanami-ui` and `nanami-core`.

Agents MUST update this document when adding, removing, or changing protocol messages.

## Protocol Goals

The protocol MUST be:

- Structured.
- Serializable.
- Recordable.
- Replayable.
- Versionable.
- Safe for UI consumption.

The UI MUST NOT parse free text to determine task state. It MUST use structured fields.

## Transport

Initial implementation SHOULD use local WebSocket.

Suggested endpoint:

```text
ws://127.0.0.1:<port>/events
```

Optional HTTP endpoints MAY be added for request-response actions.

## Common Event Shape

All events MUST include:

```json
{
  "type": "event.type",
  "id": "evt_...",
  "timestamp": "2026-01-01T00:00:00Z"
}
```

Events related to a task SHOULD include:

```json
{
  "task_id": "task_..."
}
```

Events related to a session SHOULD include:

```json
{
  "session_id": "sess_..."
}
```

Events related to a tool call SHOULD include:

```json
{
  "tool_call_id": "tool_..."
}
```

Events related to a permission request SHOULD include:

```json
{
  "permission_id": "perm_..."
}
```

## Event Categories

### Session Events

```json
{
  "type": "session.started",
  "id": "evt_001",
  "timestamp": "2026-01-01T00:00:00Z",
  "session_id": "sess_001",
  "title": "Default Session"
}
{
  "type": "session.updated",
  "id": "evt_002",
  "timestamp": "2026-01-01T00:00:00Z",
  "session_id": "sess_001",
  "status": "connected"
}
```

### OpenClaw Events

`openclaw.status` reports Nanami's view of the OpenClaw Gateway connection. The UI consumes this structured state from `nanami-core`; it must not call OpenClaw directly.

```json
{
  "type": "openclaw.status",
  "id": "evt_openclaw_001",
  "timestamp": "2026-01-01T00:00:00Z",
  "status": "connected",
  "gateway_url": "http://127.0.0.1:18789",
  "message": "OpenClaw Gateway reachable",
  "agent": "default-agent",
  "profile": "desktop"
}
```

Valid OpenClaw connection statuses:

```text
disconnected
connecting
connected
pairing_required
auth_failed
scope_missing
error
```

Optional fields:

```text
message
agent
profile
```

### Message Events

Basic chat request-response payloads:

```json
{
  "session_id": "sess_001",
  "message": "Hello Nanami"
}
{
  "session_id": "sess_001",
  "message_id": "msg_001",
  "content": "Hello user"
}
```

Valid chat roles:

```text
user
assistant
system
```

```json
{
  "type": "message.user",
  "id": "evt_010",
  "timestamp": "2026-01-01T00:00:00Z",
  "session_id": "sess_001",
  "message_id": "msg_001",
  "content": "帮我检查这个项目为什么构建失败"
}
{
  "type": "message.delta",
  "id": "evt_011",
  "timestamp": "2026-01-01T00:00:01Z",
  "session_id": "sess_001",
  "message_id": "msg_002",
  "delta": "我先检查项目结构。"
}
{
  "type": "message.completed",
  "id": "evt_012",
  "timestamp": "2026-01-01T00:00:02Z",
  "session_id": "sess_001",
  "message_id": "msg_002",
  "content": "我发现构建失败原因是缺少依赖。"
}
```

0.2b exposes `POST /chat` as a basic forwarding endpoint returning a complete `ChatResponse`. True token streaming is deferred.

### Task Events

```json
{
  "type": "task.started",
  "id": "evt_020",
  "timestamp": "2026-01-01T00:00:03Z",
  "session_id": "sess_001",
  "task_id": "task_001",
  "title": "检查项目构建错误",
  "status": "running"
}
```

Valid task statuses:

```text
pending
running
waiting_permission
failed
completed
cancelled
```

```json
{
  "type": "task.updated",
  "id": "evt_021",
  "timestamp": "2026-01-01T00:00:04Z",
  "task_id": "task_001",
  "status": "waiting_permission",
  "summary": "需要读取项目目录"
}
{
  "type": "task.completed",
  "id": "evt_022",
  "timestamp": "2026-01-01T00:00:10Z",
  "task_id": "task_001",
  "status": "completed",
  "summary": "构建问题已定位"
}
```

### Tool Events

```json
{
  "type": "tool.started",
  "id": "evt_030",
  "timestamp": "2026-01-01T00:00:05Z",
  "task_id": "task_001",
  "tool_call_id": "tool_001",
  "tool": "cube-sandbox.commands.run",
  "summary": "在沙箱中执行 cargo check"
}
{
  "type": "tool.output",
  "id": "evt_031",
  "timestamp": "2026-01-01T00:00:06Z",
  "task_id": "task_001",
  "tool_call_id": "tool_001",
  "stream": "stdout",
  "content": "checking nanami-core..."
}
```

Valid output streams:

```text
stdout
stderr
log
artifact
```

```json
{
  "type": "tool.completed",
  "id": "evt_032",
  "timestamp": "2026-01-01T00:00:08Z",
  "task_id": "task_001",
  "tool_call_id": "tool_001",
  "status": "completed",
  "exit_code": 0
}
```

### Permission Events

```json
{
  "type": "permission.requested",
  "id": "evt_040",
  "timestamp": "2026-01-01T00:00:04Z",
  "task_id": "task_001",
  "permission_id": "perm_001",
  "level": "L2",
  "action": "filesystem.read",
  "target": "/home/user/Code/nanami",
  "reason": "需要读取项目配置以分析构建错误",
  "scope": "task",
  "expires": "task_completed"
}
{
  "type": "permission.resolved",
  "id": "evt_041",
  "timestamp": "2026-01-01T00:00:05Z",
  "permission_id": "perm_001",
  "decision": "allow_once"
}
```

Valid permission decisions:

```text
allow_once
allow_for_task
deny
```

### Sandbox Events

```json
{
  "type": "sandbox.started",
  "id": "evt_050",
  "timestamp": "2026-01-01T00:00:06Z",
  "task_id": "task_001",
  "sandbox_id": "sbx_001",
  "template_id": "tpl_001",
  "network": "disabled",
  "mounts": [
    {
      "host_path": "/home/user/Code/nanami",
      "sandbox_path": "/workspace/nanami",
      "readonly": true
    }
  ]
}
{
  "type": "sandbox.completed",
  "id": "evt_051",
  "timestamp": "2026-01-01T00:00:12Z",
  "task_id": "task_001",
  "sandbox_id": "sbx_001",
  "status": "destroyed"
}
```

### Persona Events

```json
{
  "type": "persona.state",
  "id": "evt_060",
  "timestamp": "2026-01-01T00:00:03Z",
  "state": "thinking",
  "emotion": "focused",
  "text": "我先看一下项目结构。"
}
```

Valid persona states:

```text
idle
listening
thinking
speaking
tool_call
waiting_permission
success
error
```

### Error Events

```json
{
  "type": "error.occurred",
  "id": "evt_070",
  "timestamp": "2026-01-01T00:00:09Z",
  "task_id": "task_001",
  "severity": "error",
  "code": "OPENCLAW_SCOPE_MISSING",
  "message": "OpenClaw Gateway 缺少所需 scope",
  "action_hint": "请重新 pairing 或更新 Gateway 授权"
}
```

## Versioning

Protocol messages SHOULD include a version field when stable API compatibility becomes necessary.

Suggested field:

```json
{
  "protocol_version": "0.1"
}
```

## Agent Rules

Agents MUST:

1. Define event types before implementing UI.
2. Update this document when adding protocol fields.
3. Keep event names stable.
4. Use structured status fields.
5. Avoid using display text as machine-readable state.
