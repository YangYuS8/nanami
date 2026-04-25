---
name: nanami-openclaw-integration
description: 当实现 OpenClaw Gateway 连接、会话管理、skills/tool call 展示、hooks、memory、pairing/auth、OpenClaw adapter 或 OpenClaw 事件映射时使用。
---
# Nanami OpenClaw Integration Skill

Nanami 是 OpenClaw 的桌面人格化控制层，不是独立 Agent Runtime。

## 集成边界

- OpenClaw 负责 Agent 推理、skills、hooks、memory、tool calling。
- nanami-core 负责连接 OpenClaw、转换事件、管理会话、权限确认和任务状态。
- nanami-ui 只展示 OpenClaw 状态、消息、工具调用、权限请求和任务结果。
- 不允许 UI 直接调用模型、直接执行工具或直接读写项目文件。

## 原则

1. 所有 Agent 能力优先通过 OpenClaw Gateway 进入。
2. Nanami 不复制 OpenClaw 的 memory，不另起一套长期记忆系统。
3. Nanami 不直接实现 skills；只提供桌面能力桥接和权限控制。
4. OpenClaw tool call 必须映射为 Nanami task/tool 事件。
5. tool call 过程必须可展示、可记录、可追踪。
6. OpenClaw 请求桌面危险能力时，必须经过 PermissionManager。
7. pairing/auth/token 不得写死在代码中，必须进入安全配置或密钥存储。
8. Gateway 断连、重连、鉴权失败、scope 不足都必须有明确 UI 状态。
9. 不允许为了绕过 OpenClaw 限制而让 Nanami 直接执行同等危险操作。

## 必须处理的状态

- disconnected
- connecting
- connected
- pairing_required
- auth_failed
- scope_missing
- task_running
- waiting_permission
- tool_call_running
- tool_call_failed
- completed

## UI 展示要求

- 当前 OpenClaw Gateway 地址
- 连接状态
- 当前会话
- 当前 Agent/Profile
- 正在调用的 skill/tool
- 权限请求
- 错误原因和可操作建议
