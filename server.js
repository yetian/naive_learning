/**
 * Seed-Intelligence 主服务器
 * 包含学习引擎和所有API端点
 */

const express = require('express');
const cors = require('cors');
const fs = require('fs');
const path = require('path');

const crawler = require('./crawler');
const nlp = require('./nlp');
const inference = require('./inference');
const { IncrementalLearner } = require('./incremental-learner');

// 初始化增量学习引擎
const learner = new IncrementalLearner();

const app = express();
const PORT = process.env.PORT || 3000;

// 中间件
app.use(cors());
app.use(express.json());
app.use(express.static('public'));

// 知识图谱路径
const BRAIN_PATH = path.join(__dirname, 'brain.json');

// 学习状态
let learningStatus = {
  isLearning: false,
  currentConcept: null,
  learned: [],
  queue: []
};

// ============ 工具函数 =============

// 加载知识图谱 (使用增量学习器)
function loadBrain() {
  return learner.brain;
}

// 保存知识图谱
function saveBrain() {
  learner.saveBrain();
}

// ============ 核心学习引擎 (基于增量学习器) =============

/**
 * 学习一个概念 - 使用 IncrementalLearner
 * @param {string} concept - 要学习的概念
 * @param {boolean} autoLearn - 是否自动学习新概念
 * @returns {Promise<object>} 学习结果
 */
async function learn(concept, autoLearn = true) {
  console.log(`[Learn] Starting to learn: ${concept}`);

  try {
    // 1. 搜索获取相关网页
    const searchResults = await crawler.searchDuckDuckGo(concept);
    console.log(`[Learn] Got ${searchResults.length} search results`);

    // 2. 提取文本内容
    const texts = searchResults.map(r => `${r.title}: ${r.snippet}`);
    const fullText = texts.join(' ');

    // 3. 使用增量学习器学习 - 本体锚定模式
    const result = learner.learnFromText(fullText, concept);
    console.log(`[Learn] Incremental learning result:`, result);

    // 4. 执行清理（衰减+剪枝）
    const cleanupResult = learner.cleanup(false);
    console.log(`[Learn] Cleanup result:`, cleanupResult);

    // 5. 获取新发现的概念（能量最高的）
    const stats = learner.getStats();
    const newConcepts = stats.topConcepts
      .filter(c => c.name !== concept && c.energy > 0.5)
      .slice(0, 5)
      .map(c => c.name);

    // 保存
    learner.saveBrain();

    return {
      success: true,
      concept,
      keywords: stats.topConcepts.slice(0, 10),
      newConcepts,
      conceptsCount: stats.totalConcepts,
      relationsCount: stats.totalRelations,
      learningStats: result,
      cleanupStats: cleanupResult
    };

  } catch (error) {
    console.error('[Learn] Error:', error.message);
    return {
      success: false,
      concept,
      error: error.message
    };
  }
}

/**
 * 直接从文本学习（用于剪贴板监听器）
 * @param {string} text - 要学习的文本
 * @param {string|null} focusConcept - 焦点概念
 */
function learnFromText(text, focusConcept = null) {
  try {
    const result = learner.learnFromText(text, focusConcept);
    const cleanupResult = learner.cleanup(false);
    learner.saveBrain();
    return { success: true, ...result, cleanup: cleanupResult };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

// ============ API 端点 =============

// 初始化/注入本体概念
app.post('/api/init', async (req, res) => {
  try {
    const { concept, autoLearn = true } = req.body;

    if (!concept) {
      return res.status(400).json({ error: '请提供初始概念' });
    }

    learningStatus.isLearning = true;
    learningStatus.currentConcept = concept;
    learningStatus.queue = [concept];

    // 开始学习
    const result = await learn(concept, autoLearn);

    // 如果开启自动学习，依次学习新概念
    if (autoLearn && result.newConcepts && result.newConcepts.length > 0) {
      learningStatus.queue.push(...result.newConcepts);
      learningStatus.learned.push(concept);

      // 异步学习队列中的其他概念
      processQueue(autoLearn);
    }

    learningStatus.isLearning = false;
    learningStatus.currentConcept = null;

    res.json({
      success: true,
      ...result,
      status: learningStatus
    });

  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

// 触发学习（手动）
app.post('/api/learn/:concept', async (req, res) => {
  try {
    const { concept } = req.params;
    const result = await learn(concept, false);
    res.json(result);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

// 获取完整知识图谱
app.get('/api/brain', (req, res) => {
  const brain = loadBrain();
  res.json(brain);
});

// 获取学习状态
app.get('/api/status', (req, res) => {
  res.json(learningStatus);
});

// 问答接口 (旧版)
app.post('/api/query', (req, res) => {
  try {
    const { question } = req.body;

    if (!question) {
      return res.status(400).json({ error: '请提供问题' });
    }

    const result = inference.query(question);
    res.json(result);

  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

// 表达引擎 /ask 接口 (新版)
app.post('/api/ask', (req, res) => {
  try {
    const { question } = req.body;

    if (!question) {
      return res.status(400).json({ error: '请提供问题' });
    }

    const result = inference.ask(question);
    res.json(result);

  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

// 获取概念详情
app.get('/api/concept/:name', (req, res) => {
  const brain = loadBrain();
  const details = inference.getConceptDetails(req.params.name, brain);
  res.json(details || { error: '概念不存在' });
});

// 直接从文本学习（用于剪贴板监听器）
app.post('/api/learn-text', (req, res) => {
  try {
    const { text, focusConcept } = req.body;

    if (!text || text.trim().length < 2) {
      return res.status(400).json({ error: '文本内容太短' });
    }

    const result = learnFromText(text, focusConcept || null);
    res.json(result);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

// 获取学习统计
app.get('/api/stats', (req, res) => {
  const stats = learner.getStats();
  res.json(stats);
});

// 异步处理学习队列
async function processQueue(autoLearn = true) {
  while (learningStatus.queue.length > 0 && learningStatus.isLearning) {
    const nextConcept = learningStatus.queue.shift();
    learningStatus.currentConcept = nextConcept;

    console.log(`[Queue] Learning: ${nextConcept}`);

    const result = await learn(nextConcept, false);

    if (result.success) {
      learningStatus.learned.push(nextConcept);

      // 添加新发现的关联概念到队列
      if (autoLearn && result.newConcepts) {
        for (const nc of result.newConcepts) {
          if (!learningStatus.queue.includes(nc) && !learningStatus.learned.includes(nc)) {
            learningStatus.queue.push(nc);
          }
        }
      }
    }

    // 避免请求过快
    await new Promise(resolve => setTimeout(resolve, 1000));
  }

  learningStatus.isLearning = false;
  learningStatus.currentConcept = null;
}

// 启动服务器
app.listen(PORT, () => {
  console.log(`
╔═══════════════════════════════════════════╗
║      🌱 Seed-Intelligence 启动完成        ║
╠═══════════════════════════════════════════╣
║  服务器: http://localhost:${PORT}           ║
║  前端界面: http://localhost:${PORT}          ║
╚═══════════════════════════════════════════╝
  `);
});

module.exports = { app, learn };