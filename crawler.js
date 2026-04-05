const axios = require('axios');
const cheerio = require('cheerio');

/**
 * 优先使用 Wikipedia API 搜索
 */
async function searchWikipedia(query) {
  const results = [];
  try {
    // Wikipedia REST API
    const response = await axios.get(`https://en.wikipedia.org/api/rest_v1/page/summary/${encodeURIComponent(query)}`, {
      timeout: 8000,
      headers: {
        'User-Agent': 'Seed-Intelligence/1.0 (https://github.com/seed-intelligence)'
      }
    });

    if (response.data) {
      results.push({
        title: response.data.title || query,
        snippet: response.data.extract || '无摘要',
        url: response.data.content_urls?.desktop?.page || `https://en.wikipedia.org/wiki/${encodeURIComponent(query)}`,
        source: 'wikipedia'
      });
    }
  } catch (error) {
    console.log('[Crawler] Wikipedia API failed, trying Chinese Wikipedia...');
  }

  // 如果英文 Wikipedia 失败，尝试中文
  if (results.length === 0) {
    try {
      const response = await axios.get(`https://zh.wikipedia.org/api/rest_v1/page/summary/${encodeURIComponent(query)}`, {
        timeout: 8000,
        headers: {
          'User-Agent': 'Seed-Intelligence/1.0'
        }
      });

      if (response.data) {
        results.push({
          title: response.data.title || query,
          snippet: response.data.extract || '无摘要',
          url: response.data.content_urls?.desktop?.page || `https://zh.wikipedia.org/wiki/${encodeURIComponent(query)}`,
          source: 'wikipedia'
        });
      }
    } catch (e) {
      console.log('[Crawler] Chinese Wikipedia also failed');
    }
  }

  return results;
}

/**
 * 使用 DuckDuckGo HTML 版本搜索 (备选)
 */
async function searchDuckDuckGo(query) {
  const results = [];
  try {
    const response = await axios.get('https://html.duckduckgo.com/html/', {
      params: { q: query },
      headers: {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
      },
      timeout: 10000
    });

    const $ = cheerio.load(response.data);
    $('.result__body').each((i, el) => {
      if (i >= 5) return;
      const title = $(el).find('.result__a').text().trim();
      const snippet = $(el).find('.result__snippet').text().trim();
      const url = $(el).find('.result__a').attr('href');

      if (title && snippet) {
        results.push({ title, snippet, url, source: 'duckduckgo' });
      }
    });
  } catch (error) {
    console.error('DuckDuckGo search error:', error.message);
    return getMockResults(query);
  }

  if (results.length === 0) {
    return getMockResults(query);
  }

  return results;
}

/**
 * 综合搜索：优先 Wikipedia，然后 DuckDuckGo
 */
async function search(query) {
  console.log(`[Crawler] Searching for: ${query}`);

  // 首先尝试 Wikipedia
  let results = await searchWikipedia(query);
  console.log(`[Crawler] Wikipedia results: ${results.length}`);

  // 如果 Wikipedia 没有结果，使用 DuckDuckGo
  if (results.length === 0) {
    results = await searchDuckDuckGo(query);
    console.log(`[Crawler] DuckDuckGo results: ${results.length}`);
  }

  return results;
}

/**
 * 获取模拟搜索结果
 */
function getMockResults(query) {
  const mockData = {
    '水': [
      { title: '水 - 维基百科', snippet: '水是地球上最常见的物质之一，是无色、无味、无臭的液体。水的化学式是H2O，由两个氢原子和一个氧原子组成。', url: '#', source: 'mock' }
    ],
    '编程': [
      { title: '编程 - 维基百科', snippet: '编程是创建计算机程序的过程，涉及编写、测试和维护源代码。', url: '#', source: 'mock' }
    ]
  };

  return mockData[query] || [
    { title: `${query}`, snippet: `${query}是一个概念。`, url: '#', source: 'mock' }
  ];
}

/**
 * 从搜索结果中提取文本
 */
function extractText(results) {
  return results.map(r => `${r.title}: ${r.snippet}`).join(' ');
}

module.exports = {
  search,
  searchWikipedia,
  searchDuckDuckGo,
  getMockResults,
  extractText
};