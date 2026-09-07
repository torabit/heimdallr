---
paths:
  - "**/*.rs"
---

# Error handling

## Library layer vs application layer

- Domain error types are defined with `thiserror`, one per module or domain boundary.
- `anyhow` appears only at the application boundary — `main.rs` and top-level orchestration.
- Everything below that returns typed errors. `anyhow::Result` is not allowed there.

## Designing error types

- One error enum per module or domain boundary.
- Every variant carries enough context to act on.
- Use `#[from]` for automatic conversion from source errors.
- Reserve `#[error(transparent)]` for opaque catch-all variants.

## Forbidden

- `panic!()` in library code — return a `Result`.
- String-based errors in the library layer, e.g. `anyhow!("something failed")`.
- Swallowing a meaningful error with `.unwrap_or_default()`.

## Adding context

- Use `.context()` / `.with_context()` when propagating upward.
- A message must say *which operation failed* and *on what input*.
- Prefer a structured variant over string interpolation when the error will be handled programmatically.

## Never guess a mode

A failed detection is a third outcome, distinct in the type from dark and light. It is never
rounded to light: a caller told "light" applies a theme nobody asked for, and has no way to
find out it was told a default.

Say what failed. "no system theme on this host" and "reg.exe: No such file or directory" send
the reader to different places; "detection failed" sends them nowhere. The caller must not have
to infer the cause from an exit code.
