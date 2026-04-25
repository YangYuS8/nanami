---
name: nanami-cubesandbox-integration
description: 当实现 CubeSandbox/E2B adapter、沙箱任务、代码执行、Shell 执行、文件 I/O、网络策略或沙箱可视化时使用。
---
# Nanami CubeSandbox Integration Skill

CubeSandbox 是安全执行层，Nanami 不直接信任 LLM 生成的代码。

## 原则

1. 默认优先在 CubeSandbox 中执行不可信代码。
2. 创建沙箱前必须明确模板 ID、网络策略、挂载目录。
3. 宿主机目录挂载默认只读。
4. 联网默认关闭，除非用户授权。
5. stdout、stderr、exit code、artifacts 必须完整记录。
6. 沙箱执行失败时，不要自动改用宿主机执行。
7. 沙箱销毁、暂停、恢复都必须进入任务日志。

## UI 必须展示

- sandbox id
- template id
- network policy
- mounts
- command/code
- stdout
- stderr
- artifacts
- status
