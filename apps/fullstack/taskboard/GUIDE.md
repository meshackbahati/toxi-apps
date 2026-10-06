# taskboard — Guide

Task management API with real auth, sqlite persistence, uploads,
long-poll events, templates, and an OpenAPI spec. Every framework
category in one binary.

## Run

```bash
cd apps/taskboard
cargo run
```

Listens on `http://localhost:3000`. Run from this directory: templates,
migrations, and uploads resolve relative to it. Environment:

- `TASKBOARD_DB` — sqlite URL, default `sqlite:taskboard.db`.
- `TASKBOARD_JWT` — signing secret, default `taskboard-dev-secret`.
- `TOXI_ENV=production` — disables debug error pages.

## Try it

```bash
cargo test -p taskboard
```

```bash
# register and log in
curl -X POST localhost:3000/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"email":"a@b.c","password":"password1","name":"Al"}'
TOKEN=$(curl -s -X POST localhost:3000/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"a@b.c","password":"password1"}' | python3 -c "import json,sys; print(json.load(sys.stdin)['token'])")

# tasks
curl -X POST localhost:3000/tasks -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' -d '{"title":"first"}'
curl localhost:3000/tasks -H "Authorization: Bearer $TOKEN"

# uploads and docs
curl -X POST "localhost:3000/uploads?name=note.txt" --data-binary @file
curl localhost:3000/openapi.json | head -c 200
```

## Structure

```
src/main.rs          # wiring only: config, router, middleware, serve
src/routes.rs        # route table
src/controllers/     # thin handlers: web, tasks, auth, uploads, realtime, status, users, docs
src/services/        # business logic: tasks, auth, uploads
src/validators/      # input checks
src/middleware.rs    # logging layer
src/config.rs        # toxi.toml plus env settings
src/models/          # serde types, no logic
templates/           # HTML with layout inheritance
public/              # static assets
migrations/          # users, tasks, password hash
```

Every file stays under 500 lines. Split by surface when one grows.

## Framework functions used

Responses use `json_response!`; request bodies parse with
`Request::json()`; queries with the `Query` extractor. `serde_json`
appears only for serialization helpers.

## Docker

```bash
docker build -t taskboard apps/taskboard
docker run -p 3000:3000 taskboard
```
