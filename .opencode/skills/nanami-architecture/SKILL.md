---
name: nanami-architecture
description: 当修改 Nanami 的模块边界、进程通信、目录结构、协议设计、权限模型或 OpenClaw/CubeSandbox 集成架构时使用。
---
# Nanami Architecture Skill

Nanami 是 OpenClaw 的桌面人格化控制层，不是普通聊天桌宠。

## 核心边界

- nanami-ui 只负责桌面交互、Live2D 表现、聊天面板、任务面板、权限确认。
- nanami-core 负责任务状态机、权限策略、OpenClaw adapter、CubeSandbox/E2B adapter、本地事件总线。
- OpenClaw 负责 Agent 推理、skills、hooks、memory。
- CubeSandbox 负责隔离执行代码和 shell 命令。

## 规则

1. 不要把 Agent 逻辑塞进 UI。
2. 不要让 UI 直接执行本地命令。
3. 不要让 UI 直接读写项目文件，必须经过 nanami-core 权限层。
4. 不要让 CubeSandbox 逻辑散落在 UI 代码中。
5. 新增能力必须先定义事件协议，再实现 UI。
6. 所有跨进程消息必须可序列化、可记录、可回放。
