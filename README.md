# rustyflare

A minimal dynamic DNS updater for Cloudflare, written in Rust. Docker image is also very small, based on `busybox`.

No `zone id` is needed, only a CloudFlare API token with `DNS EDIT` permission and the domain to be updated.

It checks the machine's public IPv4 address and, if it differs from the content of a Cloudflare `A` record, updates the record. It runs once and exits; scheduling is left to something else (the included Docker setup runs it in a loop).

## How it works

1. Reads its configuration from environment variables.
2. Finds the zone that contains the record (or uses `CF_ZONE_ID` if set).
3. Fetches the current public IP from `https://ipv4.icanhazip.com`.
4. Compares it with the record's content and sends a `PATCH` only if they differ.

Exit code is `0` when the record is up to date or was updated, and `1` on any error (missing configuration, network or API failure, record not found).

## Configuration

| Variable       | Required | Description                                                        |
|----------------|----------|--------------------------------------------------------------------|
| `CF_API_TOKEN` | yes      | Cloudflare API token                                               |
| `CF_DOMAIN`    | yes      | Full name of the `A` record to update, e.g. `home.example.com`     |
| `CF_ZONE_ID`   | no       | Zone ID; skips the zone lookup when set                            |
| `CF_INTERVAL`  | no       | Seconds between runs in Docker (default `300`)                     |

The API token needs this permission:
- **Zone → DNS → Edit** (to read and update the record)

The record must already exist; `rustyflare` only updates it, it never creates one.

## Running with Docker Compose

Create a `.env` file next to `docker-compose.yml`:

```
CF_API_TOKEN=your_token
CF_DOMAIN=home.example.com
CF_INTERVAL=300
```

Then:

```sh
docker compose up -d --build
docker compose logs -f rustyflare
```

The container runs the program every `CF_INTERVAL` seconds. Its health check reports `healthy` when the last run exited with `0`, so a broken token or a missing record shows up as `unhealthy` in `docker ps`.

## Running with Docker

Without Compose, using the same `.env` file:

```sh
docker build -t rustyflare .
docker run -d \
  --name rustyflare \
  --env-file .env \
  --init \
  --restart unless-stopped \
  rustyflare
docker logs -f rustyflare
```

`--init` lets `docker stop` shut the container down immediately instead of waiting for the timeout.

## Running locally

```sh
cargo build --release
CF_API_TOKEN=your_token CF_DOMAIN=home.example.com ./target/release/rustyflare
```

## License

See [LICENSE](LICENSE).
