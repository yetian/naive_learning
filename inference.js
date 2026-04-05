/**
 * 推理引擎 - 基于路径聚合的回答生成
 * 适配 IncrementalLearner 的数据结构
 */

const fs = require('fs');
const path = require('path');
const axios = require('axios');
const { tokenize, filterStopWords } = require('./nlp');

/**
 * 获取 Wikipedia 摘要
 */
async function fetchWikipediaSummary(query) {
  // 先尝试中文 Wikipedia
  try {
    const response = await axios.get(
      `https://zh.wikipedia.org/api/rest_v1/page/summary/${encodeURIComponent(query)}`,
      { timeout: 5000 }
    );
    if (response.data?.extract) {
      return `[来源: 维基百科] ${response.data.extract}`;
    }
  } catch (e) {
    // 忽略错误，继续尝试英文
  }

  // 再尝试英文 Wikipedia
  try {
    const response = await axios.get(
      `https://en.wikipedia.org/api/rest_v1/page/summary/${encodeURIComponent(query)}`,
      { timeout: 5000 }
    );
    if (response.data?.extract) {
      return `[Source: Wikipedia] ${response.data.extract}`;
    }
  } catch (e) {
    // 忽略
  }

  return null;
}

// 加载知识图谱
function loadBrain() {
  const brainPath = path.join(__dirname, 'brain.json');
  try {
    const data = fs.readFileSync(brainPath, 'utf-8');
    return JSON.parse(data);
  } catch (error) {
    return { concepts: {}, relations: {}, meta: {} };
  }
}

/**
 * 分词用户问题
 */
function parseQuery(question) {
  const tokens = tokenize(question);
  return filterStopWords(tokens);
}

/**
 * 查找问题关键词对应的节点
 */
function findMatchingConcepts(queryWords, brain) {
  const matches = [];
  const matchedNames = new Set();

  queryWords.forEach(word => {
    const lowerWord = word.toLowerCase();

    // 精确匹配 (优先)
    if (brain.concepts[word] && !matchedNames.has(word)) {
      matches.push({ word, concept: brain.concepts[word], exact: true });
      matchedNames.add(word);
    }

    // 模糊匹配 (只在没精确匹配时)
    if (!matchedNames.has(word)) {
      Object.entries(brain.concepts).forEach(([conceptName, conceptData]) => {
        if (conceptName.toLowerCase().includes(lowerWord) &&
            !matchedNames.has(conceptName)) {
          matches.push({ word, concept: conceptData, exact: false });
          matchedNames.add(conceptName);
        }
      });
    }
  });

  // 去重：按概念名去重，保留能量最高的
  const uniqueMatches = [];
  const seen = new Set();

  matches.forEach(m => {
    if (!seen.has(m.word)) {
      seen.add(m.word);
      uniqueMatches.push(m);
    }
  });

  return uniqueMatches;
}

/**
 * BFS 查找关联路径 - 适配新的 relations 对象结构
 */
function findPaths(startConcept, brain, maxDepth = 3) {
  const paths = [];
  const visited = new Set();

  // 构建邻接表
  const adjacency = {};
  for (const [id, relation] of Object.entries(brain.relations || {})) {
    if (!adjacency[relation.source]) adjacency[relation.source] = [];
    if (!adjacency[relation.target]) adjacency[relation.target] = [];

    adjacency[relation.source].push({ node: relation.target, weight: relation.weight });
    adjacency[relation.target].push({ node: relation.source, weight: relation.weight });
  }

  function bfs(current, target, depth, path) {
    if (depth > maxDepth) return;
    if (current === target && target !== null) {
      paths.push([...path, current]);
      return;
    }

    visited.add(current);

    // 找到当前节点的所有连接，按权重排序
    const connections = (adjacency[current] || [])
      .sort((a, b) => b.weight - a.weight);

    for (const conn of connections) {
      const next = conn.node;
      if (!visited.has(next)) {
        path.push(next);
        bfs(next, target, depth + 1, path);
        path.pop();
      }
    }

    visited.delete(current);
  }

  bfs(startConcept, null, 0, [startConcept]);
  return paths;
}

/**
 * 聚合路径信息生成回答 - 适配新数据结构
 */
function aggregateAnswer(paths, brain, originalQuestion) {
  if (paths.length === 0) {
    return {
      answer: '我的知识库中还没有关于这个概念的信息。让我先学习一下！',
      confidence: 0,
      paths: []
    };
  }

  // 收集所有路径上的概念信息
  const allConcepts = new Set();
  const allRelations = [];

  paths.forEach(path => {
    for (let i = 0; i < path.length - 1; i++) {
      allConcepts.add(path[i]);
      allConcepts.add(path[i + 1]);

      // 查找关系 - 适配对象结构
      const key = [path[i], path[i + 1]].sort().join('|||');
      for (const [id, relation] of Object.entries(brain.relations || {})) {
        const relKey = [relation.source, relation.target].sort().join('|||');
        if (relKey === key) {
          allRelations.push(relation);
          break;
        }
      }
    }
  });

  // 构建回答
  let answer = '';
  const mainConcepts = Array.from(allConcepts).slice(0, 5);

  if (mainConcepts.length === 1) {
    const concept = brain.concepts[mainConcepts[0]];
    answer = `关于"${mainConcepts[0]}"，据我所知：\n`;
    if (concept && concept.energy) {
      answer += `这是一个重要概念，能量值为 ${concept.energy.toFixed(2)}，出现在 ${concept.count} 个上下文中。`;
    } else {
      answer += `这是一个重要的概念。`;
    }
  } else {
    answer = `根据我的知识图谱，`;
    answer += mainConcepts.slice(0, 3).join('、');
    answer += ' 等概念相互关联。\n\n';

    // 添加关联信息
    if (allRelations.length > 0) {
      const topRelations = allRelations
        .sort((a, b) => b.weight - a.weight)
        .slice(0, 3);

      answer += '它们的关系：\n';
      topRelations.forEach(r => {
        answer += `• ${r.source} ↔ ${r.target} (关联度: ${Math.round(r.weight * 100)}%)\n`;
      });
    }
  }

  // 计算置信度
  const avgWeight = allRelations.length > 0
    ? allRelations.reduce((sum, r) => sum + r.weight, 0) / allRelations.length
    : 0.5;

  return {
    answer,
    confidence: Math.round(avgWeight * 100),
    paths: paths.slice(0, 5),
    concepts: mainConcepts
  };
}

/**
 * 主查询函数
 */
function query(question) {
  const brain = loadBrain();

  // 分词问题
  const queryWords = parseQuery(question);

  if (queryWords.length === 0) {
    return {
      answer: '请告诉我你想了解什么？',
      confidence: 0,
      paths: []
    };
  }

  console.log('[Query] Parsed words:', queryWords);

  // 查找匹配的概念
  const matches = findMatchingConcepts(queryWords, brain);

  console.log('[Query] Matched concepts:', matches.map(m => m.word));

  if (matches.length === 0) {
    return {
      answer: `我还不了解"${queryWords[0]}"相关的知识。要我学习一下吗？`,
      confidence: 0,
      paths: []
    };
  }

  // 从最匹配的概念开始查找路径
  let allPaths = [];
  const processedConcepts = new Set();

  for (const match of matches) {
    if (!processedConcepts.has(match.word)) {
      const paths = findPaths(match.word, brain, 2);
      allPaths.push(...paths);
      processedConcepts.add(match.word);

      // 也从其他匹配概念查找
      for (const otherMatch of matches) {
        if (otherMatch.word !== match.word) {
          const morePaths = findPaths(match.word, brain, 2);
          allPaths.push(...morePaths);
        }
      }
    }
  }

  // 去重
  allPaths = allPaths.filter((path, index, self) =>
    index === self.findIndex(p => p.join(',') === path.join(','))
  );

  // 生成回答
  return aggregateAnswer(allPaths, brain, question);
}

/**
 * 获取概念详情 - 适配 IncrementalLearner 数据结构
 */
function getConceptDetails(conceptName, brain) {
  const concept = brain.concepts[conceptName];
  if (!concept) return null;

  return {
    name: conceptName,
    energy: concept.energy,
    count: concept.count,
    firstSeen: concept.firstSeen,
    lastSeen: concept.lastSeen
  };
}

/**
 * 迪杰斯特拉算法 - 找出两节点间权重最大的路径
 */
function dijkstra(start, end, brain) {
  // 构建邻接表
  const adjacency = {};
  for (const [id, relation] of Object.entries(brain.relations || {})) {
    if (!adjacency[relation.source]) adjacency[relation.source] = [];
    if (!adjacency[relation.target]) adjacency[relation.target] = [];

    adjacency[relation.source].push({ node: relation.target, weight: relation.weight });
    adjacency[relation.target].push({ node: relation.source, weight: relation.weight });
  }

  // 距离/权重存储 (使用权重作为"距离"，所以取负值使最短路径=最高权重)
  const dist = {};
  const prev = {};
  const visited = new Set();

  // 初始化
  for (const node of Object.keys(brain.concepts || {})) {
    dist[node] = -Infinity;
  }
  dist[start] = 0;

  // 优先队列模拟 (按权重排序)
  const pq = [{ node: start, weight: 0 }];

  while (pq.length > 0) {
    // 取出权重最大的节点
    pq.sort((a, b) => b.weight - a.weight);
    const current = pq.shift();

    if (visited.has(current.node)) continue;
    visited.add(current.node);

    // 到达目标
    if (current.node === end) break;

    // 遍历邻居
    const neighbors = adjacency[current.node] || [];
    for (const neighbor of neighbors) {
      if (visited.has(neighbor.node)) continue;

      // 计算新路径权重
      const newWeight = current.weight + neighbor.weight;

      if (newWeight > dist[neighbor.node]) {
        dist[neighbor.node] = newWeight;
        prev[neighbor.node] = current.node;
        pq.push({ node: neighbor.node, weight: newWeight });
      }
    }
  }

  // 回溯路径
  if (!prev[end] && end !== start) return null;

  const path = [];
  let current = end;
  while (current) {
    path.unshift(current);
    current = prev[current];
  }

  return path.length > 0 ? path : null;
}

/**
 * 查找两节点间的最佳路径
 */
function findBestPath(nodeA, nodeB, brain) {
  const path = dijkstra(nodeA, nodeB, brain);
  if (!path) return null;

  // 提取路径上的边权重
  const pathDetails = [];
  for (let i = 0; i < path.length - 1; i++) {
    const source = path[i];
    const target = path[i + 1];

    // 查找连接
    for (const [id, relation] of Object.entries(brain.relations || {})) {
      if ((relation.source === source && relation.target === target) ||
          (relation.source === target && relation.target === source)) {
        pathDetails.push({
          from: source,
          to: target,
          weight: relation.weight
        });
        break;
      }
    }
  }

  return { path, pathDetails, totalWeight: pathDetails.reduce((sum, r) => sum + r.weight, 0) };
}

/**
 * 表达引擎 - /ask 接口
 */
async function ask(question) {
  const brain = loadBrain();
  const startTime = Date.now();

  // 1. 查询解析: 提取核心实体
  const queryWords = parseQuery(question);
  const matchedConcepts = findMatchingConcepts(queryWords, brain);

  if (matchedConcepts.length === 0) {
    return {
      answer: `我的知识库中还没有关于"${question}"的信息。要我学习一下吗？`,
      confidence: 0,
      elapsedMs: Date.now() - startTime
    };
  }

  let answer = '';
  // 去重 - 使用 Set 去除重复的概念名
  const uniqueEntities = [...new Set(matchedConcepts.map(m => m.word))];

  // 2. 只有一个节点: 自由联想
  if (uniqueEntities.length === 1) {
    const concept = uniqueEntities[0];

    // 检查是否需要 Wikipedia 摘要
    const isWhatQuestion = /是[什么怎].*|什么.*|怎么.*|如何.*/.test(question);
    const needsWiki = isWhatQuestion && top5.length > 0;

    // 获取 Wikipedia 摘要
    let wikiInfo = null;
    if (needsWiki) {
      try {
        wikiInfo = await fetchWikipediaSummary(concept);
      } catch (e) {
        console.log('[ask] Wikipedia fetch failed:', e.message);
      }
    }

    // 找权重最高的5个关联
    const connections = [];
    for (const [id, relation] of Object.entries(brain.relations || {})) {
      if (relation.source === concept) {
        connections.push({ target: relation.target, weight: relation.weight });
      } else if (relation.target === concept) {
        connections.push({ target: relation.source, weight: relation.weight });
      }
    }

    connections.sort((a, b) => b.weight - a.weight);
    const top5 = connections.slice(0, 5);

    // 构建回答
    if (wikiInfo) {
      answer = `${wikiInfo}\n\n`;
    }

    if (top5.length === 0) {
      answer += `关于"${concept}"，我只知道这一个概念，还没有发现它与其他概念的关联。`;
    } else {
      answer += `关于"${concept}"，我联想到以下概念：\n`;
      top5.forEach((conn, i) => {
        answer += `${i + 1}. ${conn.target} (关联度: ${Math.round(conn.weight * 100)}%)\n`;
      });
    }

    return {
      answer,
      confidence: top5.length > 0 ? Math.round(top5[0].weight * 100) : 0,
      foundEntities: uniqueEntities,
      associations: top5,
      wikiSource: wikiInfo ? 'wikipedia' : null,
      elapsedMs: Date.now() - startTime
    };
  }

  // 3. 两个或多个节点: 路径查找
  if (uniqueEntities.length >= 2) {
    // 找所有节点对之间的路径
    const allPaths = [];

    for (let i = 0; i < uniqueEntities.length; i++) {
      for (let j = i + 1; j < uniqueEntities.length; j++) {
        const nodeA = uniqueEntities[i];
        const nodeB = uniqueEntities[j];

        const pathResult = findBestPath(nodeA, nodeB, brain);
        if (pathResult) {
          allPaths.push({
            from: nodeA,
            to: nodeB,
            ...pathResult
          });
        }
      }
    }

    if (allPaths.length === 0) {
      answer = `我找到了概念: ${uniqueEntities.join('、')}，但它们之间还没有建立关联路径。`;
    } else {
      // 按总权重排序，返回最佳路径
      allPaths.sort((a, b) => b.totalWeight - a.totalWeight);
      const best = allPaths[0];

      answer = `我找到了逻辑链：\n`;
      let currentNode = best.path[0];
      for (let i = 0; i < best.pathDetails.length; i++) {
        const edge = best.pathDetails[i];
        const nextNode = best.path[i + 1];
        answer += `• ${currentNode} (权重 ${Math.round(edge.weight * 100)}%) -> ${nextNode}\n`;
        currentNode = nextNode;
      }

      answer += `\n总关联度: ${Math.round(best.totalWeight * 100)}%`;
    }

    return {
      answer,
      confidence: allPaths.length > 0 ? Math.round(allPaths[0].totalWeight * 100 / allPaths[0].path.length) : 0,
      foundEntities: uniqueEntities,
      paths: allPaths.slice(0, 3),
      elapsedMs: Date.now() - startTime
    };
  }
}

module.exports = {
  query,
  ask,
  parseQuery,
  findMatchingConcepts,
  findPaths,
  findBestPath,
  dijkstra,
  aggregateAnswer,
  getConceptDetails,
  loadBrain
};