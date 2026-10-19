# zipcrawl

[![Crates.io](https://img.shields.io/crates/v/zipcrawl?style=flat-square)](https://crates.io/crates/zipcrawl)
[![CI](https://img.shields.io/github/actions/workflow/status/SirCesarium/zipcrawl/ci.yml?branch=main&style=flat-square)](https://github.com/SirCesarium/zipcrawl/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/SirCesarium/zipcrawl?style=flat-square)]()

Explore and stream ZIP archives without extracting them.

```bash
zipcrawl archive.zip tree                    # directory tree
zipcrawl archive.zip list                    # flat listing
zipcrawl archive.zip cat config.json         # stream file to stdout
zipcrawl archive.zip grep "FATAL" --glob "*.log"
zipcrawl archive.zip find "*.rs" --glob
zipcrawl archive.zip x deploy.sh bash        # pipe file into a command
zipcrawl v2.zip diff --base v1.zip           # compare two archives
```

## Install

```bash
cargo install zipcrawl
```

Without NerdFont icons:

```bash
cargo install zipcrawl --no-default-features --features cli
```

One-liner (detects OS, downloads binary, installs completions, prompts for NerdFonts):

```bash
curl -fsSL https://raw.githubusercontent.com/SirCesarium/zipcrawl/main/scripts/install.sh | bash
```

## Completions

The install script sets these up automatically. Manual:

```bash
# fish
zipcrawl completions fish > ~/.config/fish/completions/zipcrawl.fish

# zsh
zipcrawl completions zsh > /usr/local/share/zsh/site-functions/_zipcrawl

# bash
eval "$(COMPLETE=bash zipcrawl)"
```

Once installed, `archive.zip cat <TAB>` completes files from inside the archive.

## Commands

| Command | Alias | What |
|---------|-------|------|
| `tree` | `t` | Directory tree |
| `list` | `ls`, `l` | Flat entry listing |
| `cat` | | Stream file to stdout (raw) |
| `bat` | | Stream file with syntax highlighting |
| `find` | `fd`, `f` | Find files by regex (or `-g` glob) |
| `grep` | `g` | Search file contents |
| `x` | `exec` | Pipe file content into a command |
| `diff` | `d` | Compare two archives |
| `completions` | | Generate shell completion scripts |

Flags:

| Flag | On | What |
|------|----|------|
| `-s`, `--sizes` | tree, list | Show file sizes |
| `-d`, `--depth` | tree | Max depth (default 4) |
| `-g`, `--glob` | find, grep | Use glob instead of regex |
| `-p`, `--path` | find, grep | Limit to subdirectory |
| `-t`, `--entry-type` | find | `f` (files) or `d` (dirs) |
| `-q`, `--quiet` | x, diff | Suppress archive headers |
| `-m`, `--mode` | diff | `default`, `structure`, `stats`, `full` |

## Piping

`cat` streams raw content to stdout. Pipe into anything — [jq](https://jqlang.org) for JSON, [tyg](https://github.com/SirCesarium/tyg) for type generation from data samples, sha256sum, anything that reads stdin:

```bash
zipcrawl data.zip cat config.json | jq .server.port
zipcrawl releases/*.zip cat data.yaml | tyg --lang typescript
```

## License

MIT
