# Nanami Permission Model

Nanami has access to sensitive desktop and development capabilities. Agents MUST follow this permission model before implementing any operation that reads, writes, executes, mounts, uploads, downloads, or notifies.

## Core Principle

Default deny.

Current 0.4a status: mock flow only. Permission requests and resolutions are structured and visible, but decisions only record mock state and do not trigger any real operation.

Current 0.4b status: OpenClaw dangerous tool requests are classified and surfaced as `permission.requested` events. Decisions are still non-executing and non-persistent unless a later 0.4c phase changes that behavior.

Current 0.4c status: decision flow + in-memory audit log. `allow_once` / `allow_for_task` / `deny` are stored only in process memory, and audit records are stored only in memory as well.

Current 0.5a status: mock sandbox visualization only. Sandbox mounts, network policy, output, artifacts, and completion can be displayed from mock events, but Nanami still does not call real CubeSandbox, execute commands, mount host directories, read or write artifact files, or enable network access.

Current 0.5b status: OpenClaw sandbox event mapping only. Structured sandbox events from OpenClaw may be displayed in Nanami task streams, but Nanami still does not call real CubeSandbox, execute commands, mount host directories, read or write artifact files, or enable network access.

Current 0.5c status: structured sandbox view + permission/audit link. Sandbox mount and network information can be displayed in a clearer structured UI state, and related audit information can be shown alongside the view, but real mount/network capability still must go through `PermissionManager` in a future phase. Permission decisions still do not trigger sandbox execution.

Current 0.7a status: development workflow mock visualization only. Workflow steps may include mock patch proposal and mock apply-patch waiting-permission states, but no permission decision triggers real file reads, writes, command execution, sandbox execution, or patch application.

Current 0.7b status: mock project metadata + structured workflow state only. Project identity, trust, workflow steps, and patch proposal previews may be displayed more clearly in the UI, but no permission decision triggers real project reads, writes, command execution, sandbox execution, or patch application.

Current 0.7c status: mock test result + patch proposal visualization only. Workflow UI may display richer metadata such as command preview, duration, failed test names, and patch risk level, but no permission decision triggers real project reads, writes, command execution, sandbox execution, or patch application.

Current 0.7d status: permission-gated apply patch mock flow only. Nanami may record a mock permission request for patch application and display a mock apply result, but permission decisions still do not trigger real file writes, patch application, command execution, or sandbox execution.

Current 0.8a status: explicit project selection + manifest-only metadata only. User-triggered project folder selection may return minimal metadata, but Nanami still does not read source files, manifest contents, execute commands, call CubeSandbox, write files, or apply patches.

Current 0.8b status: project trust confirmation + in-memory trust state only. User-confirmed trust for the selected project is stored only in memory and does not by itself permit automatic source reads, file writes, command execution, or CubeSandbox usage.

Current 0.8c status: shallow read-only project structure summary only. Nanami may list first-level project entries for the currently selected trusted project, but still does not read source contents, manifest contents, execute commands, call CubeSandbox, write files, or apply patches.

Current 0.8d status: selected project context + mock workflow only. Nanami may reuse selected project metadata and shallow structure summary inside mock workflow visualization, but still does not read source contents, manifest contents, execute commands, call CubeSandbox, write files, or apply patches.

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
- In 0.5a, mount information may be displayed from mock sandbox events only. No real host directory mount occurs.
- In 0.5c, UI may display mount information in a structured Sandbox View, but this remains visualization only until a future PermissionManager-gated execution phase.

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
- In 0.5a, network policy is visualization-only mock state. No actual network access is enabled.
- In 0.5c, network policy may be shown alongside related audit guidance, but no permission decision enables real network access yet.

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
