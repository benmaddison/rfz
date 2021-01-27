# `rfz`

An indexer and metadata viewer for repositories of IETF documents synced to the
local file system.

[![crates.io](http://meritbadge.herokuapp.com/rfz)](https://crates.io/crates/rfz)
[![ci](https://github.com/benmaddison/rfz/workflows/ci/badge.svg)](https://github.com/benmaddison/rfz/actions?query=workflow%3Aci)
[![audit](https://github.com/benmaddison/rfz/workflows/audit/badge.svg)](https://github.com/benmaddison/rfz/actions?query=workflow%3Aaudit)
[![codecov](https://codecov.io/gh/benmaddison/rfz/branch/master/graph/badge.svg?token=ktq7dOYD49)](https://codecov.io/gh/benmaddison/rfz)

## Installation

Install via cargo:

```bash
$ cargo install rfz
```

`rsync` is required in order to use `rfz sync`.

## Usage

See `rfz --help` for basic command-line usage.

`rfz` expects to find a directory containing a local mirror of the
`rsync.tools.ietf.org::tools.html` `rsync` target.

The path to this directory can be set with `--dir` and defaults to
`${XDG_DATA_DIR:-${HOME}/.local/share}/rfz`.

`rfz sync` will create the directory if it does not already exist, and call
`rsync` to retrieve the contents.

Example `systemd` units to run `rfz sync` every hour are included in `extras/`.

`rfz` can be used standalone, but is designed to be used along side `fzf` and a
text-mode browser (e.g. `lynx` or `w3m`):

```bash
#!/usr/bin/env bash
rfz index | fzf \
            # trim the path from the 'fzf' display
            --with-nth=2.. \
            # show coloured output
            --ansi \
            # show the document metadata in the preview
            --preview 'rfz summary {1}' \
          | lynx -
```
