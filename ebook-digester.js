/**
 * =====================================================
 *  E-book Digester - 电子书流式消化器
 *  主动感官 - 自主阅读本地书籍
 *
 *  功能:
 *  - 流式读取 txt 文件 (不加载到内存)
 *  - 停用词过滤
 *  - 速率控制 (模拟阅读速度)
 *  - 概念提取后交给学习器
 *
 *  性能约束:
 *  - 内存占用 < 20MB
 *  - CPU 友好 (速率控制)
 * =====================================================
 */

const fs = require('fs');
const readline = require('readline');
const { spawn } = require('child_process');
const path = require('path');
const { IncrementalLearner } = require('./incremental-learner');

const CONFIG = {
  LINES_PER_SECOND: 8,        // 每秒处理行数 (模拟阅读速度)
  MIN_LINE_LENGTH: 5,        // 最小有效行长度
  BATCH_SIZE: 10,            // 每批处理的行数
  SANDBOX_PATH: './agent_sandbox', // 沙盒路径
  TEMP_DIR: './temp',        // 临时文件目录
  SUPPORTED_EBOOK_FORMATS: ['.epub', '.mobi', '.azw3', '.azw', '.kf8']
};

// 停用词表
const STOP_WORDS = new Set([
  // 中文停用词
  '的', '是', '在', '了', '和', '与', '或', '有', '这', '那', '个', '一', '不', '也',
  '都', '就', '而', '及', '以', '对', '可', '能', '会', '被', '于', '从', '到', '把',
  '将', '为', '但', '却', '又', '如', '因', '所', '并', '其', '之', '来', '去', '上',
  '下', '中', '大', '小', '多', '少', '最', '更', '很', '太', '过', '要', '该', '我们',
  '你们', '他们', '她们', '它们', '这个', '那个', '什么', '怎么', '如何', '为什么',
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
  'it', 'its', 'i', 'me', 'my', 'we', 'our', 'you', 'your', 'he', 'she', 'him', 'her'
]);

class EbookDigester {
  constructor(learner = null) {
    this.learner = learner || new IncrementalLearner();
    this.isReading = false;
    this.currentFile = null;
    this.linesProcessed = 0;
    this.conceptsLearned = 0;
    this.sandboxPath = CONFIG.SANDBOX_PATH;
    this.tempPath = CONFIG.TEMP_DIR;
    this.currentTempFile = null;
  }

  /**
   * 检查 Calibre 是否可用
   */
  async checkCalibre() {
    return new Promise((resolve) => {
      const proc = spawn('ebook-convert', ['--version'], { shell: true });
      let output = '';

      proc.stdout.on('data', (data) => { output += data; });
      proc.stderr.on('data', (data) => { output += data; });

      proc.on('close', (code) => {
        if (code === 0) {
          resolve({ available: true, version: output.trim() });
        } else {
          resolve({ available: false, version: null });
        }
      });

      proc.on('error', () => {
        resolve({ available: false, version: null });
      });

      // 超时保护
      setTimeout(() => {
        proc.kill();
        resolve({ available: false, version: null });
      }, 3000);
    });
  }

  /**
   * 初始化目录
   */
  async initSandbox() {
    try {
      // 确保沙盒目录
      if (!fs.existsSync(this.sandboxPath)) {
        fs.mkdirSync(this.sandboxPath, { recursive: true });
        console.log(`[Digester] 创建沙盒目录: ${this.sandboxPath}`);
      }

      // 确保临时目录
      if (!fs.existsSync(this.tempPath)) {
        fs.mkdirSync(this.tempPath, { recursive: true });
        console.log(`[Digester] 创建临时目录: ${this.tempPath}`);
      }
    } catch (error) {
      console.error('[Digester] 目录创建失败:', error.message);
    }
  }

  /**
   * 检查是否是电子书格式
   */
  isEbookFormat(filePath) {
    const ext = path.extname(filePath).toLowerCase();
    return CONFIG.SUPPORTED_EBOOK_FORMATS.includes(ext);
  }

  /**
   * 转换电子书为 TXT (异步)
   */
  convertToTxt(inputPath) {
    return new Promise(async (resolve, reject) => {
      await this.initSandbox();

      const ext = path.extname(inputPath).toLowerCase();
      const baseName = path.basename(inputPath, ext);
      const tempTxtPath = path.join(this.tempPath, `${baseName}_${Date.now()}.txt`);

      console.log(`[Digester] 转换电子书: ${inputPath} -> ${tempTxtPath}`);

      // 检查 Calibre
      const calibre = await this.checkCalibre();
      if (!calibre.available) {
        reject(new Error('Calibre 未安装。请运行: sudo apt install calibre'));
        return;
      }

      const proc = spawn('ebook-convert', [inputPath, tempTxtPath], {
        shell: true,
        stdio: ['ignore', 'pipe', 'pipe']
      });

      let stderr = '';

      proc.stderr.on('data', (data) => {
        stderr += data.toString();
      });

      proc.on('close', (code) => {
        if (code === 0 && fs.existsSync(tempTxtPath)) {
          this.currentTempFile = tempTxtPath;
          resolve(tempTxtPath);
        } else {
          reject(new Error(`转换失败: ${stderr || '未知错误'}`));
        }
      });

      proc.on('error', (err) => {
        reject(new Error(`转换进程错误: ${err.message}`));
      });

      // 超时 5 分钟
      setTimeout(() => {
        proc.kill();
        reject(new Error('转换超时'));
      }, 300000);
    });
  }

  /**
   * 清理临时文件
   */
  cleanupTempFile(tempPath = null) {
    const fileToClean = tempPath || this.currentTempFile;
    if (fileToClean && fs.existsSync(fileToClean)) {
      try {
        fs.unlinkSync(fileToClean);
        console.log(`[Digester] 清理临时文件: ${fileToClean}`);
      } catch (e) {
        console.error(`[Digester] 清理失败: ${e.message}`);
      }
    }
    this.currentTempFile = null;
  }

  /**
   * 过滤停用词，提取关键词
   */
  extractKeywords(text) {
    const tokens = [];
    const seen = new Set();

    // 中文分词 (2-4个连续汉字)
    const chineseRegex = /[\u4e00-\u9fff]{2,4}/g;
    const chinese = text.match(chineseRegex) || [];
    chinese.forEach(t => {
      if (!STOP_WORDS.has(t) && !seen.has(t)) {
        seen.add(t);
        tokens.push(t);
      }
    });

    // 英文分词
    const englishRegex = /[a-zA-Z]{2,}/g;
    const english = text.match(englishRegex) || [];
    english.forEach(t => {
      const lower = t.toLowerCase();
      if (!STOP_WORDS.has(lower) && !seen.has(lower)) {
        seen.add(lower);
        tokens.push(lower);
      }
    });

    return tokens;
  }

  /**
   * 流式读取并学习书籍 (支持 TXT 和电子书格式)
   */
  async readBook(filePath, focusConcept = null) {
    if (this.isReading) {
      return { success: false, error: '已经在阅读中' };
    }

    // 验证文件路径
    if (!fs.existsSync(filePath)) {
      return { success: false, error: '文件不存在' };
    }

    this.isReading = true;
    this.currentFile = filePath;
    this.linesProcessed = 0;
    this.conceptsLearned = 0;

    let actualFilePath = filePath;
    let tempTxtPath = null;
    const isEbook = this.isEbookFormat(filePath);

    console.log(`[Digester] 开始阅读: ${filePath} ${isEbook ? '(电子书格式)' : '(TXT)'}`);

    // 如果是电子书格式，先转换
    if (isEbook) {
      try {
        tempTxtPath = await this.convertToTxt(filePath);
        actualFilePath = tempTxtPath;
        console.log(`[Digester] 转换完成，开始阅读`);
      } catch (error) {
        this.isReading = false;
        return { success: false, error: error.message };
      }
    }

    // 核心处理逻辑 - 使用 try-catch-finally 确保清理
    let result;
    try {
      result = await this._processFile(actualFilePath, focusConcept);
    } catch (error) {
      result = { success: false, error: error.message };
    } finally {
      // 强制垃圾回收: 无论成功、失败还是中断，都删除临时文件
      this.cleanupTempFile(tempTxtPath);
      this.isReading = false;
    }

    return result;
  }

  /**
   * 内部方法: 处理文件内容
   */
  async _processFile(filePath, focusConcept) {
    const fileStream = fs.createReadStream(filePath, { encoding: 'utf-8' });
    const rl = readline.createInterface({
      input: fileStream,
      crlfDelay: Infinity
    });

    let lineBuffer = [];
    let lineCount = 0;

    for await (const line of rl) {
      lineCount++;

      // 过滤空行和太短的行
      const cleanedLine = line.trim();
      if (cleanedLine.length < CONFIG.MIN_LINE_LENGTH) continue;

      lineBuffer.push(cleanedLine);

      // 达到批次大小时处理
      if (lineBuffer.length >= CONFIG.BATCH_SIZE) {
        await this.processBatch(lineBuffer, focusConcept);
        lineBuffer = [];

        // 速率控制: 模拟阅读速度 (非阻塞)
        await this.sleep(1000 / CONFIG.LINES_PER_SECOND * CONFIG.BATCH_SIZE);
      }
    }

    // 处理剩余内容
    if (lineBuffer.length > 0) {
      await this.processBatch(lineBuffer, focusConcept);
    }

    // 清理
    const cleanupResult = this.learner.cleanup(false);
    this.learner.saveBrain();

    console.log(`[Digester] 阅读完成: ${lineCount} 行, ${this.conceptsLearned} 个概念`);

    return {
      success: true,
      linesProcessed: lineCount,
      conceptsLearned: this.conceptsLearned,
      cleanup: cleanupResult
    };
  }

  /**
   * 处理一批文本
   */
  async processBatch(lines, focusConcept) {
    const fullText = lines.join(' ');

    // 提取关键词
    const keywords = this.extractKeywords(fullText);

    if (keywords.length > 0) {
      // 让学习器学习这些概念
      const result = this.learner.learnFromText(fullText, focusConcept);
      this.conceptsLearned += keywords.length;
    }

    this.linesProcessed += lines.length;

    // 进度输出 (每100行)
    if (this.linesProcessed % 100 < CONFIG.BATCH_SIZE) {
      process.stdout.write(`\r[Digester] 已处理: ${this.linesProcessed} 行...`);
    }
  }

  /**
   * 睡眠辅助函数
   */
  sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * 获取状态
   */
  getStatus() {
    return {
      isReading: this.isReading,
      currentFile: this.currentFile,
      linesProcessed: this.linesProcessed,
      conceptsLearned: this.conceptsLearned
    };
  }

  /**
   * 停止阅读
   */
  stop() {
    this.isReading = false;
    console.log('[Digester] 已停止阅读');
  }
}

/**
 * 创建沙盒目录
 */
function ensureSandbox() {
  const sandboxPath = CONFIG.SANDBOX_PATH;
  if (!fs.existsSync(sandboxPath)) {
    fs.mkdirSync(sandboxPath, { recursive: true });
    console.log(`[Digester] 创建沙盒: ${sandboxPath}`);
  }
  return sandboxPath;
}

/**
 * 列出沙盒中的书籍
 */
function listBooks() {
  ensureSandbox();
  const files = fs.readdirSync(CONFIG.SANDBOX_PATH)
    .filter(f => f.endsWith('.txt'))
    .map(f => {
      const stats = fs.statSync(`${CONFIG.SANDBOX_PATH}/${f}`);
      return { name: f, size: stats.size, modified: stats.mtime };
    });
  return files;
}

module.exports = { EbookDigester, CONFIG, ensureSandbox, listBooks };

// 如果直接运行
if (require.main === module) {
  const digester = new EbookDigester();

  // 确保沙盒存在
  ensureSandbox();

  // 检查命令行参数
  const filePath = process.argv[2];
  if (!filePath) {
    console.log(`
╔═══════════════════════════════════════════╗
║      📚 E-book Digester 使用指南          ║
╠═══════════════════════════════════════════╣
║  用法: node ebook-digester.js <文件路径>  ║
║                                              ║
║  示例:                                       ║
║    node ebook-digester.js ./agent_sandbox/book.txt  ║
║                                              ║
║  配置:                                       ║
║    阅读速度: ${CONFIG.LINES_PER_SECOND} 行/秒                ║
║    批次大小: ${CONFIG.BATCH_SIZE} 行                      ║
╚═══════════════════════════════════════════╝
`);
    process.exit(0);
  }

  // 开始阅读
  digester.readBook(filePath).then(result => {
    console.log('\n阅读结果:', result);
  }).catch(err => {
    console.error('错误:', err);
  });
}