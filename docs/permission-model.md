# Nanami Permission Model

Nanami has access to sensitive desktop and development capabilities. Agents MUST follow this permission model before implementing any operation that reads, writes, executes, mounts, uploads, downloads, or notifies.

## Core Principle

Default deny.

Current 0.4a status: mock flow only. Permission requests and resolutions are structured and visible, but decisions only record mock state and do not trigger any real operation.

Current 0.4b status: OpenClaw dangerous tool requests are classified and surfaced as `permission.requested` events. Decisions are still non-executing and non-persistent unless a later 0.4c phase changes that behavior.

Current 0.4c status: decision flow + in-memory audit log. `allow_once` / `allow_for_task` / `deny` are stored only in process memory, and audit records are stored only in memory as well.

Nanami MUST ask the user before performing risky actions.

Permission decisions MUST be explicit, scoped, recorded, and revocable where possible.

## Permission Levels

### L0: Chat Only

Allowed:

- Normal chat.
- Displaying static UI.
- Showing existing non-sensitive app state.

No permission prompt required.

### L1: Desktop Context

Examples:

- Read clipboard.
- Capture screenshot.
- Read active window title.
- Send desktop notification.
- Use global shortcut.

Rules:

- Must ask permission before reading clipboard or screenshot.
- Notifications must not reveal sensitive content by default.
- Global shortcuts must be user-configured.

### L2: Read Project Files

Examples:

- Read files in a selected project directory.
- List project structure.
- Read logs selected by the user.

Rules:

- Must show path.
- Must show reason.
- Must limit scope to a selected directory.
- Must not read home directory recursively without explicit approval.

### L3: Write Project Files

Examples:

- Modify source files.
- Create config files.
- Apply patch.
- Update documentation.

Rules:

- Must show diff before writing.
- Must allow user to reject.
- Must record changed paths.
- Must avoid destructive overwrite unless confirmed.

### L4: Execute Local Commands

Examples:

- Run `cargo check`.
- Run `npm test`.
- Run `git status`.
- Run shell command on host.

Rules:

- Host execution is risky.
- Prefer CubeSandbox for untrusted commands.
- Must show command before execution.
- Must show working directory.
- Must show environment changes.
- Must ask permission.
- Must record stdout, stderr, and exit code.

### L5: Mount Host Directory into CubeSandbox

Examples:

- Mount project directory to `/workspace/project`.

Rules:

- Default mount mode is readonly.
- Writable mount requires separate approval.
- Must show host path and sandbox path.
- Must show whether mount is readonly.
- Must record mount in task log.

### L6: Network Access

Examples:

- Download dependencies.
- Access external APIs.
- Run network-enabled tests.
- Allow CubeSandbox outbound network.

Rules:

- Default network mode for CubeSandbox is disabled.
- Must ask before enabling network.
- Must show reason.
- Should support allowlist or denylist.
- Must record network policy.

### L7: Destructive or System-Level Operations

Examples:

- Delete files.
- Modify system config.
- Change shell profile.
- Install host packages.
- Modify SSH keys.
- Modify credential files.
- Change firewall or network settings.

Rules:

- Must require strong confirmation.
- Must show exact operation.
- Must show rollback plan when possible.
- Must not be automated silently.
- Must not be bundled with unrelated permissions.

## Permission Decision Types

Valid decisions:

```text
allow_once
allow_for_task
deny
```

Future decision types MAY include:

```text
allow_for_project
always_deny
```

But persistent permissions MUST be designed carefully and stored securely.

In 0.4a:

- `allow_once` only records a mock one-time decision.
- `allow_for_task` only records a mock task-scoped decision.
- `deny` records explicit rejection.
- No actual file read/write, command execution, screenshot, clipboard, or network escalation is performed as a result of these decisions.

In 0.4c:

- `allow_once` / `allow_for_task` / `deny` still only record decisions.
- Audit records are generated for `permission_requested` and `permission_resolved`.
- Audit records are queryable from memory only.
- No persistent storage or secure vault integration is implemented yet.

OpenClaw dangerous tool classification guide:

```text
read_file/filesystem.read/project.read -> L2
write_file/filesystem.write/apply_patch -> L3
shell/command.run/local.exec/process.spawn -> L4
sandbox.mount/cubesandbox.mount -> L5
http.request/network.fetch/download/dependency.install -> L6
delete_file/filesystem.delete/system.modify/package.install/service.modify -> L7
unknown dangerous-looking tool -> L7
```

## Permission Request Shape

Permission requests SHOULD include:

```json
{
  "permission_id": "perm_001",
  "task_id": "task_001",
  "level": "L2",
  "action": "filesystem.read",
  "target": "/home/user/Code/nanami",
  "reason": "需要读取项目结构以分析构建错误",
  "scope": "task",
  "expires": "task_completed"
}
```

## Sensitive Data Rules

Sensitive values include:

- API keys.
- Tokens.
- Cookies.
- Authorization headers.
- Gateway secrets.
- SSH keys.
- Private certificates.
- Passwords.
- Local absolute paths when not needed for display.

Rules:

1. Logs MUST NOT contain full secrets.
2. UI MUST mask secrets by default.
3. Agents MUST NOT print Authorization headers.
4. Agents MUST NOT commit secrets.
5. Storage MUST avoid plaintext secrets when platform keychain is available.

## Audit Log

Dangerous operations SHOULD produce audit records.

Audit records SHOULD include:

- timestamp
- task_id
- permission_id
- action
- target
- decision
- actor
- result
- redacted command or path summary

## Agent Rules

Agents MUST:

1. Classify every new capability by permission level.
2. Add permission prompts before risky operations.
3. Prefer sandbox execution for generated code.
4. Never bypass PermissionManager.
5. Never hide a dangerous operation inside a harmless-looking action.
