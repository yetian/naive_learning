# Seed-Intelligence 🌱

> 基于本体论驱动的生长型具身智能实验原型

验证智能是否可以通过"起点 → 搜索 → 归纳 → 固化"的循环从零生长。

## 核心特性

- **增量学习**: 基于 Hebb 学习规则 ("一起激发的神经元连在一起")
- **非神经网络**: 使用滑动窗口共现算法，不依赖预训练模型
- **具身智能**: 剪贴板监听 + 电子书阅读 + 沙盒执行
- **图谱推理**: 迪杰斯特拉路径查找回答用户问题

## 快速开始

### 安装依赖

```bash
npm install
```

### 启动服务器

```bash
node server.js
```

访问 http://localhost:3000

### 启动剪贴板监听器 (被动学习)

```bash
node clipboard-watcher.js
```

### 阅读电子书 (主动学习)

```bash
# TXT 文件
node ebook-digester.js ./agent_sandbox/book.txt

# EPUB/MOBI/AZW3 (需要安装 Calibre)
# sudo apt install calibre
node ebook-digester.js ./agent_sandbox/book.epub
```

## 架构

```
naive_learning/
├── server.js               # Express 服务器
├── incremental-learner.js  # 核心增量学习引擎
├── inference.js            # 表达引擎 (QA Engine)
├── crawler.js              # DuckDuckGo 搜索
├── clipboard-watcher.js    # 剪贴板监听器
├── ebook-digester.js       # 电子书流式消化器
├── sandbox-environment.js  # 具身控制安全沙盒
├── public/                 # 前端界面 (D3.js 可视化)
└── agent_sandbox/          # 沙盒物理宇宙
```

## API 接口

| 端点 | 方法 | 功能 |
|------|------|------|
| `/api/init` | POST | 注入初始概念，触发学习 |
| `/api/learn-text` | POST | 从文本学习 (剪贴板) |
| `/api/ask` | POST | 问答接口 (图谱路径) |
| `/api/brain` | GET | 获取知识图谱 |
| `/api/stats` | GET | 获取学习统计 |

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

- Node.js + Express
- Vanilla JS (前端)
- D3.js (可视化)
- Cheerio (爬虫)
- Clipboardy (剪贴板)
- Calibre (电子书转换, 可选)

## 许可证

MIT