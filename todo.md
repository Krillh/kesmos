
# TODO

## Maintenance

- [ ] remove dead code
- [ ] comment code
  - [ ] comment `expr.rs`
  - [ ] comment `parse.rs`
  - [ ] comment `convert.rs`
- [ ] make tests
- [ ] make docs
  - [ ] update `README.md`
- [ ] make a repo
- [ ] edit if fns are public/private
- [ ] possibly split `expr.rs` into multiple files

## Features

- [ ] add support for having things on both sides of an equality
- [x] test multi-threaded support
- [ ] implement `Context::as_fn_x()` in `expr.rs`
- [ ] derivatives
- [ ] integrals

## Display

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
