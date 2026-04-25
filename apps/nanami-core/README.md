# nanami-core

Rust daemon skeleton for Nanami local control logic.

## Run

```bash
cargo run -p nanami-core
```

## Health Check

```text
GET http://127.0.0.1:17878/health
```

Expected response:

```json
{
  "status": "ok",
  "protocol_version": "0.1"
}
```
