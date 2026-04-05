# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Seed-Intelligence** - A growing embodied intelligence system based on Hebbian learning (Ontology-driven incremental learning). The system verifies whether intelligence can grow from scratch through the cycle: "given starting point → web search → pattern induction → knowledge solidification".

## Common Commands

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
| **IncrementalLearner** | `incremental-learner.js` | Hebbian learning engine with sliding window co-occurrence, energy-based concept solidification, decay/pruning |
| **Inference Engine** | `inference.js` | Graph-based Q&A using path aggregation |
| **Crawler** | `crawler.js` | DuckDuckGo search + web scraping |
| **NLP** | `nlp.js` | Tokenization, word frequency, co-occurrence extraction |

### Sensory Modules (Embodied)

| Module | File | Purpose |
|--------|------|---------|
| **Clipboard Watcher** | `clipboard-watcher.js` | Passive learning from clipboard content |
| **E-book Digester** | `ebook-digester.js` | Stream reading TXT/EPUB/MOBI/AZW3 with rate control |
| **Sandbox Environment** | `sandbox-environment.js` | Safe file operations in `./agent_sandbox/` |
| **Nano-Causal-LM** | `nano-lm.js` | Local language model for text generation |

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

### API Endpoints

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

## Dependencies

- `express` - Web server
- `cors` - CORS middleware
- `axios` - HTTP client for crawler
- `cheerio` - HTML parsing
- `clipboardy` - Clipboard access
- Calibre (external) - For EPUB/MOBI/AZW3 conversion (`sudo apt install calibre`)