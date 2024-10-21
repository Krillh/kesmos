
# TODO

## Important

- [x] redo the whole expr module
- [ ] rework simplification and evaluation
  - [x] make `add` and `mul` multi-input instead of binary
  - [x] re-add function support to `expr.rs`
  - [x] add a field to the parsel `Fn` struct to include an optional `(recursive)` after the `fn` keyword
  - [x] change `.simplify()` to return an `Expr` and take in a variable name
  - [ ] change `.evaluate()` to evaluate an Expr & recursive functions

## Maintenance

- [ ] make docs
  - [ ] update `README.md`
  - [x] make spec for the DSL
- [x] make a repo
- [ ] organize files
- [ ] edit if fns are public/private
- [ ] possibly split `expr.rs` into multiple files
- [ ] remove dead code

## Features

- [ ] add support for having things on both sides of an equality
- [x] test multi-threaded support
- [ ] implement `Context::as_fn_x()` in `expr.rs`
- [ ] derivatives
- [ ] integrals

## Display

- [ ] move to egui

## Optional Optimization

- [ ] reduce memory footprint of generated point cloud
  - [ ] convert output into a tuple of `f32` and a new `Term` that doesn't include the `Var` variant
