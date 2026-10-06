# registry — Guide

Authors and books through `#[derive(Model)]`. Shows the macros crate:
table mapping, validation, and typed queries from plain structs.
API group: no UI.

## Run

```bash
cd apps/api/registry
cargo run
```

Listens on `http://localhost:3009`. Environment:

- `REGISTRY_DB` — sqlite URL, default `sqlite:registry.db`.

## Try it

```bash
cargo test -p registry

curl -X POST localhost:3009/authors \
  -H 'Content-Type: application/json' -d '{"name":"A","email":"a@b.c"}'
curl -X POST localhost:3009/books \
  -H 'Content-Type: application/json' -d '{"author_id":1,"title":"T"}'
curl localhost:3009/authors/1/books
```

Bad emails fail validation before any insert. IDs are integer
autoincrements: the derived `create()` omits `id` and lets the
database assign it.

## Structure

```
src/main.rs             # wiring only: state, router, serve
src/routes/catalog.rs   # authors, books, author-books
src/routes/status.rs    # liveness endpoints
src/models/author.rs    # Author derive plus CreateAuthor
src/models/book.rs      # Book derive plus CreateBook
migrations/             # authors plus books tables
```

Every file stays under 500 lines. Each model lives in its own file
because the derive emits one `columns` module per file.

## Docker

```bash
docker build -t registry apps/api/registry
docker run -p 3009:3009 registry
```
