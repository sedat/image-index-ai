Image Index AI

Tag and search your images using a local vision+language model. The backend stores tags per image; the UI lets you query in natural language.

Quick start

- Start Postgres: `docker compose up -d postgres`
- Run API: `make run` (serves `http://localhost:8080`)
- Run UI: `cd frontend && pnpm install && pnpm dev` (at `http://localhost:3000`)

Notes

- Defaults: `DATABASE_URL=postgres://user:password@localhost/image-index`, `BIND_ADDR=0.0.0.0:8080`.
- Use a local model provider (e.g., LLaVA + Llama via Ollama or LM Studio).
- The upload page supports multi-select uploads (max 4 concurrent) with per-file progress and retry.

Features

- Upload images; auto-tag via vision model
- Upload multiple images at once with per-file progress tracking
- Browse and filter by tags in the UI
- Natural-language search mapped to tags
- Vector search via embeddings (pgvector)

Prerequisites

- Rust + Cargo, Node.js + pnpm
- Docker (for Postgres with pgvector): `docker compose up -d postgres`
- One local model runtime:
  - LM Studio API at `http://localhost:1234/v1` (default)

Configuration

- Server bind: `BIND_ADDR` (default `0.0.0.0:8080`)
- Database URL: defaults to `postgres://user:password@localhost/image-index`
- LM Studio (env overrides):
  - `LMSTUDIO_BASE_URL` (default `http://localhost:1234/v1`)
  - `LMSTUDIO_IMAGE_MODEL` (default `qwen/qwen3-vl-4b`)
  - `LMSTUDIO_TEXT_MODEL` (default `qwen/qwen3-vl-4b`)
  - `LMSTUDIO_EMBED_MODEL` (default `text-embedding-nomic-embed-text-v1.5`)
  - `LMSTUDIO_TEMPERATURE` (default `0.2`)
  - `BATCH_UPLOAD_MAX_CONCURRENCY` (default `4`)

API

- List images: `GET /api/images?tags=comma,separated,tags`
- Upload image: `POST /api/images` with JSON `{ file_name, image_base64, mime_type? }`
- Batch upload: `POST /api/images/batch` with JSON `{ items: [{ file_name, image_base64, mime_type? }, ...] }` (processes concurrently; default max 4)
- Tag search: `POST /api/images/search` with `{ query }`
- Vector search: `POST /api/images/semantic-search` with `{ query, limit?, max_distance? }`

Examples

- List all: `curl http://localhost:8080/api/images`
- Search by tags: `curl "http://localhost:8080/api/images?tags=beach,summer"`
- Natural query: `curl -X POST http://localhost:8080/api/images/search -H 'content-type: application/json' -d '{"query":"photos of sunny beaches"}'`
- Vector search: `curl -X POST http://localhost:8080/api/images/semantic-search -H 'content-type: application/json' -d '{"query":"rocky mountains at sunset","limit":24}'`
- Batch upload: `curl -X POST http://localhost:8080/api/images/batch -H 'content-type: application/json' -d '{"items":[{"file_name":"a.jpg","image_base64":"...","mime_type":"image/jpeg"},{"file_name":"b.png","image_base64":"...","mime_type":"image/png"}]}'`

Project structure

- Backend: `src/`
  - Routes: `src/routes/images.rs`
  - Models: `src/models/`
  - Services: `src/services/` (`LmStudioClient` and embeddings)
  - App state: `src/state.rs`
  - Migrations helper: `src/migrations.rs` (runs on startup)
- UI: `frontend/src/` (Next.js)
- Static images: `images/` (served at `/images/...`)

Troubleshooting

- Ensure LM Studio or Ollama is running and models are downloaded
- If embeddings time out, the API falls back to tag-based search
- Run formatting and checks: `make fmt` and `make check`
