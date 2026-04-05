# Seed-Intelligence 🌱

> 基于本体论驱动的生长型具身智能实验原型

验证智能是否可以通过"起点 → 搜索 → 归纳 → 固化"的循环从零生长。

## 核心特性

- **增量学习**: 基于 Hebb 学习规则 ("一起激发的神经元连在一起")
- **中文优化**: 使用 jieba 分词，准确识别中文词汇
- **本地 LM**: 内置轻量级 Transformer 语言模型 (Candle)
- **图谱推理**: 迪杰斯特拉路径查找回答用户问题
- **多格式支持**: 支持 txt, epub, mobi, azw3, pdf 格式文件
- **具身智能**: 剪贴板监听 + 电子书阅读 + 沙盒执行

## 快速开始

### 编译安装

```bash
cd cli
cargo build --release
cp target/release/seed-intelligence ../seed
```

### 交互模式 (REPL)

```bash
./seed
```

进入交互模式后直接输入问题进行问答。

### 命令行模式

```bash
./seed <command> [options]
```

---

## 命令详解

### 学习命令

#### `init` - 从网络初始化概念

从网络搜索学习一个概念，构建初始知识图谱。

```bash
./seed init <concept>

# 示例
./seed init "人工智能"
./seed init "量子计算"
```

**参数:**
- `concept`: 要学习的概念名称
- `--auto-learn`: 是否自动学习相关概念 (默认 true)

---

#### `learn` - 从网络搜索学习

与 `init` 类似，但不自动学习相关概念。

```bash
./seed learn <concept>

# 示例
./seed learn "机器学习"
```

---

#### `learn-text` - 从文本学习

直接从文本内容学习，提取概念和关系。

```bash
./seed learn-text <text> [--focus <concept>]

# 示例
./seed learn-text "人工智能是计算机科学的一个分支，研究如何使计算机模拟人类智能"
./seed learn-text "深度学习是机器学习的子领域" --focus "深度学习"
```

**参数:**
- `text`: 要学习的文本内容
- `--focus`: 聚焦概念，会给予更高权重

---

#### `learn-file` - 从文件学习 (推荐)

从文件中读取内容学习，支持多种格式。

```bash
./seed learn-file <file> [--focus <concept>] [--rate <lines_per_sec>]

# 示例 - TXT 文件
./seed learn-file ./books/ai_basics.txt

# 示例 - EPUB 电子书
./seed learn-file ./books/machine_learning.epub

# 示例 - MOBI 格式
./seed learn-file ./books/data_science.mobi

# 示例 - PDF 文档
./seed learn-file ./documents/research_paper.pdf

# 示例 - 带聚焦概念和速率控制
./seed learn-file ./books/ai_textbook.epub --focus "神经网络" --rate 50
```

**支持的格式:**
- `.txt` - 纯文本文件 (直接读取)
- `.epub` - 电子书格式 (需要 Calibre)
- `.mobi` - Kindle 格式 (需要 Calibre)
- `.azw3` - Kindle 格式 (需要 Calibre)
- `.pdf` - PDF 文档 (需要 poppler-utils，仅支持文本型 PDF)

**参数:**
- `file`: 文件路径
- `--focus`: 聚焦概念
- `--rate`: 每秒处理行数，默认 100

**依赖安装:**
```bash
# 安装 Calibre (用于 epub, mobi, azw3)
sudo apt install calibre

# 安装 poppler-utils (用于 PDF)
sudo apt install poppler-utils
```

---

### 问答命令

#### `ask` - 增强问答

基于知识图谱回答问题，提供关联信息。

```bash
./seed ask <question>

# 示例
./seed ask "什么是人工智能"
./seed ask "机器学习和深度学习有什么关系"
```

---

#### `query` - 基础问答

基于知识图谱的基础问答。

```bash
./seed query <question>

# 示例
./seed query "人工智能的定义"
```

---

### 查看命令

#### `stats` - 统计信息

查看知识图谱的统计信息。

```bash
./seed stats
```

**输出示例:**
```
📊 Knowledge Graph Stats
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
总概念数: 128
总关系数: 512
平均权重: 0.1567
平均能量: 0.6234

🔝 Top Concepts:
  人工智能 (energy: 2.45, count: 23)
  机器学习 (energy: 1.89, count: 18)
  ...
```

---

#### `brain` - 知识图谱

查看完整的知识图谱内容。

```bash
./seed brain
```

---

#### `concept` - 概念详情

查看单个概念的详细信息。

```bash
./seed concept <name>

# 示例
./seed concept "人工智能"
./seed concept "机器学习"
```

**输出示例:**
```
📌 Concept: 人工智能
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
能量: 2.4500
出现次数: 23
首次出现: 1743878400
最后出现: 1743964800
```

---

#### `related` - 相关概念

查看与某概念相关的其他概念。

```bash
./seed related <name> [--depth <depth>]

# 示例
./seed related "人工智能"
./seed related "机器学习" --depth 3
```

**参数:**
- `name`: 概念名称
- `--depth`: 遍历深度，默认 2

---

### 语言模型命令

#### `train` - 训练语言模型

从文本训练本地语言模型。

```bash
./seed train <text> [--epochs <n>]

# 示例
./seed train "人工智能是计算机科学的分支..." -e 5
```

**参数:**
- `text`: 训练文本
- `--epochs`: 训练轮数，默认 3

---

#### `train-file` - 从文件训练模型

从文件内容训练语言模型，支持多种格式。

```bash
./seed train-file <file> [--epochs <n>]

# 示例
./seed train-file ./books/ai_basics.txt -e 3
./seed train-file ./books/textbook.epub -e 5
./seed train-file ./papers/research.pdf -e 2
```

---

#### `generate` - 生成文本

使用训练好的语言模型生成文本。

```bash
./seed generate <prompt> [--max-tokens <n>]

# 示例
./seed generate "人工智能"
./seed generate "机器学习是" -m 100
```

**参数:**
- `prompt`: 提示文本
- `--max-tokens`: 最大生成 token 数，默认 50

---

### 管理命令

#### `clear` - 清空知识库

清空当前的知识图谱。

```bash
./seed clear
```

---

#### 通用选项

```bash
./seed -b <path> <command>   # 指定 brain.json 路径
./seed --brain <path> <command>

# 示例 - 使用自定义数据路径
./seed -b ./my_brain.json stats
./seed --brain /tmp/test.json learn-text "测试文本"
```

---

## REPL 交互模式

进入 REPL 模式后，支持以下命令：

| 命令 | 说明 |
|------|------|
| 直接输入问题 | 进行问答 |
| `/help` | 显示帮助 |
| `/stats` | 统计信息 |
| `/brain` | 知识图谱 |
| `/learn-text <文本>` | 从文本学习 |
| `/init <概念>` | 从网络学习 |
| `/learn <概念>` | 学习概念 |
| `/concept <名称>` | 概念详情 |
| `/related <名称>` | 相关概念 |
| `/clear` | 清空知识库 |
| `/save` | 保存知识库 |
| `/reload` | 重新加载 |
| `/exit` | 退出程序 |

---

## 项目结构

```
naive_learning/
├── seed                      # Rust CLI 可执行文件
├── cli/                      # Rust CLI 源代码
│   └── src/
│       ├── main.rs           # CLI 入口 + REPL
│       ├── brain.rs          # 知识图谱数据结构
│       ├── learner.rs        # Hebbian 学习引擎
│       ├── inference.rs      # 问答推理引擎
│       ├── nlp.rs            # 分词 (jieba-rs)
│       ├── lm.rs             # 本地语言模型 (Candle)
│       ├── file_reader.rs    # 文件读取 (多格式支持)
│       └── crawler.rs        # Wikipedia/DuckDuckGo
├── server.js                 # Express 服务器 (Node.js)
├── incremental-learner.js    # 核心增量学习引擎
├── inference.js              # 表达引擎 (QA Engine)
├── crawler.js                # DuckDuckGo 搜索
├── clipboard-watcher.js      # 剪贴板监听器
├── ebook-digester.js         # 电子书流式消化器
├── nano-lm.js                # 本地语言模型 (Node.js)
└── public/                   # 前端界面 (D3.js)
```

---

## 数据存储

- **知识图谱**: `~/.local/share/seed-intelligence/brain.json`
- **LM 权重**: `~/.local/share/seed-intelligence/lm_weights.json`

使用 `-b` 选项指定自定义路径。

---

## 技术栈

### Rust CLI
- `clap` - CLI 参数解析
- `reqwest` - HTTP 客户端
- `tokio` - 异步运行时
- `jieba-rs` - 中文分词
- `candle-core/candle-nn` - 深度学习框架

### 外部工具
- `Calibre` - 电子书转换 (`sudo apt install calibre`)
- `poppler-utils` - PDF 处理 (`sudo apt install poppler-utils`)

### Node.js (可选)
- Express, Cheerio, Clipboardy, D3.js

---

## 许可证

MIT
