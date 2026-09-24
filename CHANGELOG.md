# Changelog

## 0.5.0

Parsing fixes. The output format is unchanged, but some comments now parse
differently:

- A tag on a line of its own (e.g. `@note` with its text on the following
  lines) starts a new tag. Previously the tag line was dropped and its text was
  appended to the previous tag.
- Flag-like tags without any text, such as `@noreturn`, are no longer emitted.
- `@param name` without a description produces a `param:name` tag instead of
  `param:unknown`.
- `@ note` (with a stray space after `@`) is recognized as a tag.
- The character right before `*/` is no longer cut off (`/** Ungag*/`).
- Block comments ending in `**/` no longer leave a trailing newline.
- `///` comments and non-ASCII text are handled; slicing by character index
  could previously panic on multi-byte characters.
- A `//` label directly above a block comment (`// Natives` then `/** ... */`)
  is ignored, and a block comment following `//` lines is no longer lost.
- All leading and trailing blank lines of a tag are trimmed.

Added `Comment::tag` and `Comment::tags_named` helpers and `Default` for `Comment`.
