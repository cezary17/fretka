# fretka — Web Search Skill

## Purpose

Search the public web or arXiv for up-to-date information, documentation, research papers, or context missing from the local codebase and your training data.

## Usage

```bash
fretka "<search query>"
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-t, --top-k <N>` | Number of results to return | 5 |
| `-e, --engine <ENGINE>` | Search engine: `duckduckgo` or `arxiv` | duckduckgo |
| `--sort <SORT>` | Sort order (arxiv only): `relevance`, `submitted-date`, `last-updated-date` | relevance |
| `--fetch` | Fetch result URLs and extract readable content | off |
| `-f, --format <FMT>` | Output format: `markdown` or `json` | markdown |
| `-v, --verbose` | Show detailed error messages | off |

### Examples

```bash
# Find documentation (DuckDuckGo, default)
fretka "python requests retry with backoff"

# Narrow a focused lookup
fretka -t 3 "typescript extend interface with optional properties"

# Fetch and extract full page content
fretka --fetch -t 3 "dom_smoothie readability crate usage"

# Search arXiv for papers
fretka --engine arxiv "ti:attention AND cat:cs.CL"

# Search arXiv by author
fretka --engine arxiv "au:hinton AND ti:deep learning" -t 5

# Search arXiv by category, sorted by date
fretka --engine arxiv "cat:cs.AI" --sort submitted-date -t 10

# Fetch full PDF content from arXiv papers
fretka --engine arxiv "ti:transformer" -t 2 --fetch
```

## arXiv Query Syntax

When using `--engine arxiv`, queries are passed directly to the arXiv API. Use field prefixes and boolean operators:

### Field Prefixes

| Prefix | Field |
|--------|-------|
| `ti:` | Title |
| `au:` | Author |
| `abs:` | Abstract |
| `cat:` | Subject category |
| `all:` | All fields |

### Boolean Operators

Combine terms with `AND`, `OR`, `ANDNOT`:

```bash
fretka --engine arxiv "ti:attention AND cat:cs.CL"
fretka --engine arxiv "au:bengio AND ti:deep learning"
fretka --engine arxiv "(cat:cs.LG OR cat:cs.AI) AND ti:transformer"
```

### Common Categories

- `cs.AI` — Artificial Intelligence
- `cs.LG` — Machine Learning
- `cs.CL` — Computation and Language (NLP)
- `cs.CV` — Computer Vision
- `cs.SE` — Software Engineering
- `math.*` — Mathematics
- `physics.*` — Physics

## Output Format

Returns a numbered markdown list. Each entry pairs a linked title with content:

```
1. [Page Title](https://example.com/page)

   Short description or snippet from the page.
```

arXiv results include metadata:

```
1. [Paper Title](https://arxiv.org/abs/1234.5678)

   **Authors:** Alice, Bob
   **Categories:** cs.AI, cs.LG
   **Published:** 2026-01-15T00:00:00Z

   Abstract text here...
```

With `--fetch`, DuckDuckGo results get extracted page content. arXiv results get extracted PDF text (falling back to abstract if PDF extraction fails).

## When to Use

- **Unfamiliar APIs or libraries** — find docs, examples, or changelogs.
- **Error messages** — paste the exact error string to surface known fixes.
- **Best practices** — find recommended patterns when the right approach is unclear.
- **Research papers** — use `--engine arxiv` for academic papers and preprints.
- **Deep reading** — use `--fetch` when snippets aren't enough.

## Prefer Other Tools Instead When

- The answer exists in the local codebase — use grep/find.
- Your training data answers the question with high confidence.
- The query requires authentication — fretka searches only public results.

## Writing Effective Queries

- Name the language or framework: `"reqwest timeout configuration rust"` outperforms `"http timeout"`.
- Quote exact error messages for precise matches.
- Use `-t 3` for targeted lookups, `-t 10` for broad exploration.
- Use `--fetch` with a low `-t` (2–3) to avoid fetching too many pages/PDFs.
- For arXiv, use field prefixes for precision: `"ti:attention"` beats `"attention"`.
