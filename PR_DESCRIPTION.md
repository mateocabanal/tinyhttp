# Consolidate tinyhttp hot-loop optimizations

## Summary

This patch bundles the hot-loop optimization ideas into one larger branch.

Implemented pieces:

- request-line fast path for routes that do not need `Request`
- direct cached route support through `#[get("/ping", cache)]`
- cached routes pre-render default HTTP response bytes when config headers/gzip are not enabled
- response serialization with `write_vectored`
- automatic `Content-Length`
- no full response-body concatenation before write
- no intermediate `Vec<&str>` for gzip detection
- gzip threshold and `Compression::fast()`
- no byte-sniffing with `infer::get()` in the hot response path
- `std::fs::read` for static file bodies
- `HeaderMap` backed by a small linear vector instead of a `HashMap`
- `HeaderMap::get`/`contains` no longer allocate a lookup `String`
- typed request method/path/version fields
- legacy `get_status_line()` preserved for compatibility
- wildcard lookup avoids cloning the wildcard string
- wildcard extraction uses `strip_prefix` instead of allocating `route.get_path().to_string() + "/"`

## New route syntax

```rust
#[get("/ping", cache)]
fn ping() -> &'static str {
    "pong\n"
}
```

`cache` is intentionally limited to no-argument handlers. If config-level custom headers or gzip are enabled, the route falls back to the normal response path so those config options are still applied.

## Caveats

I could not push/open the PR from ChatGPT because the GitHub write actions disappeared from the available tool list. This patch is therefore provided as replacement files, not an opened PR.

This is a large invasive patch. It should be treated as a first draft for a monolithic optimization PR and validated with:

```bash
cargo fmt
cargo test -p tinyhttp-internal
cargo test -p tinyhttp
cargo clippy --workspace --all-targets
```

The async backend likely needs separate attention; the default sync path is the focus here.
