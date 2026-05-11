# Mega hot-loop heap allocation optimization

## Summary

This is the consolidated heap-allocation pass discussed in chat.

It combines six related changes into one large PR:

1. Borrowed request-line fast path
   - cached/no-request routes parse the request line from a stack buffer
   - avoids per-connection `BufReader` allocation in the fast path
   - avoids allocating `path` and `version` before route lookup

2. `Route::to_res_no_req`
   - no-argument handlers no longer need a fake `Request`
   - removes unnecessary `Request::default()`/status-line work from no-request routes

3. `SmallVec` for hot small buffers
   - request `HeaderMap`
   - response headers
   - response serialized header bytes
   - full-parse request/header scratch buffer

4. Lazy legacy status line
   - `Request::new_parts` no longer eagerly builds `Vec<String>`
   - `get_status_line()` lazily constructs `[String; 3]` only if legacy code asks for it

5. `Cow<'static, str>` response metadata
   - common status lines and MIME strings can be borrowed static data
   - reduces repeated heap allocation for `Response::new()`, `.mime("...")`, and common headers

6. Macro-specialized `&'static str`
   - handlers returning `&'static str` now generate `Response::body_static(...)`
   - works with `#[get("/ping", cache)]`
   - avoids copying static string bodies into `Vec<u8>` for non-cached no-arg handlers too

## Requested cache syntax

```rust
#[get("/ping", cache)]
fn ping() -> &'static str {
    "pong\n"
}
```

## Notes

This is intentionally a mega patch. It changes public `Response` field types from `String`/`Vec<(String, String)>` to `Cow`/`SmallVec`-based storage. If that public-field API compatibility is too aggressive, split out the `Cow` change into a separate PR.

## Suggested validation

```bash
cargo fmt
cargo test -p tinyhttp-internal
cargo test -p tinyhttp
cargo clippy --workspace --all-targets
rm -rf target/criterion
cargo bench --bench create_req
```

For the cached fast path, benchmark:

```rust
#[get("/helloworld", cache)]
fn get() -> &'static str {
    "got it"
}
```
