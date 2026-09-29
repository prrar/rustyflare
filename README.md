# rustyflare

A minimal dynamic DNS updater for Cloudflare, written in Rust. The Docker image is very small, based on `busybox`.

No zone ID is needed, only a Cloudflare API token with DNS Edit permission and the record to be updated.

It checks the machine's public IPv4 address and, if it differs from the content of a Cloudflare `A` record, updates the record. It runs once and exits; scheduling is left to something else (the Docker image runs it in a loop).

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

Every method below reads the configuration from a `.env` file:

```
CF_API_TOKEN=your_token
CF_DOMAIN=home.example.com
CF_INTERVAL=300
```

In Docker, the container runs the program every `CF_INTERVAL` seconds. Its health check reports `healthy` when the last run exited with `0`, so a broken token or a missing record shows up as `unhealthy` in `docker ps`.

## Installation: prebuilt image

A prebuilt image is published to the GitHub Container Registry as `ghcr.io/prrar/rustyflare:latest`. Nothing needs to be cloned or compiled.

### Docker Compose

Create a `docker-compose.yml` next to your `.env`:

```yaml
services:
  rustyflare:
    image: ghcr.io/prrar/rustyflare:latest
    container_name: rustyflare
    environment:
      - CF_API_TOKEN=${CF_API_TOKEN}
      - CF_DOMAIN=${CF_DOMAIN}
      #- CF_ZONE_ID=${CF_ZONE_ID} # optional
      #- CF_INTERVAL=300 # optional, default 300 seconds
    init: true
    restart: unless-stopped
```

Then:

```sh
docker compose up -d
docker compose logs -f rustyflare
```

To update to the latest image:

```sh
docker compose pull
docker compose up -d
```

### Docker run

```sh
docker run -d \
  --name rustyflare \
  --env-file .env \
  --init \
  --restart unless-stopped \
  ghcr.io/prrar/rustyflare:latest
docker logs -f rustyflare
```

`--init` lets `docker stop` shut the container down immediately instead of waiting for the timeout.

## Installation: building it yourself

Clone the repository first and put your `.env` in its root:

```sh
git clone https://github.com/prrar/rustyflare.git
cd rustyflare
```

### Docker Compose

The included `docker-compose.yml` builds the image from the local `Dockerfile`:

```sh
docker compose up -d --build
docker compose logs -f rustyflare
```

### Docker run

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

### Without Docker

```sh
cargo build --release
CF_API_TOKEN=your_token CF_DOMAIN=home.example.com ./target/release/rustyflare
```

This runs once and exits; schedule it with cron, a systemd timer, or similar.

## License

See [LICENSE](LICENSE).
