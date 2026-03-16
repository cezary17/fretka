# fretka — Web Search Skill

## Purpose

Search the public web for up-to-date information, documentation, or context missing from the local codebase and your training data.

## Usage

```bash
fretka "<search query>"
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-t, --top-k <N>` | Number of results to return | 5 |
| `-v, --verbose` | Show detailed error messages | off |

### Examples

```bash
# Find documentation
fretka "python requests retry with backoff"

# Narrow a focused lookup
fretka -t 3 "typescript extend interface with optional properties"

# Explore a broad topic
fretka -t 10 "kubernetes pod graceful shutdown"
```

## Output Format

Returns a numbered markdown list. Each entry pairs a linked title with a text snippet:

```
1. [Page Title](https://example.com/page)

   Short description or snippet from the page.

2. [Another Result](https://example.com/other)

   Another snippet of text.
```

## When to Use

- **Unfamiliar APIs or libraries** — find docs, examples, or changelogs.
- **Error messages** — paste the exact error string to surface known fixes.
- **Best practices** — find recommended patterns when the right approach is unclear.
- **Version-specific details** — find release notes, migration guides, or compatibility tables.

## Prefer Other Tools Instead When

- The answer exists in the local codebase — use grep/find.
- Your training data answers the question with high confidence.
- The query requires authentication — fretka searches only public results.

## Writing Effective Queries

- Name the language or framework: `"reqwest timeout configuration rust"` outperforms `"http timeout"`.
- Quote exact error messages for precise matches.
- Use `-t 3` for targeted lookups, `-t 10` for broad exploration.
