---
name: nanami-rust-core
description: 当实现 nanami-core、Rust crate、任务状态机、权限系统、OpenClaw/CubeSandbox adapter、本地 API 时使用。
---
# Nanami Rust Core Skill

## 技术原则

- 使用 tokio 异步运行时。
- 使用官方最新稳定 Rust edition。当前项目默认使用 Rust 2024 edition。
- 使用 serde 定义所有协议类型。
- 使用 tracing 记录结构化日志。
- 所有外部调用都必须有 timeout。
- 所有危险操作都必须经过 PermissionManager。
- 核心逻辑必须可单元测试。
- 不允许在 handler 中堆业务逻辑，应拆到 service 层。

## 必须验证

每次完成后运行：

```bash
cargo fmt
cargo check
cargo test
cargo clippy
```

## 建议模块

- protocol：事件和请求响应类型
- task：任务状态机
- permission：权限策略和授权记录
- openclaw：OpenClaw Gateway adapter
- sandbox：CubeSandbox/E2B adapter
- desktop：桌面能力桥接
- api：本地 HTTP/WebSocket API
- storage：SQLite/JSON 配置和任务记录
