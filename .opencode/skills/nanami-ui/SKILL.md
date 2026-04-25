---
name: nanami-ui
description: 当实现 Nanami 的 Qt/QML UI、Live2D 桌宠窗口、聊天面板、任务面板、设置页、托盘菜单时使用。
---
# Nanami UI Skill

## UI 职责

nanami-ui 只负责表现和交互，不负责 Agent 决策。

## 窗口

- PetWindow：透明、无边框、可置顶、可拖动、显示 Live2D。
- ChatPanel：聊天、代码块、文件拖拽、流式回复。
- TaskPanel：任务状态、工具调用、stdout/stderr、diff、产物。
- PermissionDialog：展示权限请求，允许一次、本任务允许、拒绝。
- SettingsPage：OpenClaw、CubeSandbox、TTS、STT、Live2D、权限策略配置。

## 规则

1. UI 状态来自 nanami-core 事件流。
2. 不在 QML 里写复杂业务逻辑。
3. 不在 UI 里保存权限决策。
4. 所有危险操作必须展示明确原因、作用范围和有效期。
5. Live2D 动作由 persona.state 事件驱动。
