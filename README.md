# dprint-plugin-typstyle

[![CI](https://github.com/apcamargo/dprint-plugin-typstyle/workflows/CI/badge.svg)](https://github.com/apcamargo/dprint-plugin-typstyle/actions?query=workflow%3ACI)

Adapter plugin for [dprint](https://github.com/dprint/dprint) that formats Typst files via [Typstyle](https://github.com/typstyle-rs/typstyle).

## Install

Add the plugin release URL to your `dprint.json`:

```jsonc
{
  "typstyle": {
    // typstyle's config goes here
  },
  "plugins": [
    "https://plugins.dprint.dev/apcamargo/typstyle-0.2.0.wasm"
  ]
}
```

## Configuration

To add configuration, specify a `"typstyle"` key in your `dprint.json`:

```jsonc
{
  "typstyle": {
    "lineWidth": 100,
    "indentWidth": 2,
    "blankLinesUpperBound": 1,
    "collapseMarkupSpaces": false,
    "reorderImportItems": true,
    "wrapMode": "none",
    "newLineKind": "preserve",
  },
  "plugins": [
    // ...etc...
  ]
}
```

### Properties

| Property               | Type    | Default  | Description                                                                                                                                            |
| ---------------------- | ------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `lineWidth`            | integer | `80`     | Maximum width of each line. Minimum `1`.                                                                                                               |
| `indentWidth`          | integer | `2`      | Number of spaces per indentation level. Minimum `0`.                                                                                                   |
| `blankLinesUpperBound` | integer | `1`      | Maximum number of consecutive blank lines preserved. Minimum `0`.                                                                                      |
| `collapseMarkupSpaces` | boolean | `false`  | Collapse consecutive whitespace in markup into a single space.                                                                                         |
| `reorderImportItems`   | boolean | `true`   | Sort the items of an import statement alphabetically.                                                                                                  |
| `wrapMode`             | string  | `"none"` | Text wrapping mode: `"none"` (do not wrap text), `"fill"` (wrap text to fit within line width), `"sentence"` (place each sentence on its own line), or `"fill-sentence"` (place each sentence on its own line and wrap long sentences to the line width). |
| `newLineKind`          | string  | `"preserve"` | The line ending for generated line breaks: `"auto"` (use the line ending of the last line in the file), `"lf"` (line feed), `"crlf"` (carriage return, line feed), or `"preserve"` (use `"crlf"` only if the file uses it, otherwise `"lf"`). |

## Versioning

This plugin is released independently from Typstyle. The version of `typstyle-core` used by the plugin is listed in [Cargo.toml](https://github.com/apcamargo/dprint-plugin-typstyle/blob/main/Cargo.toml) and is checked for updates by this repository's monthly update workflow.

## Development

Build and test locally:

```sh
# Run unit and spec tests
cargo test

# Run clippy checks
cargo clippy --all-targets --all-features -- -D warnings

# Build Wasm module
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --features wasm --release
```
