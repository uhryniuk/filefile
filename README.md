# filefile

Build a directory tree from a YAML description.

`filefile` (binary: `ff`) reads a YAML mapping and materializes it on disk.
Nested mappings become directories, string values become file contents, and
YAML tags like `!git` and `!sh` run commands in-place.

## Install

```sh
cargo install --path .
```

This places `ff` on your `PATH` (via `~/.cargo/bin`).

## Usage

### Apply a Filefile

```sh
ff Filefile.yaml
```

Materializes the tree described by `Filefile.yaml` in the current directory.

### Filefile syntax

A Filefile is a YAML mapping. The key is the on-disk name; the value decides
what kind of node it is:

| Value                 | Result                                        |
|-----------------------|-----------------------------------------------|
| `"string"`            | file with those contents                      |
| `{ ... }` (mapping)   | directory containing the nested mapping       |
| `null` (empty value)  | empty file                                    |
| `!git <url>`          | `git clone <url>` into this node's path       |
| `!sh "<cmd>"`         | run `<cmd>` with cwd = the node's parent dir  |

### Example

```yaml
hello:
  world: "contents of the file"
  here:
    I: "am"
  empty_file:
scripted:
  marker: !sh "printf hi > marker"
```

Applying this yields:

```
./hello/world        # "contents of the file"
./hello/here/I       # "am"
./hello/empty_file   # 0 bytes
./scripted/marker    # "hi"
```

### Subcommands

- `ff apply -p <dir> -i <file>` — explicit form. Apply `<file>` into `<dir>`.
- `ff generate -p <dir> -s` — walk a directory tree and print its Filefile on stdout.

### Flags

- `-d`, `--dry-run` — print the planned writes, mkdirs, and command
  invocations without touching the filesystem.
- `-v`, `--verbose` — extra output on stderr.
- `-f`, `--force` — run operations that would otherwise warn.

## Development

```sh
cargo build
cargo test
cargo test -- --ignored   # also runs the !git network test
```

## License

GPL-3.0. See `LICENSE`.
