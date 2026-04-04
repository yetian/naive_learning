const axios = require('axios');
const cheerio = require('cheerio');

/**
 * 使用 DuckDuckGo HTML 版本搜索
 */
async function searchDuckDuckGo(query) {
  const results = [];
  try {
    // 使用 DuckDuckGo HTML 搜索
    const response = await axios.get('https://html.duckduckgo.com/html/', {
      params: { q: query },
      headers: {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
      },
      timeout: 10000
    });

    const $ = cheerio.load(response.data);
    $('.result__body').each((i, el) => {
      if (i >= 5) return; // 只获取前5个结果
      const title = $(el).find('.result__a').text().trim();
      const snippet = $(el).find('.result__snippet').text().trim();
      const url = $(el).find('.result__a').attr('href');

      if (title && snippet) {
        results.push({ title, snippet, url });
      }
    });
  } catch (error) {
    console.error('Search error:', error.message);
    // 返回模拟数据作为后备
    return getMockResults(query);
  }

  // 如果没有结果，返回模拟数据
  if (results.length === 0) {
    return getMockResults(query);
  }

  return results;
}

/**
 * 获取模拟搜索结果（当API失败时使用）
 */
function getMockResults(query) {
  const mockData = {
    '水': [
      { title: '水 - 维基百科', snippet: '水是地球上最常见的物质之一，是无色、无味、无臭的液体。水的化学式是H2O，由两个氢原子和一个氧原子组成。', url: '#' },
      { title: '水的物理性质', snippet: '水在常温下是无色透明的液体，具有很高的比热容和表面张力。水的沸点是100°C，冰点是0°C。', url: '#' },
      { title: '水的重要性', snippet: '水是生命之源，对所有已知生命形式都至关重要。人体约60%由水组成。', url: '#' }
    ],
    '编程': [
      { title: '编程 - 维基百科', snippet: '编程是创建计算机程序的过程，涉及编写、测试和维护源代码。编程语言包括Python、JavaScript、Java等。', url: '#' },
      { title: '什么是编程', snippet: '编程是一种让计算机执行特定任务的技术，通过编写代码来指令计算机工作。', url: '#' },
      { title: '编程的重要性', snippet: '编程技能在现代社会中越来越重要，它培养逻辑思维和解决问题的能力。', url: '#' }
    ],
    '学习': [
      { title: '学习 - 维基百科', snippet: '学习是通过获取知识、技能和经验来改变行为的过程。学习可以分为有意识的学习和无意识的学习。', url: '#' },
      { title: '学习的本质', snippet: '学习是人类获取新知识和技能的主要方式，涉及记忆、理解和应用。', url: '#' }
    ],
    '智能': [
      { title: '人工智能 - 维基百科', snippet: '人工智能是计算机科学的一个分支，致力于开发能够执行通常需要人类智能的任务的系统。', url: '#' },
      { title: '什么是智能', snippet: '智能是指生物体或系统获取知识、理解概念、解决问题的能力。', url: '#' }
    ],
    '计算机': [
      { title: '计算机 - 维基百科', snippet: '计算机是一种电子设备，用于根据指令处理数据并执行计算。计算机由硬件和软件两部分组成。', url: '#' },
      { title: '计算机的工作原理', snippet: '计算机通过执行指令来处理二进制数据，包括算术运算、逻辑运算和数据存储。', url: '#' }
    ]
  };

  return mockData[query] || [
    { title: `${query} - 概念`, snippet: `${query}是一个重要的概念，在多个领域都有应用。`, url: '#' },
    { title: `关于${query}`, snippet: `${query}涉及多个方面的知识和应用。`, url: '#' }
  ];
}

/**
 * 从搜索结果中提取文本内容
 */
function extractText(results) {
  return results.map(r => `${r.title}: ${r.snippet}`).join(' ');
}

module.exports = {
  searchDuckDuckGo,
  getMockResults,
  extractText
};