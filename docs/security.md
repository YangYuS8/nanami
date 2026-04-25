# Nanami Security Policy

This document defines security expectations for agents working on Nanami.

## Security Model

Nanami is a local desktop application that can interact with:

- OpenClaw Gateway.
- User files.
- Clipboard.
- Screenshots.
- Local commands.
- CubeSandbox.
- External model and TTS APIs.

Therefore Nanami MUST treat all agent-generated actions as potentially unsafe.

## Primary Security Principles

1. Default deny.
2. Least privilege.
3. User-visible intent.
4. Structured permission checks.
5. Sandboxed execution by default.
6. No secret leakage.
7. Auditable dangerous actions.
8. No silent fallback to weaker isolation.

## Trust Boundaries

```text
User
  trusted as final authority

nanami-ui
  trusted for display and input only

nanami-core
  trusted for enforcing local policy

OpenClaw
  trusted for agent reasoning, but tool requests still require policy checks

CubeSandbox
  trusted as isolated execution environment, but outputs are untrusted

Generated code
  untrusted

External APIs
  untrusted

Project files
  sensitive user data
```

## Host Execution Policy

Nanami SHOULD avoid host command execution.

When host execution is required:

- Ask permission.
- Show exact command.
- Show working directory.
- Show environment changes.
- Record output.
- Use timeout.
- Do not run destructive commands without explicit confirmation.

Commands involving the following are high risk:

```text
rm
sudo
chmod
chown
dd
mkfs
systemctl
iptables
firewall-cmd
ssh-keygen
curl | sh
wget | sh
pacman/dnf/apt install
docker volume rm
docker system prune
```

Agents MUST treat these as L7 or higher-risk operations.

## CubeSandbox Policy

CubeSandbox SHOULD be used for:

- Running generated code.
- Running tests in isolation.
- Installing dependencies for experiments.
- Network-restricted execution.

CubeSandbox defaults:

```text
network: disabled
host_mount: readonly
auto_destroy: true
```

Nanami MUST NOT automatically fall back to host execution when sandbox execution fails.

## Secret Handling

Agents MUST NOT:

- Print full tokens.
- Store API keys in repository files.
- Include secrets in screenshots or notifications.
- Commit `.env` files with real values.
- Log Authorization headers.

Recommended display format:

```text
sk-...abcd
ghp_...wxyz
```

## Notifications

Desktop notifications MUST NOT include sensitive content by default.

Allowed:

```text
Nanami: task completed
Nanami: permission required
Nanami: sandbox execution failed
```

Avoid:

```text
Full command output with secrets
Full file paths containing private project names
API response bodies
```

## File Write Policy

Before writing to user files, Nanami MUST show:

- target path
- reason
- diff or summary
- rollback possibility

Agents MUST avoid direct overwrite when patch-based modification is possible.

## Dependency Policy

When adding dependencies, agents MUST explain:

- why it is needed
- whether it is maintained
- whether it affects security
- license compatibility
- runtime impact

## Asset Policy

Assets such as Live2D models, fonts, icons, sounds, and SDK binaries MUST be checked for license compatibility before committing.

## Agent Rules

Agents MUST:

1. Use `nanami-permission-review` for risky features.
2. Use `nanami-license-assets` for third-party assets.
3. Use `nanami-release-check` before claiming completion.
4. Add tests for policy-critical logic.
5. Prefer explicit denial over unsafe implicit approval.
