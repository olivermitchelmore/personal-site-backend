# website-backend

Axum backend for [olliemitchelmore.com](https://olliemitchelmore.com).

## Requirements

- Rust
- PostgreSQL
- Env: `DATABASE_URL`, `FRONTEND_DOMAIN` (CORS origin, e.g. `https://olliemitchelmore.com`)

## API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| GET | `/thoughts` | List thoughts (x, y, z, thought) |
| POST | `/thought-submission` | Submit thought (JSON: `x`, `y`, `thought`). Max 50 chars, 3 per IP per 24h. Coords clamped 0–1000. |
| POST | `/contact-submission` | Submit contact form (form: `email`, `name`, `message`). Max 20k chars message, 15 per IP per 24h. Redirects to thank-you page. |

Thoughts and contact submissions are sanitized with ammonia. IP from `cf-connecting-ip` when behind Cloudflare, else connection IP.

## Data

- **positions**: thoughts; background job deletes rows older than 7 days (runs daily).
- **contact**: contact form submissions (no auto-delete).
