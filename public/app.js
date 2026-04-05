/**
 * Seed-Intelligence 前端应用
 * 包含：Canvas 可视化（支持缩放）、API 调用、聊天界面
 */

// ============ 全局变量 ============
let simulation = null;
let canvas = null;
let ctx = null;
const width = 950;
const height = 380;

// 缩放相关
let scale = 1;
let offsetX = 0;
let offsetY = 0;
let isDragging = false;
let lastMouseX = 0;
let lastMouseY = 0;

// 数据
let nodes = [];
let links = [];

// 缩放阈值
const TEXT_SHOW_THRESHOLD = 1.5;  // 放大到 1.5x 以上才显示文字
const TEXT_HIDE_THRESHOLD = 0.8;  // 缩小到 0.8x 以下隐藏文字

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
const clearBrainBtn = document.getElementById('clear-brain');
const questionInput = document.getElementById('question-input');
const sendBtn = document.getElementById('send-btn');
const chatMessages = document.getElementById('chat-messages');
const brainViz = document.getElementById('brain-viz');

// 语言实验室元素
const lmPrompt = document.getElementById('lm-prompt');
const lmEpochs = document.getElementById('lm-epochs');
const lmTemp = document.getElementById('lm-temp');
const lmTrainingText = document.getElementById('lm-training-text');
const lmOutput = document.getElementById('lm-output');
const trainLmBtn = document.getElementById('train-lm-btn');
const generateBtn = document.getElementById('generate-btn');

// ============ 初始化 Canvas ============
function initCanvas() {
  brainViz.innerHTML = '';

  canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  canvas.style.cursor = 'grab';
  brainViz.appendChild(canvas);

  ctx = canvas.getContext('2d');

  // 鼠标事件 - 缩放
  canvas.addEventListener('wheel', handleWheel, { passive: false });

  // 鼠标事件 - 拖拽
  canvas.addEventListener('mousedown', handleMouseDown);
  canvas.addEventListener('mousemove', handleMouseMove);
  canvas.addEventListener('mouseup', handleMouseUp);
  canvas.addEventListener('mouseleave', handleMouseUp);

  // 触摸事件
  canvas.addEventListener('touchstart', handleTouchStart, { passive: false });
  canvas.addEventListener('touchmove', handleTouchMove, { passive: false });
  canvas.addEventListener('touchend', handleTouchEnd);
}

// 鼠标滚轮缩放
function handleWheel(e) {
  e.preventDefault();

  const rect = canvas.getBoundingClientRect();
  const mouseX = e.clientX - rect.left;
  const mouseY = e.clientY - rect.top;

  const delta = e.deltaY > 0 ? 0.9 : 1.1;
  const newScale = Math.max(0.1, Math.min(5, scale * delta));

  // 以鼠标为中心缩放
  offsetX = mouseX - (mouseX - offsetX) * (newScale / scale);
  offsetY = mouseY - (mouseY - offsetY) * (newScale / scale);
  scale = newScale;

  render();
}

// 鼠标拖拽
function handleMouseDown(e) {
  isDragging = true;
  lastMouseX = e.clientX;
  lastMouseY = e.clientY;
  canvas.style.cursor = 'grabbing';
}

function handleMouseMove(e) {
  if (!isDragging) return;

  const dx = e.clientX - lastMouseX;
  const dy = e.clientY - lastMouseY;

  offsetX += dx;
  offsetY += dy;

  lastMouseX = e.clientX;
  lastMouseY = e.clientY;

  render();
}

function handleMouseUp() {
  isDragging = false;
  canvas.style.cursor = 'grab';
}

// 触摸事件 - 双指缩放
let lastTouchDist = 0;
let lastTouchCenter = { x: 0, y: 0 };

function handleTouchStart(e) {
  if (e.touches.length === 2) {
    e.preventDefault();
    const dx = e.touches[0].clientX - e.touches[1].clientX;
    const dy = e.touches[0].clientY - e.touches[1].clientY;
    lastTouchDist = Math.sqrt(dx * dx + dy * dy);
    lastTouchCenter = {
      x: (e.touches[0].clientX + e.touches[1].clientX) / 2,
      y: (e.touches[0].clientY + e.touches[1].clientY) / 2
    };
  } else if (e.touches.length === 1) {
    isDragging = true;
    lastTouchCenter = { x: e.touches[0].clientX, y: e.touches[0].clientY };
  }
}

function handleTouchMove(e) {
  e.preventDefault();

  if (e.touches.length === 2) {
    const dx = e.touches[0].clientX - e.touches[1].clientX;
    const dy = e.touches[0].clientY - e.touches[1].clientY;
    const dist = Math.sqrt(dx * dx + dy * dy);
    const center = {
      x: (e.touches[0].clientX + e.touches[1].clientX) / 2,
      y: (e.touches[0].clientY + e.touches[1].clientY) / 2
    };

    if (lastTouchDist > 0) {
      const newScale = Math.max(0.1, Math.min(5, scale * (dist / lastTouchDist)));

      // 以双指中心缩放
      offsetX = center.x - (center.x - offsetX) * (newScale / scale);
      offsetY = center.y - (center.y - offsetY) * (newScale / scale);
      scale = newScale;

      render();
    }

    lastTouchDist = dist;
    lastTouchCenter = center;
  } else if (e.touches.length === 1 && isDragging) {
    const dx = e.touches[0].clientX - lastTouchCenter.x;
    const dy = e.touches[0].clientY - lastTouchCenter.y;

    offsetX += dx;
    offsetY += dy;

    lastTouchCenter = { x: e.touches[0].clientX, y: e.touches[0].clientY };
    render();
  }
}

function handleTouchEnd() {
  isDragging = false;
  lastTouchDist = 0;
}

// ============ 渲染 ============
function render() {
  if (!ctx) return;

  // 清空画布
  ctx.fillStyle = '#0f1419';
  ctx.fillRect(0, 0, width, height);

  ctx.save();
  ctx.translate(offsetX, offsetY);
  ctx.scale(scale, scale);

  // 绘制边
  ctx.strokeStyle = '#2d3640';
  ctx.lineWidth = 1 / scale;  // 保持线条细度
  links.forEach(link => {
    if (!link.source.x || !link.source.y || !link.target.x || !link.target.y) return;

    ctx.beginPath();
    ctx.moveTo(link.source.x, link.source.y);
    ctx.lineTo(link.target.x, link.target.y);

    // 边的透明度基于权重
    ctx.globalAlpha = Math.min(1, link.weight * 1.5);
    ctx.stroke();
  });

  ctx.globalAlpha = 1;

  // 绘制节点
  const showText = scale >= TEXT_SHOW_THRESHOLD;

  nodes.forEach(node => {
    if (!node.x || !node.y) return;

    const radius = Math.max(3, Math.min(15, node.weight * 0.8));

    // 节点颜色基于权重
    const hue = 120 + node.weight * 10;  // 绿色系
    const saturation = 60 + node.weight * 5;
    ctx.fillStyle = `hsl(${hue}, ${saturation}%, 50%)`;

    ctx.beginPath();
    ctx.arc(node.x, node.y, radius, 0, Math.PI * 2);
    ctx.fill();

    // 节点边框
    ctx.strokeStyle = '#fff';
    ctx.lineWidth = 1.5 / scale;
    ctx.stroke();

    // 绘制文字（仅当放大时）
    if (showText && node.id.length > 0) {
      ctx.fillStyle = '#e8eaed';
      ctx.font = `${Math.max(8, 12 / scale)}px -apple-system, sans-serif`;
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';

      const text = node.id.length > 10 ? node.id.substring(0, 8) + '...' : node.id;
      ctx.fillText(text, node.x, node.y + radius + 8 / scale);
    }
  });

  ctx.restore();
}

// ============ 事件监听 ============
startBtn.addEventListener('click', startLearning);
seedInput.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') startLearning();
});

refreshBtn.addEventListener('click', loadBrain);
clearBrainBtn.addEventListener('click', clearBrain);
sendBtn.addEventListener('click', sendQuestion);
questionInput.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') sendQuestion();
});

// 语言实验室事件
trainLmBtn.addEventListener('click', trainLanguageModel);
generateBtn.addEventListener('click', generateText);
lmPrompt.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') generateText();
});

// ============ 学习功能 ============
async function startLearning() {
  const concept = seedInput.value.trim();
  if (!concept) {
    addMessage('请输入一个概念', 'bot', true);
    return;
  }

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

      const learned = result.newConcepts
        ? [concept, ...result.newConcepts].join('、')
        : concept;
      learnedConcepts.textContent = learned;

      addMessage(`学习 "${concept}" 成功！我发现了 ${result.keywords?.length || 0} 个关联概念。`, 'bot');

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

    conceptCount.textContent = brain.meta?.totalConcepts || Object.keys(brain.concepts || {}).length;
    relationCount.textContent = brain.meta?.totalRelations || Object.keys(brain.relations || {}).length;

    renderBrain(brain);

  } catch (error) {
    console.error('Load brain error:', error);
  }
}

async function clearBrain() {
  if (!confirm('确定要清空所有知识吗？此操作不可恢复！')) {
    return;
  }

  try {
    const response = await fetch('/api/clear', {
      method: 'POST'
    });

    if (response.ok) {
      alert('知识库已清空');
      // 重置视图
      nodes = [];
      links = [];
      render();
      loadBrain();
    } else {
      alert('清空失败');
    }
  } catch (error) {
    alert('清空失败: ' + error.message);
  }
}

function renderBrain(brain) {
  // 初始化 Canvas
  initCanvas();

  const concepts = brain.concepts || {};

  // relations 现在是对象，需要转换为数组
  let relationsArray = [];
  if (Array.isArray(brain.relations)) {
    relationsArray = brain.relations;
  } else if (brain.relations && typeof brain.relations === 'object') {
    relationsArray = Object.values(brain.relations);
  }

  // 准备节点数据
  nodes = Object.entries(concepts).map(([name, data]) => ({
    id: name,
    x: Math.random() * width,
    y: Math.random() * height,
    weight: data.energy || data.weight || 1,
    count: data.count || 1
  }));

  if (nodes.length === 0) {
    ctx.fillStyle = '#9aa0a6';
    ctx.font = '16px -apple-system, sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText('知识库为空，请先注入初始概念', width / 2, height / 2);
    return;
  }

  // 准备边数据
  links = relationsArray.map(r => ({
    source: r.source || r.from,
    target: r.target || r.to,
    weight: r.weight || 0.5
  })).filter(l =>
    nodes.find(n => n.id === l.source) &&
    nodes.find(n => n.id === l.target)
  );

  // 创建节点映射
  const nodeMap = {};
  nodes.forEach(n => nodeMap[n.id] = n);

  // 更新 links 的源和目标为节点对象
  links.forEach(l => {
    l.source = nodeMap[l.source];
    l.target = nodeMap[l.target];
  });

  // 使用 D3 力导向布局
  if (typeof d3 !== 'undefined') {
    simulation = d3.forceSimulation(nodes)
      .force('link', d3.forceLink(links).id(d => d.id).distance(60))
      .force('charge', d3.forceManyBody().strength(-100))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(d => Math.max(10, d.weight)))
      .alpha(0.3)
      .on('tick', render);
  } else {
    // 如果没有 D3，简单随机分布
    render();
  }

  // 自动适应视图
  fitToView();
}

function fitToView() {
  if (nodes.length === 0) return;

  // 计算边界
  let minX = Infinity, maxX = -Infinity;
  let minY = Infinity, maxY = -Infinity;

  nodes.forEach(n => {
    if (n.x < minX) minX = n.x;
    if (n.x > maxX) maxX = n.x;
    if (n.y < minY) minY = n.y;
    if (n.y > maxY) maxY = n.y;
  });

  const padding = 50;
  const graphWidth = maxX - minX + padding * 2;
  const graphHeight = maxY - minY + padding * 2;

  // 计算缩放比例
  const scaleX = width / graphWidth;
  const scaleY = height / graphHeight;
  scale = Math.min(scaleX, scaleY, 1);

  // 居中
  offsetX = (width - graphWidth * scale) / 2 + padding * scale - minX * scale;
  offsetY = (height - graphHeight * scale) / 2 + padding * scale - minY * scale;

  render();
}

// ============ 问答功能 ============
async function sendQuestion() {
  const question = questionInput.value.trim();
  if (!question) return;

  addMessage(question, 'user');
  questionInput.value = '';

  const loadingMsg = addMessage('思考中...', 'bot');

  try {
    const response = await fetch('/api/ask', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ question })
    });

    const result = await response.json();

    loadingMsg.remove();

    addMessage(result.answer, 'bot');

    if (result.confidence > 0) {
      setTimeout(() => {
        addMessage(`（置信度: ${result.confidence}%，涉及 ${result.foundEntities?.length || 0} 个概念）`, 'bot');
      }, 100);
    }

    if (result.associations && result.associations.length > 0) {
      setTimeout(() => {
        const assocText = result.associations.slice(0, 3).map(a =>
          `${a.target || a.name} (${Math.round((a.weight || 0.5) * 100)}%)`
        ).join('、');
        addMessage(`相关概念: ${assocText}`, 'bot');
      }, 200);
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

// ============ 语言实验室功能 ============
async function trainLanguageModel() {
  const text = lmTrainingText.value.trim();
  if (!text || text.length < 10) {
    lmOutput.textContent = '错误: 请输入至少 10 个字符的训练文本';
    return;
  }

  const epochs = parseInt(lmEpochs.value) || 3;

  trainLmBtn.disabled = true;
  trainLmBtn.textContent = '训练中...';
  lmOutput.textContent = '正在训练语言模型...';

  try {
    const response = await fetch('/api/train-lm', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ text, epochs })
    });

    const result = await response.json();

    if (result.success) {
      lmOutput.textContent = `训练完成！词表大小: ${result.vocabSize}`;
    } else {
      lmOutput.textContent = `训练失败: ${result.error}`;
    }
  } catch (error) {
    lmOutput.textContent = `错误: ${error.message}`;
  } finally {
    trainLmBtn.disabled = false;
    trainLmBtn.textContent = '训练模型';
  }
}

async function generateText() {
  const prompt = lmPrompt.value.trim();
  if (!prompt) {
    lmOutput.textContent = '错误: 请输入起始句子';
    return;
  }

  const maxTokens = 50;
  const temperature = parseFloat(lmTemp.value) || 0.8;

  generateBtn.disabled = true;
  generateBtn.textContent = '生成中...';
  lmOutput.textContent = '正在生成...';

  try {
    // 使用 SSE 流式生成
    const response = await fetch('/api/generate-stream', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ prompt, maxTokens, temperature })
    });

    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    lmOutput.textContent = '';

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      const text = decoder.decode(value);
      const lines = text.split('\n');

      for (const line of lines) {
        if (line.startsWith('data: ')) {
          const data = line.slice(6);
          if (data === '[DONE]') {
            generateBtn.disabled = false;
            generateBtn.textContent = '生成';
            return;
          }

          try {
            const parsed = JSON.parse(data);
            if (parsed.token) {
              lmOutput.textContent += parsed.token;
            } else if (parsed.error) {
              lmOutput.textContent = `错误: ${parsed.error}`;
            }
          } catch (e) {
            // 忽略解析错误
          }
        }
      }
    }
  } catch (error) {
    lmOutput.textContent = `错误: ${error.message}`;
  } finally {
    generateBtn.disabled = false;
    generateBtn.textContent = '生成';
  }
}

// ============ 初始化 ============
document.addEventListener('DOMContentLoaded', () => {
  initCanvas();
  loadBrain();
});