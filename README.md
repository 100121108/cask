# Cask
![GitHub License](https://img.shields.io/github/license/100121108/cask)

Lightweight, self-hosted artifact hosting server. SQLite-backed with token auth and filesystem storage.

## Install

```sh
cargo install --git https://github.com/8124562/cask
```

Or build from source:

```sh
cargo build --release
```

## Quick Start
> [!WARNING]
> First run of cask will leave the create token route unauthorized. Run cask locally and [create your admin token](#bootstrap-token) before exposing the server.

```sh
# Start in foreground
cask run --port 8080

# Or start as a daemon
cask start --port 8080
```

## CLI

```
cask start [--host, --port, --data-dir, --max-upload-size, --log-level]
cask run   [--host, --port, --data-dir, --max-upload-size, --log-level]
cask stop  [--data-dir]
cask pid   [--data-dir]
cask log   [--data-dir, -n, -f]
```

- `start` — daemonize and run in background
- `run` — run in foreground (ctrl+c to stop)
- `stop` — send SIGTERM to a running daemon
- `pid` — print the daemon's PID
- `log` — tail the daemon log file

All runtime data (database, logs, PID file, artifacts) lives under `--data-dir` (default `./data`).

## API

### Bootstrap

The first token is created without authentication and is automatically an admin token:

```sh
curl -X POST http://localhost:8080/v1/tokens \
  -H "Content-Type: application/json" \
  -d '{"label": "admin"}'
```

Save the returned `token` value — it is only shown once.

### Artifacts

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| PUT | `/v1/artifacts/{name}/{version}` | Token | Upload binary |
| GET | `/v1/artifacts/{name}/{version}` | Public | Download binary |
| GET | `/v1/artifacts/{name}` | Public | List versions |
| GET | `/v1/artifacts` | Public | List all artifacts |
| DELETE | `/v1/artifacts/{name}/{version}` | Token | Delete artifact |

```sh
# Upload
curl -X PUT "http://localhost:8080/v1/artifacts/myapp/1.0.0?filename=myapp.tar.gz" \
  -H "Authorization: Bearer $TOKEN" \
  --data-binary @myapp.tar.gz

# Download
curl -O http://localhost:8080/v1/artifacts/myapp/1.0.0
```

### Metadata

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/v1/artifacts/{name}/{version}/meta` | Public | Get key/value pairs |
| PUT | `/v1/artifacts/{name}/{version}/meta` | Token | Set key/value pairs |
| DELETE | `/v1/artifacts/{name}/{version}/meta/{key}` | Token | Remove a key |

### Tokens

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/v1/tokens` | Admin | Create token |
| GET | `/v1/tokens` | Admin | List tokens |
| DELETE | `/v1/tokens/{id}` | Admin | Revoke token |

<a id="bootstrap-token"></a>
```sh
# Create a new token (authorization isnt needed if no tokens exist already)
curl -X POST http://localhost:8080/v1/tokens \
  -H "Content-Type: application/json" \
  -d '{"label": "admin"}'
```

### Stats

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/v1/artifacts/{name}/{version}/stats` | Public | Download count for version |
| GET | `/v1/artifacts/{name}/stats` | Public | Download count across all versions |

### Health

```sh
curl http://localhost:8080/health  # returns "ok"
```
