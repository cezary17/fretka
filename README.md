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
fretka --fetch "kubernetes pod graceful shutdown"
fretka -f json "rust serde deserialize enum"
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-t, --top-k <N>` | Return the top *N* results (1–100) | 5 |
| `--fetch` | Fetch each result URL and extract its readable content | off |
| `-f, --format <FORMAT>` | Output format: `markdown` or `json` | markdown |
| `-v, --verbose` | Print detailed error messages | off |

## License

MIT
