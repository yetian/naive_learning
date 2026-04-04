/**
 * =====================================================
 *  IncrementalLearner - 增量学习引擎
 *  基于 Hebb 学习规则: "一起激发的神经元连在一起"
 *
 *  核心特性:
 *  1. 滑动窗口关联 (Sliding Window Co-occurrence)
 *  2. 对数权重增强 (Logarithmic Weight Growth)
 *  3. 距离衰减 (Distance Decay)
 *  4. 概念凝固 (Concept Solidification)
 *  5. 代谢与遗忘 (Decay & Pruning)
 *  6. 本体锚定 (Ontology Anchoring)
 * =====================================================
 */

const fs = require('fs');
const path = require('path');

// 配置常量
const CONFIG = {
  WINDOW_SIZE: 6,           // 滑动窗口大小 (5-8)
  DECAY_RATE: 0.95,         // 衰减率 (每次学习后)
  MIN_WEIGHT: 0.01,         // 最小权重阈值
  MIN_ENERGY: 0.1,          // 最小能量阈值
  ENERGY_PER_MENTION: 0.1,  // 每次提及增加的能量
  LOG_BASE: Math.E,         // 对数增长底数
  FOCUS_BOOST: 2.0,         // 焦点概念权重倍增
  MAX_TEXT_LENGTH: 50000   // 最大处理文本长度
};

class IncrementalLearner {
  constructor(brainPath = null) {
    this.brainPath = brainPath || path.join(__dirname, 'brain.json');
    this.brain = this.loadBrain();

    // 性能优化: 使用 Map 存储节点实现 O(1) 查找
    this.conceptMap = new Map();
    this.relationMap = new Map();

    // 初始化索引
    this._buildIndexes();
  }

  /**
   * 加载知识图谱
   */
  loadBrain() {
    try {
      const data = fs.readFileSync(this.brainPath, 'utf-8');
      return JSON.parse(data);
    } catch (error) {
      // 返回空知识库结构
      return {
        version: '2.0',
        lastUpdate: null,
        concepts: {},
        relations: {},
        meta: {
          totalConcepts: 0,
          totalRelations: 0,
          totalLearnCount: 0
        }
      };
    }
  }

  /**
   * 保存知识图谱
   */
  saveBrain() {
    this.brain.lastUpdate = new Date().toISOString();
    this.brain.meta.totalConcepts = this.conceptMap.size;
    this.brain.meta.totalRelations = this.relationMap.size;
    fs.writeFileSync(this.brainPath, JSON.stringify(this.brain, null, 2), 'utf-8');
  }

  /**
   * 构建索引以提高查找性能
   */
  _buildIndexes() {
    // 初始化概念 Map
    this.conceptMap.clear();
    for (const [name, data] of Object.entries(this.brain.concepts || {})) {
      this.conceptMap.set(name, data);
    }

    // 初始化关系 Map (双向索引)
    this.relationMap.clear();
    for (const [id, relation] of Object.entries(this.brain.relations || {})) {
      const key = this._makeRelationKey(relation.source, relation.target);
      this.relationMap.set(key, { id, ...relation });
    }
  }

  /**
   * 生成关系唯一键
   */
  _makeRelationKey(source, target) {
    // 确保一致性: 字母顺序小的在前
    return [source, target].sort().join('|||');
  }

  /**
   * =============================================
   * 核心 API
   * =============================================
   */

  /**
   * 从文本学习 - 核心入口
   * @param {string} text - 要学习的文本
   * @param {string|null} focusConcept - 焦点概念(本体锚定)
   * @returns {object} 学习结果统计
   */
  learnFromText(text, focusConcept = null) {
    const startTime = Date.now();

    // 文本预处理
    text = this._preprocessText(text);
    const tokens = this._tokenize(text);

    console.log(`[IncrementalLearner] Processing ${tokens.length} tokens, focus: ${focusConcept}`);

    let addedRelations = 0;
    let updatedConcepts = 0;

    // 滑动窗口处理
    for (let i = 0; i < tokens.length - 1; i++) {
      const window = tokens.slice(i, i + CONFIG.WINDOW_SIZE);

      // 计算窗口内每对词的关系
      for (let j = 0; j < window.length; j++) {
        for (let k = j + 1; k < window.length; k++) {
          const wordA = window[j];
          const wordB = window[k];

          // 跳过停用词和短词
          if (!this._isValidToken(wordA) || !this._isValidToken(wordB)) continue;

          // 计算距离衰减
          const distance = k - j;
          const distanceDecay = 1 / (1 + distance * 0.5);

          // 焦点概念权重奖励
          const focusBoost = this._calculateFocusBoost(wordA, wordB, focusConcept);

          // 更新关系权重
          const updated = this._updateRelation(wordA, wordB, distanceDecay, focusBoost);
          if (updated) addedRelations++;
        }
      }

      // 更新窗口内每个词的能量
      for (const token of window) {
        if (this._isValidToken(token)) {
          const updated = this._updateConceptEnergy(token, focusConcept);
          if (updated) updatedConcepts++;
        }
      }
    }

    const elapsed = Date.now() - startTime;
    console.log(`[IncrementalLearner] Completed in ${elapsed}ms, relations: ${addedRelations}, concepts: ${updatedConcepts}`);

    return {
      success: true,
      tokensProcessed: tokens.length,
      relationsAdded: addedRelations,
      conceptsUpdated: updatedConcepts,
      elapsedMs: elapsed,
      performance: elapsed < 100 ? 'excellent' : elapsed < 500 ? 'good' : 'slow'
    };
  }

  /**
   * 衰减与剪枝 - 防止内存溢出
   * @param {boolean} aggressive - 是否执行激进剪枝
   */
  cleanup(aggressive = false) {
    let prunedRelations = 0;
    let prunedConcepts = 0;

    // 衰减所有关系权重
    const relationIdsToDelete = [];
    for (const [id, relation] of Object.entries(this.brain.relations || {})) {
      relation.weight *= CONFIG.DECAY_RATE;
      relation.last_updated = Date.now();

      if (relation.weight < CONFIG.MIN_WEIGHT) {
        relationIdsToDelete.push(id);
      }
    }

    // 删除低权重关系
    for (const id of relationIdsToDelete) {
      delete this.brain.relations[id];
      prunedRelations++;
    }

    // 衰减所有概念能量
    const conceptNamesToDelete = [];
    for (const [name, concept] of Object.entries(this.brain.concepts || {})) {
      concept.energy = (concept.energy || 0) * CONFIG.DECAY_RATE;
      concept.lastSeen = new Date().toISOString();

      if (concept.energy < CONFIG.MIN_ENERGY) {
        conceptNamesToDelete.push(name);
      }
    }

    // 删除低能量概念
    for (const name of conceptNamesToDelete) {
      delete this.brain.concepts[name];
      prunedConcepts++;
    }

    // 激进模式下删除孤立节点（没有关联的概念）
    if (aggressive) {
      const connectedConcepts = new Set();
      for (const relation of Object.values(this.brain.relations || {})) {
        connectedConcepts.add(relation.source);
        connectedConcepts.add(relation.target);
      }

      for (const name of Object.keys(this.brain.concepts || {})) {
        if (!connectedConcepts.has(name)) {
          delete this.brain.concepts[name];
          prunedConcepts++;
        }
      }
    }

    this.saveBrain();
    this._buildIndexes();

    console.log(`[IncrementalLearner] Cleanup: pruned ${prunedRelations} relations, ${prunedConcepts} concepts`);

    return {
      prunedRelations,
      prunedConcepts,
      remainingRelations: Object.keys(this.brain.relations || {}).length,
      remainingConcepts: Object.keys(this.brain.concepts || {}).length
    };
  }

  /**
   * 获取概念信息
   */
  getConcept(name) {
    return this.conceptMap.get(name) || null;
  }

  /**
   * 获取与概念关联的所有节点
   */
  getRelatedConcepts(name, maxDepth = 2) {
    const related = new Map();
    const visited = new Set();

    function traverse(current, depth) {
      if (depth > maxDepth || visited.has(current)) return;
      visited.add(current);

      for (const [key, relation] of this.relationMap) {
        let neighbor = null;
        if (relation.source === current) neighbor = relation.target;
        else if (relation.target === current) neighbor = relation.source;

        if (neighbor && !visited.has(neighbor)) {
          related.set(neighbor, relation.weight);
          traverse.call(this, neighbor, depth + 1);
        }
      }
    }

    traverse.call(this, name, 0);
    return Object.fromEntries(related);
  }

  /**
   * =============================================
   * 内部方法
   * =============================================
   */

  /**
   * 文本预处理
   */
  _preprocessText(text) {
    if (!text || typeof text !== 'string') return '';

    // 限制长度
    text = text.substring(0, CONFIG.MAX_TEXT_LENGTH);

    // 统一空格、转小写
    text = text.replace(/\s+/g, ' ').trim();

    return text;
  }

  /**
   * 简单分词 (中英文混合)
   */
  _tokenize(text) {
    const tokens = [];

    // 中文: 2-4个连续汉字
    const chineseRegex = /[\u4e00-\u9fff]{2,4}/g;
    const chinese = text.match(chineseRegex) || [];
    tokens.push(...chinese);

    // 英文: 2+字母的单词
    const englishRegex = /[a-zA-Z]{2,}/g;
    const english = text.match(englishRegex) || [];
    tokens.push(...english.map(w => w.toLowerCase()));

    return tokens;
  }

  /**
   * 验证 token 是否有效
   */
  _isValidToken(token) {
    if (!token || token.length < 2) return false;

    // 过滤停用词
    const stopWords = new Set([
      '的', '是', '在', '了', '和', '与', '或', '有', '这', '那',
      'the', 'is', 'are', 'was', 'been', 'being', 'have', 'has',
      'and', 'but', 'or', 'not', 'this', 'that', 'with', 'for'
    ]);

    return !stopWords.has(token.toLowerCase());
  }

  /**
   * 计算焦点概念权重奖励
   */
  _calculateFocusBoost(wordA, wordB, focusConcept) {
    if (!focusConcept) return 1.0;

    const focus = focusConcept.toLowerCase();
    const aMatch = wordA.toLowerCase().includes(focus);
    const bMatch = wordB.toLowerCase().includes(focus);

    if (aMatch && bMatch) return CONFIG.FOCUS_BOOST;       // 都与焦点相关
    if (aMatch || bMatch) return Math.sqrt(CONFIG.FOCUS_BOOST); // 只有一个相关

    return 1.0;
  }

  /**
   * 更新关系权重
   * 使用对数增长防止权重爆炸
   */
  _updateRelation(source, target, distanceDecay = 1, focusBoost = 1) {
    const key = this._makeRelationKey(source, target);

    // 获取或创建关系
    let relation = this.relationMap.get(key);

    if (!relation) {
      // 创建新关系
      const id = `rel_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      relation = {
        id,
        source,
        target,
        weight: 0,  // 初始权重为0，后续增加
        count: 0,
        last_updated: Date.now()
      };
      this.brain.relations[id] = relation;
      this.relationMap.set(key, relation);
    }

    // 对数增长: log_base(count + 1)
    relation.count = (relation.count || 0) + 1;
    const logGrowth = Math.log(relation.count + 1) / Math.log(CONFIG.LOG_BASE);

    // 新权重 = 旧权重 + 对数增长 * 距离衰减 * 焦点奖励
    const weightIncrement = logGrowth * distanceDecay * focusBoost;
    relation.weight = Math.min(1, relation.weight + weightIncrement * 0.1);
    relation.last_updated = Date.now();

    return true;
  }

  /**
   * 更新概念能量
   */
  _updateConceptEnergy(token, focusConcept = null) {
    if (!this.brain.concepts[token]) {
      // 创建新概念节点
      this.brain.concepts[token] = {
        energy: CONFIG.ENERGY_PER_MENTION,
        count: 1,
        firstSeen: new Date().toISOString(),
        lastSeen: new Date().toISOString(),
        coOccurrences: {}
      };
    } else {
      // 更新现有概念
      const concept = this.brain.concepts[token];
      concept.energy += CONFIG.ENERGY_PER_MENTION;
      concept.count = (concept.count || 0) + 1;
      concept.lastSeen = new Date().toISOString();
    }

    // 焦点概念额外能量奖励
    if (focusConcept && token.toLowerCase().includes(focusConcept.toLowerCase())) {
      this.brain.concepts[token].energy += CONFIG.ENERGY_PER_MENTION;
    }

    // 更新索引
    this.conceptMap.set(token, this.brain.concepts[token]);

    return true;
  }

  /**
   * 获取统计信息
   */
  getStats() {
    return {
      totalConcepts: this.conceptMap.size,
      totalRelations: this.relationMap.size,
      avgWeight: this._calculateAverageWeight(),
      avgEnergy: this._calculateAverageEnergy(),
      topConcepts: this._getTopConcepts(10)
    };
  }

  _calculateAverageWeight() {
    const relations = Object.values(this.brain.relations || {});
    if (relations.length === 0) return 0;
    return relations.reduce((sum, r) => sum + r.weight, 0) / relations.length;
  }

  _calculateAverageEnergy() {
    const concepts = Object.values(this.brain.concepts || {});
    if (concepts.length === 0) return 0;
    return concepts.reduce((sum, c) => sum + (c.energy || 0), 0) / concepts.length;
  }

  _getTopConcepts(limit) {
    return Object.entries(this.brain.concepts || {})
      .sort((a, b) => (b[1].energy || 0) - (a[1].energy || 0))
      .slice(0, limit)
      .map(([name, data]) => ({ name, energy: data.energy, count: data.count }));
  }
}

module.exports = { IncrementalLearner, CONFIG };

// 如果直接运行
if (require.main === module) {
  const learner = new IncrementalLearner();

  // 测试学习
  const testText = `
    水是地球上最常见的物质之一。水的化学式是H2O，由两个氢原子和一个氧原子组成。
    水在常温下是无色透明的液体。水的沸点是100摄氏度，冰点是0摄氏度。
    水对所有生命形式都至关重要。人体约60%由水组成。植物需要水进行光合作用。
  `;

  console.log('\n=== 测试增量学习 ===\n');
  const result = learner.learnFromText(testText, '水');
  console.log('学习结果:', result);

  // 执行清理
  console.log('\n=== 执行清理(衰减) ===\n');
  const cleanupResult = learner.cleanup();
  console.log('清理结果:', cleanupResult);

  // 获取统计
  console.log('\n=== 统计信息 ===\n');
  console.log(learner.getStats());
}