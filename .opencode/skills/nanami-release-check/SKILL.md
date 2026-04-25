---
name: nanami-release-check
description: 当准备合并分支、标记功能完成、打包发布或声称任务完成前使用。
---
# Nanami Release Check Skill

完成前必须给出验证证据。

## Rust

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy -- -D warnings
```

## Qt/C++

```bash
cmake -S . -B build -G Ninja
cmake --build build
```

## 文档

- README 是否更新
- docs/architecture.md 是否需要更新
- docs/protocol.md 是否需要更新
- docs/security.md 是否需要更新

## 安全

- 是否绕过 PermissionManager
- 是否新增危险能力
- 是否记录审计日志
- 是否泄露 token/key/path
