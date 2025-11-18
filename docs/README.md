# Colony Documentation

This directory contains the source files for Colony's documentation, built with [mdBook](https://rust-lang.github.io/mdBook/).

## Building the Documentation

### Prerequisites

Install mdBook:
```bash
cargo install mdbook
```

### Build

```bash
cd docs
mdbook build
```

The generated HTML will be in `docs/book/`.

### Serve Locally

```bash
cd docs
mdbook serve
```

Then open http://localhost:3000 in your browser.

### Watch Mode

mdBook automatically rebuilds on file changes when using `serve`:
```bash
cd docs
mdbook serve --open
```

## Documentation Structure

```
docs/
├── book.toml              # mdBook configuration
├── src/                   # Documentation source
│   ├── SUMMARY.md         # Table of contents
│   ├── introduction.md    # Introduction page
│   ├── getting-started/   # Getting started guides
│   ├── concepts/          # Core concepts
│   ├── cli/               # CLI reference
│   ├── advanced/          # Advanced topics
│   └── development/       # Development guides
└── book/                  # Generated output (git-ignored)
```

## Contributing

When adding new documentation:
1. Create/edit markdown files in `src/`
2. Update `src/SUMMARY.md` if adding new pages
3. Test with `mdbook serve`
4. Commit only the source files (not `book/`)

## Deployment

The documentation can be deployed to:
- GitHub Pages
- Netlify
- Vercel
- Any static site host

Simply build and deploy the `docs/book/` directory.
