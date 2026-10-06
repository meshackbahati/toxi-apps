# toxi-apps

<div align="center">

<img src="docs/logo/toxi.svg" width="200" alt="Toxi Logo">

Working applications built with the [Toxi](https://github.com/toxi-rs/toxi)
web framework, plus the framework documentation hub.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

</div>

---

## Applications

| Group | App | Guide |
| ----- | --- | ----- |
| fullstack | [taskboard](apps/fullstack/taskboard) — tasks with auth, sqlite, uploads, events, templates, OpenAPI | [GUIDE.md](apps/fullstack/taskboard/GUIDE.md) |
| fullstack | [pulse](apps/fullstack/pulse) — request metrics dashboard plus Prometheus | [GUIDE.md](apps/fullstack/pulse/GUIDE.md) |
| fullstack | [pastebin](apps/fullstack/pastebin) — sanitized pastes with rendered preview | [GUIDE.md](apps/fullstack/pastebin/GUIDE.md) |
| fullstack | [blog](apps/fullstack/blog) — rendered posts with auth-gated writing | [GUIDE.md](apps/fullstack/blog/GUIDE.md) |
| api | [registry](apps/api/registry) — authors and books through Model derive | [GUIDE.md](apps/api/registry/GUIDE.md) |
| api | [todo](apps/api/todo) — todos over REST plus live GraphQL playground | [GUIDE.md](apps/api/todo/GUIDE.md) |
| api | [hooks](apps/api/hooks) — in-process plugin pipeline demo | [GUIDE.md](apps/api/hooks/GUIDE.md) |
| api | [notifier](apps/api/notifier) — queued welcome emails over SMTP | [GUIDE.md](apps/api/notifier/GUIDE.md) |
| api | [shortlink](apps/api/shortlink) — cache-first redirects, rate limits, stats | [GUIDE.md](apps/api/shortlink/GUIDE.md) |
| serverless | [echo](apps/serverless/echo) — stateless webhook plus warmth probe | [GUIDE.md](apps/serverless/echo/GUIDE.md) |

Every app depends on the published crates, follows the modular
`routes / models / services` layout, and keeps each file under 500 lines.
Each app ships its own guide with run, test, structure, and deploy notes.

## Framework crates

All crates live in the [`toxi-rs`](https://github.com/toxi-rs)
organization, one repo per crate: `toxi`, `toxi-core`, `toxi-cli`,
`toxi-auth`, `toxi-cache`, `toxi-config`, `toxi-db`, `toxi-graphql`,
`toxi-middleware`, `toxi-openapi`, `toxi-plugin`, `toxi-queue`,
`toxi-realtime`, `toxi-security`, `toxi-storage`, `toxi-template`,
`toxi-testing`, `toxi-utils`, `toxi-macros`, `toxi-mail`.

```toml
[dependencies]
toxi = "3"
tokio = { version = "1", features = ["full"] }
```

## Documentation

- [`docs/`](docs) — framework guides: getting started, core concepts,
  authentication, database, middleware, deployment, migrations.
- [`examples/`](examples) — single-file code samples.

## Run an app

```bash
cd apps/fullstack/taskboard
cargo run
```
