/**
 * =====================================================
 *  Embodied Sandbox - 具身控制安全沙盒
 *  智能体的"物理宇宙"
 *
 *  功能:
 *  - touch(filename): 创建文件
 *  - write(filename, content): 写入数据
 *  - look(): 返回目录文件列表及总大小
 *  - destroy(filename): 删除文件
 *
 *  安全机制:
 *  - 路径校验 (防止 ../ 逃逸)
 *  - 严格的白名单操作
 *  - 异常捕获与反馈
 * =====================================================
 */

const fs = require('fs');
const path = require('path');

const CONFIG = {
  SANDBOX_PATH: './agent_sandbox',
  MAX_FILE_SIZE: 10 * 1024 * 1024,  // 10MB 单文件限制
  ALLOWED_EXTENSIONS: ['.txt', '.json', '.md', '.log', '.csv']
};

class SandboxEnvironment {
  constructor(sandboxPath = null) {
    this.sandboxPath = path.resolve(sandboxPath || CONFIG.SANDBOX_PATH);
    this.ensureSandbox();
  }

  /**
   * 确保沙盒目录存在
   */
  ensureSandbox() {
    try {
      if (!fs.existsSync(this.sandboxPath)) {
        fs.mkdirSync(this.sandboxPath, { recursive: true });
        console.log(`[Sandbox] 创建沙盒: ${this.sandboxPath}`);
      }
    } catch (error) {
      console.error('[Sandbox] 创建沙盒失败:', error.message);
    }
  }

  /**
   * 校验路径是否在沙盒内 (安全核心)
   */
  _validatePath(filename) {
    // 解析绝对路径
    const absolutePath = path.resolve(this.sandboxPath, filename);

    // 确保路径在沙盒内 (不能包含 ../)
    if (!absolutePath.startsWith(this.sandboxPath)) {
      throw new Error('SECURITY_VIOLATION: 路径试图逃逸沙盒');
    }

    // 检查非法路径组件
    const pathParts = absolutePath.split(path.sep);
    if (pathParts.includes('..')) {
      throw new Error('SECURITY_VIOLATION: 非法路径组件');
    }

    // 检查文件扩展名
    const ext = path.extname(absolutePath).toLowerCase();
    if (ext && !CONFIG.ALLOWED_EXTENSIONS.includes(ext)) {
      throw new Error(`FORBIDDEN_EXTENSION: 不允许的文件类型 ${ext}`);
    }

    return absolutePath;
  }

  /**
   * touch(filename): 创建空文件
   */
  touch(filename) {
    try {
      const fullPath = this._validatePath(filename);

      // 检查文件是否已存在
      if (fs.existsSync(fullPath)) {
        return {
          action: 'touch',
          success: false,
          filename,
          error: 'FILE_EXISTS',
          message: '文件已存在'
        };
      }

      // 创建文件
      fs.writeFileSync(fullPath, '', 'utf-8');
      const stats = fs.statSync(fullPath);

      return {
        action: 'touch',
        success: true,
        filename,
        size: stats.size,
        created: stats.birthtime
      };

    } catch (error) {
      return {
        action: 'touch',
        success: false,
        filename,
        error: error.name || 'ERROR',
        message: error.message
      };
    }
  }

  /**
   * write(filename, content): 写入数据
   */
  write(filename, content) {
    try {
      const fullPath = this._validatePath(filename);

      // 检查文件大小限制
      const contentSize = Buffer.byteLength(content, 'utf-8');
      if (contentSize > CONFIG.MAX_FILE_SIZE) {
        return {
          action: 'write',
          success: false,
          filename,
          error: 'FILE_TOO_LARGE',
          message: `内容过大: ${contentSize} bytes (最大 ${CONFIG.MAX_FILE_SIZE})`
        };
      }

      // 获取写入前的大小
      let beforeSize = 0;
      if (fs.existsSync(fullPath)) {
        const stats = fs.statSync(fullPath);
        beforeSize = stats.size;
      }

      // 写入文件
      fs.writeFileSync(fullPath, content, 'utf-8');
      const stats = fs.statSync(fullPath);

      return {
        action: 'write',
        success: true,
        filename,
        beforeSize,
        afterSize: stats.size,
        deltaSize: stats.size - beforeSize,
        written: contentSize
      };

    } catch (error) {
      return {
        action: 'write',
        success: false,
        filename,
        error: error.name || 'ERROR',
        message: error.message
      };
    }
  }

  /**
   * read(filename): 读取文件内容
   */
  read(filename) {
    try {
      const fullPath = this._validatePath(filename);

      if (!fs.existsSync(fullPath)) {
        return {
          action: 'read',
          success: false,
          filename,
          error: 'ENOENT',
          message: '文件不存在'
        };
      }

      const content = fs.readFileSync(fullPath, 'utf-8');
      const stats = fs.statSync(fullPath);

      return {
        action: 'read',
        success: true,
        filename,
        content: content.substring(0, 1000),  // 限制返回大小
        size: stats.size,
        truncated: content.length > 1000
      };

    } catch (error) {
      return {
        action: 'read',
        success: false,
        filename,
        error: error.name || 'ERROR',
        message: error.message
      };
    }
  }

  /**
   * look(): 返回目录文件列表及总大小
   */
  look() {
    try {
      if (!fs.existsSync(this.sandboxPath)) {
        return {
          action: 'look',
          success: false,
          error: 'SANDBOX_NOT_FOUND',
          message: '沙盒目录不存在'
        };
      }

      const files = fs.readdirSync(this.sandboxPath);
      let totalSize = 0;
      const fileList = [];

      for (const file of files) {
        const fullPath = path.join(this.sandboxPath, file);
        try {
          const stats = fs.statSync(fullPath);
          if (stats.isFile()) {
            fileList.push({
              name: file,
              size: stats.size,
              modified: stats.mtime,
              created: stats.birthtime
            });
            totalSize += stats.size;
          }
        } catch (e) {
          // 忽略单个文件的错误
        }
      }

      return {
        action: 'look',
        success: true,
        sandboxPath: this.sandboxPath,
        fileCount: fileList.length,
        totalSize,
        files: fileList
      };

    } catch (error) {
      return {
        action: 'look',
        success: false,
        error: error.name || 'ERROR',
        message: error.message
      };
    }
  }

  /**
   * destroy(filename): 删除文件
   */
  destroy(filename) {
    try {
      const fullPath = this._validatePath(filename);

      if (!fs.existsSync(fullPath)) {
        return {
          action: 'destroy',
          success: false,
          filename,
          error: 'ENOENT',
          message: '文件不存在'
        };
      }

      const stats = fs.statSync(fullPath);
      const fileSize = stats.size;

      // 删除文件
      fs.unlinkSync(fullPath);

      return {
        action: 'destroy',
        success: true,
        filename,
        freedSize: fileSize
      };

    } catch (error) {
      return {
        action: 'destroy',
        success: false,
        filename,
        error: error.name || 'ERROR',
        message: error.message
      };
    }
  }

  /**
   * execute(action, ...args): 统一执行入口
   */
  execute(action, ...args) {
    switch (action) {
      case 'touch':
        return this.touch(args[0]);
      case 'write':
        return this.write(args[0], args[1]);
      case 'read':
        return this.read(args[0]);
      case 'look':
        return this.look();
      case 'destroy':
        return this.destroy(args[0]);
      default:
        return {
          action,
          success: false,
          error: 'UNKNOWN_ACTION',
          message: `未知操作: ${action}`
        };
    }
  }
}

/**
 * 动作反馈接口 - 用于记录到 brain.json
 */
function formatActionFeedback(result) {
  const feedback = {
    action: result.action,
    success: result.success,
    timestamp: new Date().toISOString()
  };

  if (result.success) {
    feedback.sensation = 'success';
    feedback.details = {};
    if (result.filename) feedback.details.filename = result.filename;
    if (result.size) feedback.details.size = result.size;
    if (result.afterSize) feedback.details.afterSize = result.afterSize;
  } else {
    feedback.sensation = result.error === 'SECURITY_VIOLATION' ? 'pain' : 'void';
    feedback.details = {
      error: result.error,
      message: result.message
    };
  }

  return feedback;
}

module.exports = { SandboxEnvironment, CONFIG, formatActionFeedback };

// 如果直接运行
if (require.main === module) {
  const sandbox = new SandboxEnvironment();

  console.log(`
╔═══════════════════════════════════════════╗
║      🎭 Embodied Sandbox 测试            ║
╚═══════════════════════════════════════════╝
`);

  // 测试 touch
  console.log('\n1. 测试 touch:');
  console.log(sandbox.touch('test.txt'));

  // 测试 write
  console.log('\n2. 测试 write:');
  console.log(sandbox.write('test.txt', 'Hello from Seed-Intelligence!'));

  // 测试 look
  console.log('\n3. 测试 look:');
  console.log(sandbox.look());

  // 测试 read
  console.log('\n4. 测试 read:');
  console.log(sandbox.read('test.txt'));

  // 测试 destroy
  console.log('\n5. 测试 destroy:');
  console.log(sandbox.destroy('test.txt'));

  // 测试安全校验
  console.log('\n6. 测试安全校验 (尝试逃逸):');
  console.log(sandbox.write('../../../etc/passwd', 'hacked'));
}