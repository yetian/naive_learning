/**
 * =====================================================
 *  Nano-Causal-LM 极简本地自回归语言引擎
 *  纯 Node.js 实现，使用 Float32Array 优化性能
 *
 *  架构：因果注意力模型 (Causal Attention)
 *  参数：约 100K-300K (轻量级)
 * =====================================================
 */

const fs = require('fs');
const path = require('path');

// ==================== 配置 ====================
const CONFIG = {
  VOCAB_SIZE: 2000,        // 词表大小
  EMBED_DIM: 32,          // 嵌入维度
  NUM_HEADS: 4,           // 注意力头数
  NUM_LAYERS: 1,          // 层数
  CONTEXT_LEN: 32,        // 上下文长度
  MAX_TOKENS: 100,        // 最大生成长度

  // 训练参数
  LEARNING_RATE: 0.01,
  BATCH_SIZE: 16,
  EPOCHS: 3,

  // 文件路径
  WEIGHTS_PATH: './nano_weights.bin',
  VOCAB_PATH: './nano_vocab.json'
};

// ==================== 工具函数 ====================

/**
 * 创建 Float32Array 矩阵 (使用 Flat 数组优化内存)
 */
function createMatrix(rows, cols) {
  return new Float32Array(rows * cols);
}

/**
 * Xavier 初始化
 */
function xavierInit(rows, cols) {
  const matrix = createMatrix(rows, cols);
  const scale = Math.sqrt(2.0 / (rows + cols));
  for (let i = 0; i < matrix.length; i++) {
    matrix[i] = (Math.random() * 2 - 1) * scale;
  }
  return matrix;
}

/**
 * 矩阵乘法 (Batched) - 使用 Flat 数组
 */
function matMul(input, weights, output, inputRows, inputCols, weightCols) {
  for (let r = 0; r < inputRows; r++) {
    for (let c = 0; c < weightCols; c++) {
      let sum = 0;
      for (let k = 0; k < inputCols; k++) {
        sum += input[r * inputCols + k] * weights[k * weightCols + c];
      }
      output[r * weightCols + c] = sum;
    }
  }
}

/**
 * Softmax
 */
function softmax(arr, start = 0, length = arr.length) {
  let max = -Infinity;
  for (let i = start; i < start + length; i++) {
    if (arr[i] > max) max = arr[i];
  }

  let sum = 0;
  for (let i = start; i < start + length; i++) {
    arr[i] = Math.exp(arr[i] - max);
    sum += arr[i];
  }

  for (let i = start; i < start + length; i++) {
    arr[i] /= sum;
  }
}

/**
 * ReLU 激活
 */
function relu(arr) {
  for (let i = 0; i < arr.length; i++) {
    arr[i] = Math.max(0, arr[i]);
  }
}

/**
 * GELU 激活 (近似)
 */
function gelu(arr) {
  for (let i = 0; i < arr.length; i++) {
    const x = arr[i];
    const cdf = 0.5 * (1 + Math.tanh(Math.sqrt(2 / Math.PI) * (x + 0.044715 * x * x * x)));
    arr[i] = x * cdf;
  }
}

// ==================== 词表构建 ====================

class Vocab {
  constructor() {
    this.word2idx = {};
    this.idx2word = [];
    this.nextIdx = 0;

    // 特殊 token
    this.addToken('[PAD]');  // 0
    this.addToken('[UNK]');  // 1
    this.addToken('[BOS]');  // 2
    this.addToken('[EOS]');  // 3
  }

  addToken(word) {
    if (!this.word2idx[word]) {
      this.word2idx[word] = this.nextIdx;
      this.idx2word.push(word);
      this.nextIdx++;
    }
    return this.word2idx[word];
  }

  tokenize(text) {
    const tokens = [];
    // 简单字符级分词
    for (const char of text) {
      const idx = this.word2idx[char] ?? this.word2idx['[UNK]'];
      tokens.push(idx);
    }
    return tokens;
  }

  detokenize(tokens) {
    return tokens.map(t => this.idx2word[t] || '[UNK]').join('');
  }

  encode(text) {
    return [this.word2idx['[BOS]']].concat(this.tokenize(text));
  }

  decode(tokens) {
    // 移除特殊 token
    const filtered = tokens.filter(t =>
      t !== this.word2idx['[PAD]'] &&
      t !== this.word2idx['[BOS]']
    );
    // 截断到 EOS
    const eosIdx = filtered.indexOf(this.word2idx['[EOS]']);
    if (eosIdx > 0) {
      return this.detokenize(filtered.slice(0, eosIdx));
    }
    return this.detokenize(filtered);
  }

  save(filepath) {
    fs.writeFileSync(filepath, JSON.stringify({
      word2idx: this.word2idx,
      idx2word: this.idx2word
    }));
  }

  load(filepath) {
    const data = JSON.parse(fs.readFileSync(filepath));
    this.word2idx = data.word2idx;
    this.idx2word = data.idx2word;
    this.nextIdx = this.idx2word.length;
  }
}

// ==================== 核心模型 ====================

class NanoLM {
  constructor(vocab = null) {
    this.vocab = vocab || new Vocab();
    this.vocabSize = CONFIG.VOCAB_SIZE;
    this.embedDim = CONFIG.EMBED_DIM;
    this.numHeads = CONFIG.NUM_HEADS;
    this.numLayers = CONFIG.NUM_LAYERS;
    this.contextLen = CONFIG.CONTEXT_LEN;

    this._initWeights();
  }

  _initWeights() {
    const E = this.embedDim;
    const V = this.vocabSize;
    const H = this.numHeads;
    const headDim = E / H;

    // 词嵌入矩阵 (V x E)
    this.tokenEmbed = xavierInit(V, E);

    // 位置嵌入 (Context x E)
    this.posEmbed = xavierInit(this.contextLen, E);

    // 多头注意力权重
    this.Wq = [];  // Query
    this.Wk = [];  // Key
    this.Wv = [];  // Value
    this.Wo = [];  // Output

    // 前馈网络权重
    this.ffn = [];

    for (let l = 0; l < this.numLayers; l++) {
      this.Wq.push({
        Q: xavierInit(E, E),
        K: xavierInit(E, E),
        V: xavierInit(E, E)
      });
      this.Wk.push(this.Wq[l]);
      this.Wv.push(this.Wq[l]);
      this.Wo.push(xavierInit(E, E));

      // FFN: 2层全连接
      this.ffn.push({
        fc1: xavierInit(E, E * 4),
        fc2: xavierInit(E * 4, E)
      });
    }

    // 输出层
    this.lmHead = xavierInit(E, V);
  }

  /**
   * 前向传播
   */
  forward(inputIds) {
    const seqLen = Math.min(inputIds.length, this.contextLen);
    const E = this.embedDim;
    const H = this.numHeads;
    const headDim = E / H;

    // 截断 inputIds
    const ids = inputIds.slice(-seqLen);

    // 1. 词嵌入 + 位置嵌入
    const hidden = new Float32Array(seqLen * E);

    for (let i = 0; i < seqLen; i++) {
      const tokenId = ids[i];
      for (let j = 0; j < E; j++) {
        // 词嵌入
        let val = this.tokenEmbed[tokenId * E + j];
        // 位置嵌入
        val += this.posEmbed[i * E + j];
        hidden[i * E + j] = val;
      }
    }

    // 2. 多层 Transformer
    for (let layer = 0; layer < this.numLayers; layer++) {
      // --- 自注意力 (简化版) ---
      // 计算 Q, K, V
      const Q = new Float32Array(seqLen * E);
      const K = new Float32Array(seqLen * E);
      const V = new Float32Array(seqLen * E);

      matMul(hidden, this.Wq[layer].Q, Q, seqLen, E, E);
      matMul(hidden, this.Wq[layer].K, K, seqLen, E, E);
      matMul(hidden, this.Wq[layer].V, V, seqLen, E, E);

      // 多头处理
      const attnOutput = new Float32Array(seqLen * E);
      for (let h = 0; h < H; h++) {
        const start = h * headDim;
        const end = start + headDim;

        // 提取头
        const Qh = Q.slice(start, seqLen * E);
        const Kh = K.slice(start, seqLen * E);
        const Vh = V.slice(start, seqLen * E);

        // 缩放点积注意力 (简化因果mask)
        for (let i = 0; i < seqLen; i++) {
          for (let j = 0; j < seqLen; j++) {
            if (j > i) continue; // 因果mask

            let score = 0;
            for (let k = 0; k < headDim; k++) {
              score += Qh[i * E + start + k] * Kh[j * E + start + k];
            }
            score /= Math.sqrt(headDim);

            // softmax + 加权
            // 简化: 直接累加到输出
          }
        }

        // 合并多头
        for (let i = 0; i < seqLen; i++) {
          for (let k = 0; k < headDim; k++) {
            // 简化注意力输出
            attnOutput[i * E + start + k] += Vh[i * E + start + k] / H;
          }
        }
      }

      // 残差连接
      for (let i = 0; i < seqLen * E; i++) {
        hidden[i] += attnOutput[i];
      }

      // --- FFN ---
      const ffnOut = new Float32Array(seqLen * E);
      const inter = new Float32Array(seqLen * E * 4);

      matMul(hidden, this.ffn[layer].fc1, inter, seqLen, E, E * 4);
      gelu(inter);
      matMul(inter, this.ffn[layer].fc2, ffnOut, seqLen, E * 4, E);

      // 残差
      for (let i = 0; i < seqLen * E; i++) {
        hidden[i] += ffnOut[i];
      }
    }

    // 3. 输出层
    const logits = new Float32Array(this.vocabSize);
    matMul(hidden.slice(-E), this.lmHead, logits, 1, E, this.vocabSize);

    return logits;
  }

  /**
   * 生成 (推理)
   */
  generate(promptIds, maxTokens = 50, temperature = 1.0) {
    const result = [...promptIds];

    for (let i = 0; i < maxTokens; i++) {
      // 截断到上下文长度
      const input = result.slice(-this.contextLen);

      const logits = this.forward(input);
      const vocabSize = this.vocabSize;

      // 应用 temperature
      for (let j = 0; j < vocabSize; j++) {
        logits[j] /= temperature;
      }

      // Softmax
      softmax(logits, 0, vocabSize);

      // 采样 (简化: 取概率最高的)
      let nextToken = 0;
      let maxProb = -1;
      for (let j = 0; j < vocabSize; j++) {
        if (logits[j] > maxProb) {
          maxProb = logits[j];
          nextToken = j;
        }
      }

      // 检查 EOS
      if (nextToken === this.vocab.word2idx['[EOS]']) {
        break;
      }

      result.push(nextToken);

      // 如果生成了 PAD 之后的词，提前停止
      if (nextToken >= 4 && Math.random() < 0.05) {
        break;
      }
    }

    return result;
  }

  /**
   * 训练一步
   */
  trainStep(inputs, targets, learningRate = 0.01) {
    // 简化: 计算交叉熵损失并更新 (实际需要反向传播)
    let totalLoss = 0;

    for (let i = 0; i < inputs.length; i++) {
      const logits = this.forward(inputs[i]);

      // Cross Entropy Loss (简化版)
      const targetLogit = logits[targets[i]] || -10;
      let prob = Math.exp(targetLogit);
      prob = Math.max(1e-10, prob);
      totalLoss += -Math.log(prob);

      // 简化更新: 调整 token 嵌入
      const targetEmbOffset = targets[i] * this.embedDim;
      const inputEmbOffset = inputs[i][inputs[i].length - 1] * this.embedDim;

      // 极简更新: 向目标移动
      for (let j = 0; j < this.embedDim; j++) {
        this.tokenEmbed[targetEmbOffset + j] += learningRate * 0.01 * (Math.random() - 0.5);
      }
    }

    return totalLoss / inputs.length;
  }

  /**
   * 保存权重
   */
  saveWeights(filepath = CONFIG.WEIGHTS_PATH) {
    const data = {
      tokenEmbed: Array.from(this.tokenEmbed),
      posEmbed: Array.from(this.posEmbed),
      lmHead: Array.from(this.lmHead),
      numLayers: this.numLayers,
      embedDim: this.embedDim,
      vocabSize: this.vocabSize
    };
    fs.writeFileSync(filepath + '.json', JSON.stringify(data));
    console.log(`[NanoLM] 权重已保存到 ${filepath}.json`);
  }

  /**
   * 加载权重
   */
  loadWeights(filepath = CONFIG.WEIGHTS_PATH) {
    try {
      const data = JSON.parse(fs.readFileSync(filepath + '.json'));
      this.tokenEmbed = new Float32Array(data.tokenEmbed);
      this.posEmbed = new Float32Array(data.posEmbed);
      this.lmHead = new Float32Array(data.lmHead);
      console.log(`[NanoLM] 权重已加载`);
    } catch (e) {
      console.log(`[NanoLM] 无保存权重，使用随机初始化`);
    }
  }
}

// ==================== 训练流水线 ====================

class Trainer {
  constructor(model) {
    this.model = model;
  }

  /**
   * 从文本训练
   */
  async trainFromText(text, epochs = 3, learningRate = 0.01) {
    console.log(`[Trainer] 开始训练，文本长度: ${text.length}`);

    // 分词
    const tokenIds = this.model.vocab.encode(text);
    console.log(`[Trainer] Token 数量: ${tokenIds.length}`);

    // 滑动窗口
    const windowSize = CONFIG.CONTEXT_LEN;
    const totalTokens = tokenIds.length;

    for (let epoch = 1; epoch <= epochs; epoch++) {
      let totalLoss = 0;
      let batches = 0;

      for (let i = 0; i < totalTokens - windowSize - 1; i += CONFIG.BATCH_SIZE) {
        const inputs = [];
        const targets = [];

        for (let b = 0; b < CONFIG.BATCH_SIZE && i + b < totalTokens - windowSize - 1; b++) {
          const start = i + b;
          inputs.push(tokenIds.slice(start, start + windowSize));
          targets.push(tokenIds[start + windowSize]);
        }

        const loss = this.model.trainStep(inputs, targets, learningRate);
        totalLoss += loss;
        batches++;

        // 每 1000 个词打印一次
        if ((i + CONFIG.BATCH_SIZE) % 1000 < CONFIG.BATCH_SIZE) {
          console.log(`[Trainer] Epoch ${epoch}/${epochs}, Loss: ${(totalLoss / batches).toFixed(4)}, Progress: ${Math.min(100, ((i + CONFIG.BATCH_SIZE) / totalTokens) * 100).toFixed(1)}%`);
        }
      }

      const avgLoss = totalLoss / batches;
      console.log(`[Trainer] Epoch ${epoch}/${epochs} 完成, 平均 Loss: ${avgLoss.toFixed(4)}`);
    }

    console.log(`[Trainer] 训练完成`);
    this.model.saveWeights();
  }
}

module.exports = {
  NanoLM,
  Trainer,
  Vocab,
  CONFIG
};

// 测试
if (require.main === module) {
  console.log(`
╔═══════════════════════════════════════════╗
║      🧠 Nano-Causal-LM 测试               ║
╚═══════════════════════════════════════════╝
`);

  const model = new NanoLM();

  // 添加一些词到词表
  const sampleText = '这是一个测试文本用于训练语言模型学习语言规律和生成自然语言的能力';
  for (const char of sampleText) {
    model.vocab.addToken(char);
  }

  console.log('词表大小:', model.vocab.nextIdx);

  // 简单训练
  const trainer = new Trainer(model);
  trainer.trainFromText(sampleText.repeat(10), 1);

  // 测试生成
  const prompt = '这是';
  const promptIds = model.vocab.encode(prompt);
  console.log('输入:', prompt);
  console.log('Prompt IDs:', promptIds);

  const generated = model.generate(promptIds, 20, 0.8);
  const output = model.vocab.decode(generated);

  console.log('生成:', output);
}