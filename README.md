# rustyflare

A minimal Cloudflare dynamic DNS updater written in Rust.

It checks the machine's public IPv4 address and, if it differs from a Cloudflare `A` record, updates the record. No zone ID is needed: the zone is found automatically. It runs once and exits; the Docker image (based on `busybox`) runs it in a loop.

## How it works

1. Reads `CF_API_TOKEN` and `CF_DOMAIN` from the environment.
2. Fetches the public IP from `https://ipv4.icanhazip.com` and validates it.
3. Finds the zone that contains the record (the longest matching zone among those the token can see).
4. Reads the `A` record and, only if the IP changed, sends a `PATCH` with the new content. TTL and proxy status are left untouched.

Exit code is `0` when the record is up to date or was updated, and `1` on any error.

## Configuration

| Variable       | Required | Description                                                    |
|----------------|----------|----------------------------------------------------------------|
| `CF_API_TOKEN` | yes      | Cloudflare API token                                           |
| `CF_DOMAIN`    | yes      | Full name of the `A` record to update, e.g. `home.example.com` |
| `CF_INTERVAL`  | no       | Seconds between runs in Docker (default `300`)                 |

The token needs:
- **Zone → DNS → Edit** (to read and update the record)

Exactly one `A` record with that name must already exist. `rustyflare` never creates records, and it stops with an error if it finds more than one.

Example `.env`:

```
CF_API_TOKEN=your_token
CF_DOMAIN=home.example.com
CF_INTERVAL=300
```

## Running with Docker

A prebuilt image is available at `ghcr.io/prrar/rustyflare:latest`.

```yaml
services:
  rustyflare:
    image: ghcr.io/prrar/rustyflare:latest
    container_name: rustyflare
    env_file: .env
    init: true
    restart: unless-stopped
```

```sh
docker compose up -d
```

Or with `docker run`:

```sh
docker run -d --name rustyflare --env-file .env --init --restart unless-stopped \
  ghcr.io/prrar/rustyflare:latest
```

The health check reports `healthy` when the last run exited with `0`, so a bad token or a missing record shows up as `unhealthy` in `docker ps`. `--init` lets `docker stop` shut the container down immediately.

To build the image yourself, clone the repository and run `docker compose up -d --build` (the included compose file builds from the local `Dockerfile`).

## Running without Docker

```sh
cargo build --release
CF_API_TOKEN=your_token CF_DOMAIN=home.example.com ./target/release/rustyflare
```

It runs once and exits; schedule it with cron, a systemd timer, or similar.

## License

See [LICENSE](LICENSE).