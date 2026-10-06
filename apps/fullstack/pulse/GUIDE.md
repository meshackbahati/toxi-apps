# pulse — Guide

Request metrics dashboard. Shows the utils crate plus a hand-written
tower middleware. Fullstack group: the dashboard is rendered HTML,
`/metrics` is Prometheus text.

## Run

```bash
cd apps/fullstack/pulse
cargo run
```

Listens on `http://localhost:3007`.

## Try it

```bash
cargo test -p pulse

curl localhost:3007/api/status
curl localhost:3007/
curl localhost:3007/metrics
```

Every request passes through the recorder middleware, so the table
fills as you browse.

## Structure

```
src/main.rs                # wiring only: state, router, middleware, serve
src/routes/dashboard.rs    # dashboard, prometheus, status, health
src/middleware/record.rs   # Recorder tower service
templates/                 # layout, dashboard table
```

Every file stays under 500 lines. Responses use `json_response!`.

## Docker

```bash
docker build -t pulse apps/fullstack/pulse
docker run -p 3007:3007 pulse
```
