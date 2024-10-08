
# TODO

## Important

- [ ] rework simplification and evaluation
  - [ ] add a field to the parsel `Fn` struct to include an optional `(recursive)` after the `fn` keyword
  - [ ] change `.simplify()` to return an `Expr` and take in a variable name
  - [ ] make a `set_var` function for the new struct `VarContext` that expands a variable to some `Expr`
  - [ ] change `.evaluate()` to evaluate an Expr & recursive functions

## Maintenance

- [ ] comment code
  - [ ] comment `expr.rs`
  - [ ] comment `parse.rs`
  - [ ] comment `convert.rs`
- [ ] make tests
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

- [ ] decide if i want to completely change the way things are displayed
- [ ] display graphs
  - [x] draw lines
  - [x] draw axis
  - [ ] label axis
  - [ ] add arrows to axis
  - [ ] draw tickmarks
  - [ ] label tickmarks
- [x] be able to move/rotate graphs
- [x] continuously update graphs
- [ ] implement editor
- [ ] make web runnable

## Optional Optimization

- [ ] reduce memory footprint of generated point cloud
  - [ ] convert output into a tuple of `f32` and a new `Term` that doesn't include the `Var` variant
