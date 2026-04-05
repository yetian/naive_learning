# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Seed-Intelligence** - A growing embodied intelligence system based on Hebbian learning (Ontology-driven incremental learning). The system verifies whether intelligence can grow from scratch through the cycle: "given starting point → web search → pattern induction → knowledge solidification".

## Project Structure

```
naive_learning/
├── seed              # Rust CLI executable
├── cli/              # Rust CLI source code
│   └── src/
│       ├── main.rs           # CLI entry point + REPL
│       ├── brain.rs          # Knowledge graph data structure
│       ├── learner.rs        # Hebbian learning engine
│       ├── inference.rs      # Q&A inference engine
│       ├── nlp.rs            # Tokenization (jieba-rs)
│       ├── lm.rs             # Local language model (Candle)
│       ├── file_reader.rs    # Multi-format file reader
│       └── crawler.rs        # Multi-language Wikipedia search
└── agent_sandbox/    # Safe file operations directory
```

## Supported File Formats

The CLI supports learning from multiple file formats:
- `.txt` - Plain text files (read directly)
- `.epub` - E-books (requires Calibre)
- `.mobi` - Kindle format (requires Calibre)
- `.azw3` - Kindle format (requires Calibre)
- `.pdf` - PDF documents (requires poppler-utils, text-based only)

## Common Commands

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

## Architecture

### Core Modules

| Module | File | Purpose |
|--------|------|---------|
| **IncrementalLearner** | `cli/src/learner.rs` | Hebbian learning engine with sliding window co-occurrence, energy-based concept solidification, decay/pruning |
| **Inference Engine** | `cli/src/inference.rs` | Graph-based Q&A using path aggregation, multi-word phrase matching |
| **Crawler** | `cli/src/crawler.rs` | Multi-language Wikipedia search (11 languages) + DuckDuckGo API |
| **NLP** | `cli/src/nlp.rs` | Tokenization (Jieba for Chinese), word frequency, co-occurrence extraction |
| **LM** | `cli/src/lm.rs` | Local language model (Candle transformer) for text generation |
| **FileReader** | `cli/src/file_reader.rs` | Multi-format file reading with streaming support |

### CLI Commands

| Command | Usage |
|---------|-------|
| `./seed` | Start REPL interactive mode |
| `./seed ask <question>` | Q&A with concept descriptions |
| `./seed learn-text <text>` | Learn from text |
| `./seed learn-file <file>` | Learn from file (txt, epub, mobi, azw3, pdf) |
| `./seed init <concept>` | Initialize and learn from web |
| `./seed observe` | Embodied intelligence mode (watch files, clipboard, commands) |
| `./seed stats` | View statistics |
| `./seed brain` | View knowledge graph |
| `./seed concept <name>` | Concept details with description |
| `./seed related <name>` | Related concepts |
| `./seed clear` | Clear knowledge base |
| `./seed train <text>` | Train local language model |
| `./seed train-file <file>` | Train LM from file |
| `./seed generate <prompt>` | Generate text with local LM |

### Data Structure (brain.json v2.1)

```json
{
  "concepts": {
    "概念名": {
      "energy": 1.2,
      "count": 45,
      "firstSeen": "...",
      "lastSeen": "...",
      "description": "概念的定义说明（来自Wikipedia）"
    }
  },
  "relations": {
    "rel_xxx": { "source": "A", "target": "B", "weight": 0.85, "count": 10 }
  }
}
```

### Multi-language Support

The crawler automatically detects query language and searches the appropriate Wikipedia edition:

| Language | Code | Example Query |
|----------|------|---------------|
| Chinese | zh | 人工智能 |
| Japanese | ja | 人工知能 |
| Korean | ko | 인공지능 |
| Russian | ru | Искусственный интеллект |
| Arabic | ar | الذكاء الاصطناعي |
| Thai | th | ปัญญาประดิษฐ์ |
| Vietnamese | vi | Trí tuệ nhân tạo |
| German | de | Künstliche Intelligenz |
| French | fr | Intelligence artificielle |
| Spanish | es | Inteligencia artificial |
| English | en | Artificial Intelligence |

Language detection uses Unicode character ranges and language-specific characters.

### Observe Mode (Embodied Intelligence)

The `observe` command enables embodied intelligence mode where Seed learns from its environment:

```bash
./seed observe
./seed observe --sandbox ./my_sandbox
```

**Features:**
- **File watching**: Monitors `agent_sandbox/` for new/modified files
- **Clipboard monitoring**: Learns from clipboard content (polling-based)
- **Command execution**: Run commands and learn from output
- **Batch learning**: Accumulates observations and learns in batches

**Interactive Commands in Observe Mode:**
| Command | Description |
|---------|-------------|
| `<text>` | Add text to observation buffer |
| `/run <cmd>` | Execute command and capture output |
| `/file <path>` | Read file from sandbox |
| `/history` | Show recent observations |
| `/stats` | Show observation statistics |
| `/learn` | Force immediate batch learning |
| `/save` | Save knowledge base |
| `/exit` | Exit observe mode |

## Key Design Principles

1. **Memory Efficient**: Stream processing for large files, never loads full file into memory
2. **CPU Friendly**: Rate-controlled file reading, no heavy computation
3. **No LLM for Q&A**: Q&A uses graph traversal (Dijkstra/BFS), not generative AI
4. **Security**: Sandbox environment blocks `../` path traversal attempts

## Building CLI

```bash
cd cli
cargo build --release
cp target/release/seed-intelligence ../seed
```

CLI data path: `~/.local/share/seed-intelligence/brain.json`

## Dependencies

### Rust
- `clap` - CLI argument parsing
- `reqwest` - HTTP client
- `tokio` - Async runtime
- `directories` - Platform-specific data directories
- `jieba-rs` - Chinese word segmentation
- `stop-words` - Stop words for multiple languages (Chinese, English, etc.)
- `candle-core` / `candle-nn` - Deep learning framework for LM

### External Tools (optional)
- Calibre - For EPUB/MOBI/AZW3 conversion (`sudo apt install calibre`)
- poppler-utils - For PDF processing (`sudo apt install poppler-utils`)
