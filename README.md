# Seed-Intelligence 🌱

> 基于本体论驱动的生长型具身智能实验原型

## 为什么做这个项目？

当前的大语言模型（LLM）存在一个根本性问题：**效率极低**。

人类大脑功耗仅约 20 瓦，却能处理极其复杂的逻辑和感知。而运行一个同等智能水平的 LLM 可能需要几千瓦甚至更多的电力。这种量级上的差异说明，目前的神经网络在信息编码和检索效率上，离真正的"智能"还差得很远。

LLM 本质上是"概率预测引擎"——无论你问它"1+1等于几"还是"如何造火箭"，模型内部的几十亿个参数都会被全部调用一遍。这不是智能，这是"暴力美学"。

**我相信智能可以在普通电脑上实现。** 不需要 GPU，不需要海量参数，不需要云端算力。

这个项目的目标是探索一条不同的路：
- **Hebbian 学习**：像生物大脑一样，"一起激发的神经元连在一起"
- **稀疏激活**：只激活相关的概念，而不是全部参数
- **增量生长**：从零开始，通过学习逐渐构建知识图谱
- **轻量推理**：用图遍历而非矩阵乘法来回答问题
- **神经符号融合**：神经网络负责感知，符号逻辑负责推理，各司其职
- **联想记忆**：基于模式匹配而非概率预测，模拟大脑的记忆检索机制

验证智能是否可以通过"起点 → 搜索 → 归纳 → 固化"的循环从零生长。

## 核心特性

- **增量学习**: 基于 Hebb 学习规则 ("一起激发的神经元连在一起")
- **多语言支持**: 自动检测11种语言，搜索对应语言的Wikipedia
- **概念描述**: 从Wikipedia获取概念定义，提供更完整的回答
- **具身智能**: 观察模式可监听文件、剪贴板、命令输出，主动从环境学习
- **本地 LM**: 内置轻量级 Transformer 语言模型 (Candle)
- **图谱推理**: 迪杰斯特拉路径查找回答用户问题
- **多格式支持**: 支持 txt, epub, mobi, azw3, pdf 格式文件
- **轻量高效**: 无需 GPU，无需云端，普通电脑即可运行

---

## 多语言支持

系统自动检测查询语言，并搜索对应语言的 Wikipedia：

| 语言 | 代码 | 示例查询 |
|------|------|----------|
| 中文 | zh | 人工智能 |
| 日文 | ja | 人工知能 |
| 韩文 | ko | 인공지능 |
| 俄文 | ru | Искусственный интеллект |
| 阿拉伯文 | ar | الذكاء الاصطناعي |
| 泰文 | th | ปัญญาประดิษฐ์ |
| 越南文 | vi | Trí tuệ nhân tạo |
| 德文 | de | Künstliche Intelligenz |
| 法文 | fr | Intelligence artificielle |
| 西班牙文 | es | Inteligencia artificial |
| 英文 | en | Artificial Intelligence |

```bash
# 中文查询
./seed init "人工智能"
./seed ask "什么是人工智能"

# 英文查询
./seed init "Machine Learning"
./seed ask "What is Machine Learning"

# 日文查询
./seed init "人工知能"
```

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

从网络搜索学习一个概念，构建初始知识图谱。自动从 Wikipedia 获取概念描述。

```bash
./seed init <concept>

# 示例
./seed init "人工智能"
./seed init "量子计算"
./seed init "Machine Learning"   # 支持多语言
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
./seed learn-file <file> [--focus <concept>] [--batch <lines>]

# 示例 - TXT 文件
./seed learn-file ./books/ai_basics.txt

# 示例 - EPUB 电子书
./seed learn-file ./books/machine_learning.epub

# 示例 - MOBI 格式
./seed learn-file ./books/data_science.mobi

# 示例 - PDF 文档
./seed learn-file ./documents/research_paper.pdf

# 示例 - 带聚焦概念和批处理大小
./seed learn-file ./books/ai_textbook.epub --focus "神经网络" --batch 1000
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
- `--batch`: 每批处理行数，默认 500。值越大速度越快，但内存占用越高

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

基于知识图谱回答问题，显示概念描述和相关概念。

```bash
./seed ask <question>

# 示例
./seed ask "什么是人工智能"
./seed ask "机器学习和深度学习有什么关系"
./seed ask "What is Machine Learning"
```

**输出示例:**
```
🤖 **人工智能**

人工智能，是指计算机系统执行通常与人类智慧相关的任务的能力，例如学习、推理、解决问题、感知和决策。
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
📝 人工智能，是指计算机系统执行通常与人类智慧相关的任务的能力，例如学习、推理、解决问题、感知和决策。
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

### 具身智能命令

#### `observe` - 观察模式

进入观察模式，Seed 会主动观察环境并从中学习。这是具身智能的核心功能。

```bash
./seed observe
./seed observe --sandbox ./my_sandbox
```

**功能特性:**
- **文件监听**: 自动监听 `agent_sandbox/` 目录的文件变化
- **剪贴板监听**: 监听剪贴板内容，自动学习复制的内容
- **命令执行**: 执行命令并捕获输出进行学习
- **批量学习**: 积累一定量内容后批量学习

**交互命令:**

| 命令 | 说明 |
|------|------|
| `<文本>` | 直接输入文本添加到观察缓冲区 |
| `/run <命令>` | 执行命令并学习输出 |
| `/file <路径>` | 读取 sandbox 中的文件 |
| `/history` | 显示最近的观察记录 |
| `/stats` | 显示观察统计信息 |
| `/learn` | 立即触发批量学习 |
| `/save` | 保存知识库 |
| `/exit` | 退出观察模式 |

**使用示例:**
```bash
# 启动观察模式
./seed observe

# 在观察模式中：
🌱 > /run ls -la          # 执行命令并学习输出
🌱 > /file notes.txt      # 学习 sandbox 中的文件
🌱 > 这是一些笔记内容      # 直接输入文本学习
🌱 > /history             # 查看观察历史
🌱 > /exit                # 退出
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
├── seed              # Rust CLI 可执行文件
├── cli/              # Rust CLI 源代码
│   └── src/
│       ├── main.rs           # CLI 入口 + REPL
│       ├── brain.rs          # 知识图谱数据结构
│       ├── learner.rs        # Hebbian 学习引擎
│       ├── inference.rs      # 问答推理引擎
│       ├── nlp.rs            # 分词 (jieba-rs)
│       ├── lm.rs             # 本地语言模型 (Candle)
│       ├── file_reader.rs    # 文件读取 (多格式支持)
│       └── crawler.rs        # 多语言Wikipedia搜索 (11种语言)
└── agent_sandbox/    # 沙盒目录 (文件操作)
```

---

## 数据存储

- **知识图谱**: `~/.local/share/seed-intelligence/brain.db` (SQLite 数据库)
- **LM 权重**: `~/.local/share/seed-intelligence/lm_weights.json`

使用 `-b` 选项指定自定义路径。

**性能优化**: 学习引擎使用内存批量累积 + 单事务 UPSERT 写入，处理速度约 ~15-30ms/批，比之前快约 500 倍。

---

## 技术栈

### Rust
- `clap` - CLI 参数解析
- `reqwest` - HTTP 客户端
- `tokio` - 异步运行时
- `jieba-rs` - 中文分词
- `stop-words` - 多语言停用词 (中文、英文等 60+ 语言)
- `candle-core/candle-nn` - 深度学习框架

### 外部工具 (可选)
- `Calibre` - 电子书转换 (`sudo apt install calibre`)
- `poppler-utils` - PDF 处理 (`sudo apt install poppler-utils`)

---

## 许可证

MIT
