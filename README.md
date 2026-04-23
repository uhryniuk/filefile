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

The positional argument can also be an `http://` or `https://` URL — the
Filefile is streamed over HTTP and applied without ever touching disk:

```sh
ff https://example.com/Filefile.yaml
```

For safety, `!git` and `!sh` tags in a **remote** Filefile are rejected by
default (a remote Filefile is otherwise a remote-code-execution primitive).
Pass `--allow-remote-ops` to opt in when you trust the source:

```sh
ff --allow-remote-ops https://example.com/Filefile.yaml
```

Local Filefiles run tags unconditionally.

### Filefile syntax

A Filefile is a YAML mapping. The key is the on-disk name; the value decides
what kind of node it is:

| Value                 | Result                                        |
|-----------------------|-----------------------------------------------|
| `"string"`            | file with those contents                      |
| `{ ... }` (mapping)   | directory containing the nested mapping       |
| `null` (empty value)  | empty file                                    |
| `!git <url>`          | `git clone <url>` into this node's path       |
| `!sh "<cmd>"`         | run `<cmd>`; its stdout becomes the file's contents |

### Example

```yaml
hello:
  world: "contents of the file"
  here:
    I: "am"
  empty_file:
scripted:
  marker: !sh "printf hi"
  today: !sh "date -u +%Y-%m-%d"
```

Applying this yields:

```
.
├── hello/
│   ├── world          # "contents of the file"
│   ├── here/
│   │   └── I          # "am"
│   └── empty_file     # 0 bytes
└── scripted/
    ├── marker         # "hi"   (captured stdout)
    └── today          # "2026-04-23"
```

The command runs with cwd set to the node's parent directory, so sibling
files can be referenced by name. Stderr is inherited, so build errors
still surface on your terminal.

### Subcommands

- `ff apply -p <dir> -i <file>` — explicit form. Apply `<file>` into `<dir>`.
- `ff generate -p <dir> -s` — walk a directory tree and print its Filefile on stdout.

### Flags

- `-d`, `--dry-run` — print the planned writes, mkdirs, and command
  invocations without touching the filesystem.
- `-v`, `--verbose` — extra output on stderr.
- `-f`, `--force` — run operations that would otherwise warn.
- `--allow-remote-ops` — permit `!git`/`!sh` in Filefiles fetched over http(s).

## Development

```sh
cargo build
cargo test
cargo test -- --ignored   # also runs the !git network test
```

## License

GPL-3.0. See `LICENSE`.
