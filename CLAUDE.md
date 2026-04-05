# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Seed-Intelligence** - A growing embodied intelligence system based on Hebbian learning (Ontology-driven incremental learning). The system verifies whether intelligence can grow from scratch through the cycle: "given starting point → web search → pattern induction → knowledge solidification".

## Project Structure

```
naive_learning/
├── seed                      # Rust CLI executable (recommended)
├── cli/                      # Rust CLI source code
├── server.js                 # Express server (Node.js)
├── incremental-learner.js    # Core learning engine
├── inference.js              # Q&A engine
├── crawler.js                # Web search
├── clipboard-watcher.js      # Passive learning
├── ebook-digester.js         # E-book reader
├── sandbox-environment.js    # Safe file operations
├── nano-lm.js                # Local language model
├── brain.json                # Knowledge graph data
└── public/                   # Web UI
```

## Supported File Formats

The CLI supports learning from multiple file formats:
- `.txt` - Plain text files (read directly)
- `.epub` - E-books (requires Calibre)
- `.mobi` - Kindle format (requires Calibre)
- `.azw3` - Kindle format (requires Calibre)
- `.pdf` - PDF documents (requires poppler-utils, text-based only)

## Common Commands

### CLI (Recommended - High Performance)

```bash
# Run REPL interactive mode
./seed

# Q&A
./seed ask "什么是人工智能"

# Learn from text
./seed learn-text "水是生命的源泉"

# Initialize concept (auto web search)
./seed init "人工智能"

# View stats
./seed stats
```

### Node.js (Web Server)

```bash
# Start the main server
node server.js

# Start clipboard watcher (passive learning)
node clipboard-watcher.js

# Read an ebook (active learning)
node ebook-digester.js ./agent_sandbox/book.txt
node ebook-digester.js ./agent_sandbox/book.epub   # requires Calibre

# Test sandbox environment
node sandbox-environment.js

# Test Nano-LM (local language model)
node nano-lm.js
```

## Architecture

### Core Modules

| Module | File | Purpose |
|--------|------|---------|
| **IncrementalLearner** | `incremental-learner.js` / `cli/src/learner.rs` | Hebbian learning engine with sliding window co-occurrence, energy-based concept solidification, decay/pruning |
| **Inference Engine** | `inference.js` / `cli/src/inference.rs` | Graph-based Q&A using path aggregation |
| **Crawler** | `crawler.js` / `cli/src/crawler.rs` | DuckDuckGo search + Wikipedia API + web scraping |
| **NLP** | `nlp.js` / `cli/src/nlp.rs` | Tokenization (Jieba for Chinese), word frequency, co-occurrence extraction |
| **LM** | `nano-lm.js` / `cli/src/lm.rs` | Local language model (Candle transformer) for text generation |

### Sensory Modules (Embodied)

| Module | File | Purpose |
|--------|------|---------|
| **Clipboard Watcher** | `clipboard-watcher.js` | Passive learning from clipboard content |
| **E-book Digester** | `ebook-digester.js` | Stream reading TXT/EPUB/MOBI/AZW3 with rate control |
| **Sandbox Environment** | `sandbox-environment.js` | Safe file operations in `./agent_sandbox/` |
| **Nano-Causal-LM** | `nano-lm.js` | Local language model for text generation |

### CLI Commands

| Command | Usage |
|---------|-------|
| `./seed` | Start REPL interactive mode |
| `./seed ask <question>` | Q&A |
| `./seed learn-text <text>` | Learn from text |
| `./seed learn-file <file>` | Learn from file (txt, epub, mobi, azw3, pdf) |
| `./seed init <concept>` | Initialize and learn from web |
| `./seed stats` | View statistics |
| `./seed brain` | View knowledge graph |
| `./seed concept <name>` | Concept details |
| `./seed related <name>` | Related concepts |
| `./seed clear` | Clear knowledge base |
| `./seed train <text>` | Train local language model |
| `./seed train-file <file>` | Train LM from file |
| `./seed generate <prompt>` | Generate text with local LM |

### Data Structure (brain.json v2.0)

```json
{
  "concepts": {
    "概念名": { "energy": 1.2, "count": 45, "firstSeen": "...", "lastSeen": "..." }
  },
  "relations": {
    "rel_xxx": { "source": "A", "target": "B", "weight": 0.85, "count": 10 }
  }
}
```

### API Endpoints (Node.js only)

- `POST /api/init` - Inject initial concept, trigger learning
- `POST /api/learn-text` - Learn from raw text (clipboard watcher)
- `POST /api/query` - Q&A via graph pathfinding
- `POST /api/ask` - Enhanced Q&A with Wikipedia integration
- `POST /api/clear` - Clear knowledge base
- `GET /api/brain` - Get knowledge graph
- `GET /api/stats` - Get learning statistics
- `POST /api/train-lm` - Train local language model
- `POST /api/generate` - Generate text with local LM
- `POST /api/generate-stream` - Stream generated text (SSE)

## Key Design Principles

1. **Memory < 20MB**: All stream processing uses `fs.createReadStream`, never loads full file into memory
2. **CPU Friendly**: Polling-based clipboard watcher (1.5s interval), e-book rate control (8 lines/sec)
3. **No LLM**: Q&A uses graph traversal (Dijkstra/BFS), not generative AI
4. **Security**: Sandbox environment blocks `../` path traversal attempts

## Building CLI

```bash
cd cli
cargo build --release
cp target/release/seed-intelligence ../seed
```

CLI data path: `~/.local/share/seed-intelligence/brain.json`

## Dependencies

### Node.js
- `express` - Web server
- `cors` - CORS middleware
- `axios` - HTTP client for crawler
- `cheerio` - HTML parsing
- `clipboardy` - Clipboard access
- Calibre (external) - For EPUB/MOBI/AZW3 conversion (`sudo apt install calibre`)

### Rust CLI
- `clap` - CLI argument parsing
- `reqwest` - HTTP client
- `tokio` - Async runtime
- `directories` - Platform-specific data directories
- `jieba-rs` - Chinese word segmentation
- `stop-words` - Stop words for multiple languages (Chinese, English, etc.)
- `candle-core` / `candle-nn` - Deep learning framework for LM