#!/usr/bin/env node
/**
 * =====================================================
 *  Clipboard Watcher - 剪贴板监听器
 *  伴随式学习感知器官
 *
 *  功能:
 *  - 监听剪贴板变化
 *  - 清洗文本内容
 *  - POST 到后端 /api/learn-text
 *
 *  性能约束:
 *  - 内存占用 < 20MB
 *  - CPU 友好 (休眠轮询)
 * =====================================================
 */

const clipboardy = require('clipboardy');
const http = require('http');

const CONFIG = {
  API_ENDPOINT: 'http://localhost:3000/api/learn-text',
  POLL_INTERVAL: 1500,      // 1.5秒轮询 (CPU友好)
  MIN_TEXT_LENGTH: 10,     // 最小文本长度
  MAX_TEXT_LENGTH: 5000,   // 最大文本长度
  SERVER_HOST: 'localhost',
  SERVER_PORT: 3000
};

// 状态变量
let lastClipboard = '';
let isRunning = true;

/**
 * 清洗文本
 */
function cleanText(text) {
  if (!text || typeof text !== 'string') return '';

  // 移除多余空白
  text = text.replace(/\s+/g, ' ').trim();

  // 移除特殊符号 (保留中文、英文、数字、基本标点)
  text = text.replace(/[^\u4e00-\u9fa5a-zA-Z0-9\s,.!?;:'"()-]/g, '');

  // 限制长度
  if (text.length > CONFIG.MAX_TEXT_LENGTH) {
    text = text.substring(0, CONFIG.MAX_TEXT_LENGTH);
  }

  return text;
}

/**
 * 检查是否是有效文本
 */
function isValidText(text) {
  if (!text || text.length < CONFIG.MIN_TEXT_LENGTH) return false;

  // 检查是否包含中文或英文
  const hasChinese = /[\u4e00-\u9fa5]/.test(text);
  const hasEnglish = /[a-zA-Z]{3,}/.test(text);

  return hasChinese || hasEnglish;
}

/**
 * 发送到后端
 */
function sendToBackend(text, focusConcept = null) {
  const postData = JSON.stringify({
    text,
    focusConcept
  });

  const url = new URL(CONFIG.API_ENDPOINT);

  const options = {
    hostname: url.hostname,
    port: url.port || CONFIG.SERVER_PORT,
    path: url.pathname,
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Content-Length': Buffer.byteLength(postData)
    },
    timeout: 5000
  };

  const req = http.request(options, (res) => {
    let data = '';

    res.on('data', (chunk) => {
      data += chunk;
    });

    res.on('end', () => {
      try {
        const result = JSON.parse(data);
        console.log(`[Watcher] 学习结果: 概念数=${result.conceptsCount}, 关系数=${result.relationsCount}`);
      } catch (e) {
        console.error('[Watcher] 解析响应失败:', e.message);
      }
    });
  });

  req.on('error', (error) => {
    console.error('[Watcher] 发送失败:', error.message);
  });

  req.on('timeout', () => {
    req.destroy();
    console.error('[Watcher] 请求超时');
  });

  req.write(postData);
  req.end();
}

/**
 * 主循环
 */
function watch() {
  if (!isRunning) return;

  try {
    const currentClipboard = clipboardy.readSync();

    // 检测到新内容
    if (currentClipboard && currentClipboard !== lastClipboard) {
      lastClipboard = currentClipboard;

      const cleanedText = cleanText(currentClipboard);

      if (isValidText(cleanedText)) {
        console.log(`[Watcher] 检测到新文本: "${cleanedText.substring(0, 50)}..."`);
        sendToBackend(cleanedText);
      }
    }
  } catch (error) {
    // 忽略剪贴板读取错误 (可能是权限问题)
  }

  // 休眠轮询 (CPU 友好)
  setTimeout(watch, CONFIG.POLL_INTERVAL);
}

/**
 * 优雅退出
 */
function gracefulShutdown() {
  console.log('\n[Watcher] 正在停止...');
  isRunning = false;
  process.exit(0);
}

process.on('SIGINT', gracefulShutdown);
process.on('SIGTERM', gracefulShutdown);

// 启动
console.log(`
╔═══════════════════════════════════════════╗
║      📋 Clipboard Watcher 启动           ║
╠═══════════════════════════════════════════╣
║  监听间隔: ${CONFIG.POLL_INTERVAL}ms                     ║
║  最小文本: ${CONFIG.MIN_TEXT_LENGTH}字符                     ║
║  目标API: ${CONFIG.API_ENDPOINT}       ║
╚═══════════════════════════════════════════╝
`);

// 初始化剪贴板内容 (避免启动时就发送)
try {
  lastClipboard = clipboardy.readSync();
} catch (e) {
  console.log('[Watcher] 无法读取初始剪贴板，将从下一个变化开始监听');
}

// 开始监听
watch();