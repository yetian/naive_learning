/**
 * Seed-Intelligence 前端应用
 * 包含：D3.js 可视化、API 调用、聊天界面
 */

// ============ 全局变量 ============
let simulation = null;
let svg = null;
const width = 950;
const height = 380;

// ============ DOM 元素 ============
const seedInput = document.getElementById('seed-concept');
const startBtn = document.getElementById('start-btn');
const autoLearnCheckbox = document.getElementById('auto-learn');
const statusSection = document.getElementById('status-section');
const statusText = document.getElementById('status-text');
const progressFill = document.getElementById('progress-fill');
const learnedConcepts = document.getElementById('learned-concepts');
const conceptCount = document.getElementById('concept-count');
const relationCount = document.getElementById('relation-count');
const refreshBtn = document.getElementById('refresh-brain');
const questionInput = document.getElementById('question-input');
const sendBtn = document.getElementById('send-btn');
const chatMessages = document.getElementById('chat-messages');
const brainViz = document.getElementById('brain-viz');

// ============ 事件监听 ============
startBtn.addEventListener('click', startLearning);
seedInput.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') startLearning();
});

refreshBtn.addEventListener('click', loadBrain);
sendBtn.addEventListener('click', sendQuestion);
questionInput.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') sendQuestion();
});

// ============ 学习功能 ============
async function startLearning() {
  const concept = seedInput.value.trim();
  if (!concept) {
    addMessage('请输入一个概念', 'bot', true);
    return;
  }

  // 禁用按钮
  setLoading(true);
  statusSection.style.display = 'block';
  statusText.textContent = `正在学习 "${concept}"...`;
  progressFill.style.width = '30%';

  try {
    const response = await fetch('/api/init', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        concept,
        autoLearn: autoLearnCheckbox.checked
      })
    });

    const result = await response.json();

    if (result.success) {
      statusText.textContent = `学习完成！`;
      progressFill.style.width = '100%';

      // 显示已学习的概念
      const learned = result.newConcepts
        ? [concept, ...result.newConcepts].join('、')
        : concept;
      learnedConcepts.textContent = learned;

      addMessage(`学习 "${concept}" 成功！我发现了 ${result.keywords.length} 个关联概念。`, 'bot');

      // 刷新可视化
      setTimeout(loadBrain, 500);
    } else {
      addMessage(`学习失败: ${result.error}`, 'bot', true);
    }

  } catch (error) {
    addMessage(`错误: ${error.message}`, 'bot', true);
  } finally {
    setLoading(false);
    seedInput.value = '';
  }
}

function setLoading(loading) {
  startBtn.disabled = loading;
  startBtn.querySelector('.btn-text').style.display = loading ? 'none' : 'inline';
  startBtn.querySelector('.btn-loading').style.display = loading ? 'inline' : 'none';
}

// ============ 知识图谱加载与可视化 ============
async function loadBrain() {
  try {
    const response = await fetch('/api/brain');
    const brain = await response.json();

    // 更新统计
    conceptCount.textContent = brain.meta?.totalConcepts || 0;
    relationCount.textContent = brain.meta?.totalRelations || 0;

    // 渲染可视化
    renderBrain(brain);

  } catch (error) {
    console.error('Load brain error:', error);
  }
}

function renderBrain(brain) {
  // 清空画布
  brainViz.innerHTML = '';

  const concepts = brain.concepts || {};
  const relations = brain.relations || [];

  // 准备节点数据
  const nodes = Object.entries(concepts).map(([name, data]) => ({
    id: name,
    weight: data.weight || 1,
    count: Object.keys(data.coOccurrences || {}).length
  }));

  // 准备边数据
  const links = relations.map(r => ({
    source: r.from,
    target: r.to,
    weight: r.weight || 0.5
  })).filter(l => nodes.find(n => n.id === l.source) && nodes.find(n => n.id === l.target));

  if (nodes.length === 0) {
    brainViz.innerHTML = `
      <div class="empty-state">
        <span>🧠</span>
        <p>知识库为空，请先注入初始概念</p>
      </div>
    `;
    return;
  }

  // 创建SVG
  svg = d3.select('#brain-viz')
    .append('svg')
    .attr('viewBox', [0, 0, width, height]);

  // 颜色比例尺
  const colorScale = d3.scaleSequential()
    .domain([0, d3.max(nodes, d => d.weight) || 1])
    .interpolator(d3.interpolateYlGn);

  // 节点大小比例
  const sizeScale = d3.scaleSqrt()
    .domain([1, d3.max(nodes, d => d.weight) || 5])
    .range([6, 20]);

  // 力导向模拟
  simulation = d3.forceSimulation(nodes)
    .force('link', d3.forceLink(links).id(d => d.id).distance(80))
    .force('charge', d3.forceManyBody().strength(-200))
    .force('center', d3.forceCenter(width / 2, height / 2))
    .force('collision', d3.forceCollide().radius(d => sizeScale(d.weight) + 5));

  // 绘制边
  const link = svg.append('g')
    .selectAll('line')
    .data(links)
    .join('line')
    .attr('class', 'link')
    .attr('stroke-width', d => Math.max(1, d.weight * 3));

  // 绘制节点
  const node = svg.append('g')
    .selectAll('g')
    .data(nodes)
    .join('g')
    .attr('class', 'node')
    .call(d3.drag()
      .on('start', dragstarted)
      .on('drag', dragged)
      .on('end', dragended));

  // 节点圆圈
  node.append('circle')
    .attr('r', d => sizeScale(d.weight))
    .attr('fill', d => colorScale(d.weight))
    .append('title')
    .text(d => `${d.id}\n权重: ${d.weight.toFixed(2)}\n关联: ${d.count} 个`);

  // 节点标签
  node.append('text')
    .attr('dx', d => sizeScale(d.weight) + 4)
    .attr('dy', 4)
    .text(d => d.id.length > 12 ? d.id.substring(0, 10) + '...' : d.id);

  // 模拟tick更新
  simulation.on('tick', () => {
    link
      .attr('x1', d => d.source.x)
      .attr('y1', d => d.source.y)
      .attr('x2', d => d.target.x)
      .attr('y2', d => d.target.y);

    node.attr('transform', d => `translate(${d.x},${d.y})`);
  });

  // 拖拽函数
  function dragstarted(event) {
    if (!event.active) simulation.alphaTarget(0.3).restart();
    event.subject.fx = event.subject.x;
    event.subject.fy = event.subject.y;
  }

  function dragged(event) {
    event.subject.fx = event.x;
    event.subject.fy = event.y;
  }

  function dragended(event) {
    if (!event.active) simulation.alphaTarget(0);
    event.subject.fx = null;
    event.subject.fy = null;
  }
}

// ============ 问答功能 ============
async function sendQuestion() {
  const question = questionInput.value.trim();
  if (!question) return;

  // 显示用户消息
  addMessage(question, 'user');
  questionInput.value = '';

  // 显示加载状态
  const loadingMsg = addMessage('思考中...', 'bot');

  try {
    const response = await fetch('/api/query', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ question })
    });

    const result = await response.json();

    // 移除加载消息
    loadingMsg.remove();

    // 显示回答
    addMessage(result.answer, 'bot');

    // 如果有路径信息，显示置信度
    if (result.confidence > 0) {
      setTimeout(() => {
        addMessage(`（置信度: ${result.confidence}%，涉及 ${result.concepts?.length || 0} 个概念）`, 'bot');
      }, 100);
    }

  } catch (error) {
    loadingMsg.remove();
    addMessage(`错误: ${error.message}`, 'bot', true);
  }
}

function addMessage(text, type, isError = false) {
  const div = document.createElement('div');
  div.className = `message ${type}${isError ? ' error' : ''}`;
  div.textContent = text;
  chatMessages.appendChild(div);
  chatMessages.scrollTop = chatMessages.scrollHeight;
  return div;
}

// ============ 初始化 ============
document.addEventListener('DOMContentLoaded', () => {
  loadBrain();
  // 定时刷新知识图谱（每30秒）
  setInterval(loadBrain, 30000);
});