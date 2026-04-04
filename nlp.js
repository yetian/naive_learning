/**
 * NLP 模块 - 轻量级中英文分词
 * 功能：分词、词频统计、共现关系提取
 */

// 常见中文词汇字典 (常用 2-4 字词)
const COMMON_CHINESE_WORDS = new Set([
  // 常见名词
  '苹果', '水果', '手机', '电脑', '网络', '软件', '硬件', '系统', '数据', '信息',
  '技术', '科学', '学习', '教育', '学校', '学生', '老师', '工作', '公司', '产品',
  '服务', '用户', '客户', '市场', '价格', '质量', '时间', '空间', '世界', '中国',
  '美国', '日本', '欧洲', '亚洲', '国际', '政治', '经济', '文化', '历史', '社会',
  '生命', '水', '空气', '光', '热', '温度', '能量', '物质', '分子', '原子',
  // 常见动词
  '学习', '工作', '生活', '使用', '开发', '设计', '创造', '管理', '组织', '计划',
  '开始', '结束', '进行', '完成', '发展', '变化', '增长', '减少', '提高', '降低',
  // 常见形容词
  '重要', '简单', '复杂', '困难', '容易', '快速', '慢速', '高效', '低效', '现代',
  '传统', '新', '旧', '大', '小', '长', '短', '高', '低', '好', '坏',
  // 常见虚词 (保留用于上下文)
  '什么', '怎么', '如何', '为什么', '因为', '所以', '但是', '如果', '虽然', '只是'
]);

// 停用词列表
const STOP_WORDS = new Set([
  // 中文停用词
  '的', '是', '在', '了', '和', '与', '或', '有', '这', '那', '个', '一', '不', '也',
  '都', '就', '而', '及', '以', '对', '可', '能', '会', '被', '于', '从', '到', '把',
  '将', '为', '但', '却', '又', '如', '因', '所', '并', '其', '之', '来', '去', '上',
  '下', '中', '大', '小', '多', '少', '最', '更', '很', '太', '过', '要', '该', '我们',
  '你们', '他们', '她们', '它们', '这个', '那个', '可以', '没有', '这样', '那样',
  '自己', '已经', '因为', '所以', '但是', '而且', '或者', '如果', '虽然', '只是',
  '就是', '还是', '应该', '需要', '可能', '关于', '什么', '怎么', '如何', '为什么',
  // 英文停用词
  'the', 'a', 'an', 'is', 'are', 'was', 'were', 'be', 'been', 'being', 'have', 'has',
  'had', 'do', 'does', 'did', 'will', 'would', 'could', 'should', 'may', 'might',
  'must', 'shall', 'can', 'need', 'dare', 'ought', 'used', 'to', 'of', 'in', 'for',
  'on', 'with', 'at', 'by', 'from', 'as', 'into', 'through', 'during', 'before',
  'after', 'above', 'below', 'between', 'under', 'again', 'further', 'then', 'once',
  'and', 'but', 'or', 'nor', 'so', 'yet', 'both', 'either', 'neither', 'not', 'only',
  'own', 'same', 'than', 'too', 'very', 'just', 'also', 'now', 'here', 'there', 'when',
  'where', 'why', 'how', 'all', 'each', 'every', 'few', 'more', 'most', 'other', 'some',
  'such', 'no', 'any', 'what', 'which', 'who', 'whom', 'this', 'that', 'these', 'those',
  'it', 'its', 'i', 'me', 'my', 'we', 'our', 'you', 'your', 'he', 'she', 'him', 'her', 'his'
]);

/**
 * 智能分词 - 混合中英文
 * 策略：优先匹配常见词汇，然后使用 n-gram
 */
function tokenize(text) {
  if (!text || typeof text !== 'string') return [];

  const tokens = [];
  const seen = new Set();

  // 1. 尝试匹配常见中文词汇 (2-4字)
  let remaining = text;
  const wordMatches = [];

  // 先找出所有可能匹配的位置
  COMMON_CHINESE_WORDS.forEach(word => {
    let pos = 0;
    while (true) {
      const idx = remaining.indexOf(word, pos);
      if (idx === -1) break;
      wordMatches.push({ word, start: idx, end: idx + word.length });
      pos = idx + 1;
    }
  });

  // 按起始位置排序
  wordMatches.sort((a, b) => a.start - b.start);

  // 提取匹配的词 (避免重叠)
  const matchedRanges = [];
  wordMatches.forEach(match => {
    const overlaps = matchedRanges.some(r =>
      (match.start >= r.start && match.start < r.end) ||
      (match.end > r.start && match.end <= r.end)
    );
    if (!overlaps) {
      matchedRanges.push(match);
      if (!seen.has(match.word)) {
        seen.add(match.word);
        tokens.push(match.word);
      }
    }
  });

  // 2. 提取英文单词 (2+字母)
  const englishRegex = /[a-zA-Z]{2,}/g;
  const englishWords = text.match(englishRegex) || [];
  englishWords.forEach(w => {
    const lower = w.toLowerCase();
    if (!STOP_WORDS.has(lower) && !seen.has(lower)) {
      seen.add(lower);
      tokens.push(lower);
    }
  });

  // 3. 提取数字
  const numberRegex = /\d+/g;
  const numbers = text.match(numberRegex) || [];
  numbers.forEach(n => {
    if (n.length >= 2 && !seen.has(n)) {
      seen.add(n);
      tokens.push(n);
    }
  });

  // 4. 对未覆盖的中文字符，使用 2-gram
  let chineseOnly = text;
  matchedRanges.forEach(r => {
    chineseOnly = chineseOnly.replace(r.word, ' '.repeat(r.word.length));
  });

  // 移除已处理的英文字母和数字
  chineseOnly = chineseOnly.replace(/[a-zA-Z0-9]/g, ' ');

  // 提取剩余的中文 2-gram
  const chineseNgram = chineseOnly.match(/[\u4e00-\u9fff]{2}/g) || [];
  chineseNgram.forEach(gram => {
    if (!STOP_WORDS.has(gram) && !seen.has(gram)) {
      seen.add(gram);
      tokens.push(gram);
    }
  });

  // 5. 额外：尝试提取 3-gram 关键词
  const chinese3gram = text.match(/[\u4e00-\u9fff]{3}/g) || [];
  chinese3gram.forEach(gram => {
    if (!STOP_WORDS.has(gram) && !seen.has(gram) && !COMMON_CHINESE_WORDS.has(gram)) {
      // 只保留看起来像词的 (非随机组合)
      if (gram.includes('的') || gram.includes('了') || gram.includes('是')) {
        // 包含虚词的跳过
      } else {
        seen.add(gram);
        tokens.push(gram);
      }
    }
  });

  return tokens;
}

/**
 * 过滤停用词
 */
function filterStopWords(tokens) {
  return tokens.filter(token => {
    if (token.length < 2) return false;
    if (STOP_WORDS.has(token.toLowerCase())) return false;
    return true;
  });
}

/**
 * 统计词频
 */
function countWordFrequency(texts, topN = 20) {
  const frequency = {};

  texts.forEach(text => {
    const tokens = tokenize(text);
    tokens.forEach(token => {
      frequency[token] = (frequency[token] || 0) + 1;
    });
  });

  const sorted = Object.entries(frequency)
    .sort((a, b) => b[1] - a[1])
    .slice(0, topN);

  return Object.fromEntries(sorted);
}

/**
 * 计算共现关系
 */
function extractCoOccurrences(targetConcept, texts, minFrequency = 1) {
  const coOccurrences = {};

  texts.forEach(text => {
    const tokens = tokenize(text);
    const hasTarget = tokens.some(t => t.includes(targetConcept) || targetConcept.includes(t));

    if (hasTarget) {
      tokens.forEach(token => {
        if (!token.includes(targetConcept) && !targetConcept.includes(token)) {
          coOccurrences[token] = (coOccurrences[token] || 0) + 1;
        }
      });
    }
  });

  return Object.fromEntries(
    Object.entries(coOccurrences).filter(([_, count]) => count >= minFrequency)
  );
}

/**
 * 提取关键词
 */
function extractKeywords(texts, targetConcept, topN = 15) {
  const wordFreq = countWordFrequency(texts, topN * 2);
  const coOccurrences = extractCoOccurrences(targetConcept, texts);

  const keywords = {};
  const allWords = new Set([...Object.keys(wordFreq), ...Object.keys(coOccurrences)]);

  allWords.forEach(word => {
    const freqScore = wordFreq[word] || 0;
    const coocScore = coOccurrences[word] || 0;
    keywords[word] = freqScore * 0.4 + coocScore * 0.6;
  });

  const sorted = Object.entries(keywords)
    .sort((a, b) => b[1] - a[1])
    .slice(0, topN);

  return sorted.map(([word, score]) => ({ word, score: Math.round(score * 100) / 100 }));
}

/**
 * 提取上下文
 */
function extractContexts(texts, targetConcept, contextLength = 50) {
  const contexts = [];

  texts.forEach(text => {
    const index = text.indexOf(targetConcept);
    if (index !== -1) {
      const start = Math.max(0, index - contextLength);
      const end = Math.min(text.length, index + targetConcept.length + contextLength);
      let context = text.substring(start, end);

      if (start > 0) context = '...' + context;
      if (end < text.length) context = context + '...';

      contexts.push(context);
    }
  });

  return contexts.slice(0, 5);
}

module.exports = {
  tokenize,
  filterStopWords,
  countWordFrequency,
  extractCoOccurrences,
  extractKeywords,
  extractContexts,
  STOP_WORDS
};