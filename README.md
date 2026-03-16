# fretka

Web-search CLI for coding agents. Queries DuckDuckGo, returns clean Markdown results.

## Install

```bash
cargo install fretka
```

## Usage

```bash
fretka "python requests retry with backoff"
fretka -t 3 "typescript extend interface"
fretka -t 10 "kubernetes pod graceful shutdown"
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-t, --top-k <N>` | Return the top *N* results | 5 |
| `-v, --verbose` | Print detailed error messages | off |

## License

MIT
