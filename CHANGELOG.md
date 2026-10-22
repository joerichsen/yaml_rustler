# Changelog

## 0.2.0 (2026-10-22)

### Fixed

- `parse/1` no longer raises `:nif_panicked` on empty input (or input with
  only comments). It returns `{:ok, nil}`.
- `parse/1` no longer raises `:nif_panicked` on `.inf`, `-.inf` and `.nan`
  (in any of their spellings, e.g. `.Inf` or `.NAN`).
  They are returned as `:infinity`, `:neg_infinity` and `:nan`, as the BEAM
  has no such floats.
- Merge keys with a list of mappings (`<<: [*a, *b]`) are now merged instead
  of being silently dropped.
- `parse/1` no longer raises `ArgumentError` on `inf`, `nan` and `infinity`
  without a leading dot. They are returned as strings.
- The Hex package no longer contains stray macOS NIF builds.

### Changed

- Inputs larger than 20 KB are parsed on a dirty CPU scheduler, so parsing a
  large document no longer blocks a normal BEAM scheduler.
- About 10-15% faster, as the parsed document is no longer copied before it
  is converted.
- `rustler` is now an optional dependency, only needed to build the NIF from
  source, and any `rustler_precompiled` `~> 0.8` is accepted. Apps no longer
  get rustler and its HTTP dependencies pulled in.
- `True`, `TRUE`, `False` and `FALSE` are parsed as booleans instead of
  strings, following the YAML 1.2 core schema.
- `.NaN` is parsed as `:nan` instead of the string `".NaN"`.
- Updated to yaml-rust2 0.13 (which also fixes an infinite loop on some short
  documents) and rustler 0.38.
- Added a precompiled NIF for `aarch64-unknown-linux-musl` (Alpine on ARM).

## 0.1.6

- Initial releases with precompiled NIFs.
