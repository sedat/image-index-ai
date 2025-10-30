# Repository Guidelines

## Project Structure & Module Organization
- Backend Rust code lives in `src/`: `routes/` for Axum handlers, `models/` for SQLx DTOs, `services/` for outbound clients, and `state.rs` for shared wiring. DB migrations are orchestrated via `migrations.rs` and the `migrations/` directory.
- Static assets belong in `images/`; avoid committing generated binaries or large artifacts. The Next.js UI sits in `frontend/src/`, while shared assets for the UI live in `frontend/public/`. Keep `image-index-frontend/` reserved for future variants.

## Build, Test, and Development Commands
- `make run` (backend): boots the API with tracing enabled; mirrors `cargo run`.
- `make check` or `cargo check`: compiles Rust sources without executing tests.
- `cargo test`: runs backend unit and integration suites alongside the modules they cover.
- `cd frontend && pnpm install`: installs UI dependencies once per environment.
- `pnpm dev`: launches the Next.js dev server; `pnpm build` compiles the production bundle; `pnpm lint` enforces ESLint rules.

## Coding Style & Naming Conventions
- Run `make fmt` before commits; the repo assumes default `rustfmt` (4-space indent, `snake_case` modules).
- Frontend code favors functional components, `camelCase` variables, and `PascalCase` component names. Rely on ESLint and Prettier via `pnpm lint` to catch deviations.
- Co-locate component styles and respect existing folder boundaries when adding features.

## Testing Guidelines
- Keep Rust tests in `#[cfg(test)]` modules beside the code they exercise; reserve `tests/` for larger integration suites named `*_test.rs`.
- Leverage SQLx query checks or mocks when touching database interactions.
- Frontend behavior tests are not yet wired; document any new scripts and add them to CI if introduced.

## Commit & Pull Request Guidelines
- Follow the Git history: single-line, lowercase imperative summaries (e.g., `add image upload route`).
- Group related changes together and call out follow-ups or debt in the PR body.
- PRs should list setup steps (migrations, `.env` keys), link issues when available, and include screenshots or curl snippets for API/UI changes.

## Environment & Configuration Tips
- Mirror the keys read in `main.rs`: `DATABASE_URL`, `BIND_ADDR`, and LM Studio endpoints; store them in a local `.env`.
- Run `sqlx migrate run` (or the helper in `migrations.rs`) before testing against Postgres.
- Keep secrets out of version control and clear large generated assets before committing.
