# Seed-Intelligence 🌱

> 基于本体论驱动的生长型具身智能实验原型

验证智能是否可以通过"起点 → 搜索 → 归纳 → 固化"的循环从零生长。

## 核心特性

- **增量学习**: 基于 Hebb 学习规则 ("一起激发的神经元连在一起")
- **非神经网络**: 使用滑动窗口共现算法，不依赖预训练模型
- **具身智能**: 剪贴板监听 + 电子书阅读 + 沙盒执行
- **图谱推理**: 迪杰斯特拉路径查找回答用户问题

## 快速开始

### Node.js 版本 (Web 服务)

```bash
# 安装依赖
npm install

# 启动服务器
node server.js
```

访问 http://localhost:3000

### CLI 版本 (推荐)

高性能 Rust CLI 应用，无需 Node.js 依赖。

```bash
# 运行 REPL 交互模式
./seed

# 或使用命令模式
./seed ask "什么是人工智能"
./seed learn-text "水是生命的源泉"
./seed stats
```

详细用法见 [CLI 使用指南](#cli-使用指南)。

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
│       ├── nlp.rs            # 分词/停用词
│       └── crawler.rs        # Wikipedia/DuckDuckGo 搜索
├── server.js                 # Express 服务器 (Node.js)
├── incremental-learner.js    # 核心增量学习引擎
├── inference.js              # 表达引擎 (QA Engine)
├── crawler.js                # DuckDuckGo 搜索
├── clipboard-watcher.js      # 剪贴板监听器
├── ebook-digester.js         # 电子书流式消化器
├── sandbox-environment.js    # 具身控制安全沙盒
├── public/                   # 前端界面 (D3.js 可视化)
└── agent_sandbox/            # 沙盒物理宇宙
```

## CLI 使用指南

### 安装编译

```bash
cd cli
cargo build --release
cp target/release/seed-intelligence ../seed
```

### 基本命令

```bash
# 问答
./seed ask "什么是人工智能"

# 从文本学习
./seed learn-text "水是生命的源泉"

# 初始化概念 (自动从网络搜索学习)
./seed init "人工智能"

# 查看统计
./seed stats

# 查看知识图谱
./seed brain

# 查看概念详情
./seed concept "智能"

# 查看相关概念
./seed related "智能"

# 清空知识库
./seed clear
```

### REPL 交互模式

```bash
./seed
```

支持以下命令：
- 直接输入问题 → 问答
- `/help` - 显示帮助
- `/stats` - 统计信息
- `/brain` - 知识图谱
- `/learn-text <文本>` - 从文本学习
- `/init <概念>` - 从网络学习
- `/concept <名称>` - 概念详情
- `/related <名称>` - 相关概念
- `/clear` - 清空知识库
- `/save` - 保存
- `/exit` - 退出

数据保存在 `~/.local/share/seed-intelligence/brain.json`

## 核心技术

### 增量学习 (IncrementalLearner)

- 滑动窗口关联 (窗口大小 6)
- 对数权重增长 (防止爆炸)
- 距离衰减 (窗口内越远权重越低)
- 概念凝固 (energy 值)
- 代谢遗忘 (0.95 衰减率)

### 表达引擎 (/ask)

- 查询解析: 提取核心实体
- 图谱寻路: 迪杰斯特拉算法
- 结果格式化: 逻辑链输出

### 安全沙盒

- 路径校验 (防止 `../` 逃逸)
- 扩展名白名单
- 文件大小限制 (10MB)

## 性能

- 处理 10KB 文本 < 100ms
- 内存占用 < 20MB
- CPU 友好 (1.5s 轮询, 8行/秒阅读速度)

## 技术栈

### CLI 版本 (推荐)
- Rust 2021
- clap (CLI)
- reqwest (HTTP)
- tokio (异步)

### Node.js 版本
- Node.js + Express
- Vanilla JS (前端)
- D3.js (可视化)
- Cheerio (爬虫)
- Clipboardy (剪贴板)
- Calibre (电子书转换, 可选)

## 许可证

MIT