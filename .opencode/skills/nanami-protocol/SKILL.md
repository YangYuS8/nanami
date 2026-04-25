---
name: nanami-protocol
description: 当新增或修改 nanami-ui 与 nanami-core 之间的事件、WebSocket/HTTP API、任务状态、权限请求、tool call schema 时使用。
---
# Nanami Protocol Skill

## 原则

1. 协议类型先于 UI 实现。
2. 所有事件必须有 type、id、timestamp。
3. task_id、session_id、permission_id、tool_call_id 必须明确。
4. 事件必须可序列化、可记录、可回放。
5. 协议变更必须考虑向后兼容或版本号。
6. UI 不应解析自由文本来判断状态，必须依赖结构化字段。

## 常见事件

- session.started
- message.delta
- message.completed
- task.started
- task.updated
- task.completed
- tool.started
- tool.output
- tool.completed
- permission.requested
- permission.resolved
- sandbox.started
- sandbox.output
- sandbox.completed
- persona.state
- error.occurred
