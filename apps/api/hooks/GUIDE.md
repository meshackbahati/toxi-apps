# hooks — Guide

In-process plugin pipeline. Shows the plugin crate: audit counting
plus maintenance short-circuit. API group: no UI.

## Run

```bash
cd apps/api/hooks
cargo run
```

Listens on `http://localhost:3008`.

## Try it

```bash
cargo test -p hooks

curl localhost:3008/demo
curl -X POST localhost:3008/maintenance
curl localhost:3008/demo
curl localhost:3008/plugins
```

The first `/demo` serves and audits once. After toggling
maintenance, `/demo` short-circuits with the maintenance reason
while the audit count keeps climbing.

## Structure

```
src/main.rs                # wiring only: manager, router, serve
src/routes/demo.rs         # demo, maintenance toggle, plugin list
src/routes/status.rs       # liveness endpoints
src/plugins/audit.rs       # counts PreRequest hooks
src/plugins/maintenance.rs # Stop while the flag is set
```

Every file stays under 500 lines. Responses use `json_response!`.
Only in-process plugins are real: file loading is an acknowledged
stub in the framework.

## Docker

```bash
docker build -t hooks apps/api/hooks
docker run -p 3008:3008 hooks
```
