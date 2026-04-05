// NLP Module - Tokenization, Stop Words, Word Frequency

use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;

lazy_static! {
    // Common Chinese words dictionary
    static ref COMMON_CHINESE: HashSet<&'static str> = {
        let mut s = HashSet::new();
        // Common nouns
        s.insert("苹果"); s.insert("水果"); s.insert("手机"); s.insert("电脑");
        s.insert("网络"); s.insert("软件"); s.insert("硬件"); s.insert("系统");
        s.insert("数据"); s.insert("信息"); s.insert("技术"); s.insert("科学");
        s.insert("学习"); s.insert("教育"); s.insert("学校"); s.insert("学生");
        s.insert("老师"); s.insert("工作"); s.insert("公司"); s.insert("产品");
        s.insert("服务"); s.insert("用户"); s.insert("客户"); s.insert("市场");
        s.insert("价格"); s.insert("质量"); s.insert("时间"); s.insert("空间");
        s.insert("世界"); s.insert("中国"); s.insert("美国"); s.insert("日本");
        s.insert("欧洲"); s.insert("亚洲"); s.insert("国际"); s.insert("政治");
        s.insert("经济"); s.insert("文化"); s.insert("历史"); s.insert("社会");
        s.insert("生命"); s.insert("水"); s.insert("空气"); s.insert("光");
        s.insert("热"); s.insert("温度"); s.insert("能量"); s.insert("物质");
        s.insert("分子"); s.insert("原子");
        // Common verbs
        s.insert("学习"); s.insert("工作"); s.insert("生活"); s.insert("使用");
        s.insert("开发"); s.insert("设计"); s.insert("创造"); s.insert("管理");
        s.insert("组织"); s.insert("计划"); s.insert("开始"); s.insert("结束");
        s.insert("进行"); s.insert("完成"); s.insert("发展"); s.insert("变化");
        s.insert("增长"); s.insert("减少"); s.insert("提高"); s.insert("降低");
        // Common adjectives
        s.insert("重要"); s.insert("简单"); s.insert("复杂"); s.insert("困难");
        s.insert("容易"); s.insert("快速"); s.insert("慢速"); s.insert("高效");
        s.insert("低效"); s.insert("现代"); s.insert("传统");
        // Question words
        s.insert("什么"); s.insert("怎么"); s.insert("如何"); s.insert("为什么");
        s
    };

    // Stop words (Chinese + English)
    static ref STOP_WORDS: HashSet<&'static str> = {
        let mut s = HashSet::new();
        // Chinese stop words
        s.insert("的"); s.insert("是"); s.insert("在"); s.insert("了"); s.insert("和");
        s.insert("与"); s.insert("或"); s.insert("有"); s.insert("这"); s.insert("那");
        s.insert("个"); s.insert("一"); s.insert("不"); s.insert("也"); s.insert("都");
        s.insert("就"); s.insert("而"); s.insert("及"); s.insert("以"); s.insert("对");
        s.insert("可"); s.insert("能"); s.insert("会"); s.insert("被"); s.insert("于");
        s.insert("从"); s.insert("到"); s.insert("把"); s.insert("将"); s.insert("为");
        s.insert("但"); s.insert("却"); s.insert("又"); s.insert("如"); s.insert("因");
        s.insert("所"); s.insert("并"); s.insert("其"); s.insert("之"); s.insert("来");
        s.insert("去"); s.insert("上"); s.insert("下"); s.insert("中"); s.insert("大");
        s.insert("小"); s.insert("多"); s.insert("少"); s.insert("最"); s.insert("更");
        s.insert("很"); s.insert("太"); s.insert("过"); s.insert("要"); s.insert("该");
        s.insert("我们"); s.insert("你们"); s.insert("他们"); s.insert("她们");
        s.insert("它们"); s.insert("这个"); s.insert("那个"); s.insert("可以");
        s.insert("没有"); s.insert("这样"); s.insert("那样"); s.insert("自己");
        s.insert("已经"); s.insert("因为"); s.insert("所以"); s.insert("但是");
        s.insert("而且"); s.insert("或者"); s.insert("如果"); s.insert("虽然");
        s.insert("只是"); s.insert("就是"); s.insert("还是"); s.insert("应该");
        s.insert("需要"); s.insert("可能"); s.insert("关于");
        // English stop words
        s.insert("the"); s.insert("a"); s.insert("an"); s.insert("is"); s.insert("are");
        s.insert("was"); s.insert("were"); s.insert("be"); s.insert("been"); s.insert("being");
        s.insert("have"); s.insert("has"); s.insert("had"); s.insert("do"); s.insert("does");
        s.insert("did"); s.insert("will"); s.insert("would"); s.insert("could"); s.insert("should");
        s.insert("may"); s.insert("might"); s.insert("must"); s.insert("shall"); s.insert("can");
        s.insert("need"); s.insert("dare"); s.insert("ought"); s.insert("used"); s.insert("to");
        s.insert("of"); s.insert("in"); s.insert("for"); s.insert("on"); s.insert("with");
        s.insert("at"); s.insert("by"); s.insert("from"); s.insert("as"); s.insert("into");
        s.insert("through"); s.insert("during"); s.insert("before"); s.insert("after");
        s.insert("above"); s.insert("below"); s.insert("between"); s.insert("under");
        s.insert("again"); s.insert("further"); s.insert("then"); s.insert("once");
        s.insert("and"); s.insert("but"); s.insert("or"); s.insert("nor"); s.insert("so");
        s.insert("yet"); s.insert("both"); s.insert("either"); s.insert("neither");
        s.insert("not"); s.insert("only"); s.insert("own"); s.insert("same"); s.insert("than");
        s.insert("too"); s.insert("very"); s.insert("just"); s.insert("also"); s.insert("now");
        s.insert("here"); s.insert("there"); s.insert("when"); s.insert("where"); s.insert("why");
        s.insert("how"); s.insert("all"); s.insert("each"); s.insert("every"); s.insert("few");
        s.insert("more"); s.insert("most"); s.insert("other"); s.insert("some"); s.insert("such");
        s.insert("no"); s.insert("any"); s.insert("what"); s.insert("which"); s.insert("who");
        s.insert("whom"); s.insert("this"); s.insert("that"); s.insert("these"); s.insert("those");
        s.insert("it"); s.insert("its"); s.insert("i"); s.insert("me"); s.insert("my");
        s.insert("we"); s.insert("our"); s.insert("you"); s.insert("your"); s.insert("he");
        s.insert("she"); s.insert("him"); s.insert("her"); s.insert("his");
        s
    };
}

/// Tokenize text (Chinese + English mixed)
pub fn tokenize(text: &str) -> Vec<String> {
    if text.is_empty() {
        return vec![];
    }

    let mut tokens = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // 1. Extract English words (2+ letters)
    if let Ok(re) = Regex::new(r"[a-zA-Z]{2,}") {
        for cap in re.find_iter(text) {
            let word = cap.as_str().to_lowercase();
            if !STOP_WORDS.contains(word.as_str()) && !seen.contains(&word) && word.len() >= 2 {
                seen.insert(word.clone());
                tokens.push(word);
            }
        }
    }

    // 2. Extract Chinese 2-gram
    if let Ok(re) = Regex::new(r"[\u4e00-\u9fff]{2}") {
        for cap in re.find_iter(text) {
            let word = cap.as_str();
            if !STOP_WORDS.contains(word) && !seen.contains(word) {
                seen.insert(word.to_string());
                tokens.push(word.to_string());
            }
        }
    }

    // 3. Extract Chinese 3-gram (keywords)
    if let Ok(re) = Regex::new(r"[\u4e00-\u9fff]{3}") {
        for cap in re.find_iter(text) {
            let word = cap.as_str();
            // Skip if contains function words
            if word.contains('的') || word.contains('了') || word.contains('是') {
                continue;
            }
            if !STOP_WORDS.contains(word) && !seen.contains(word) {
                seen.insert(word.to_string());
                tokens.push(word.to_string());
            }
        }
    }

    // 4. Extract Chinese 4-gram (common words)
    if let Ok(re) = Regex::new(r"[\u4e00-\u9fff]{4}") {
        for cap in re.find_iter(text) {
            let word = cap.as_str();
            if COMMON_CHINESE.contains(word) && !seen.contains(word) {
                seen.insert(word.to_string());
                tokens.push(word.to_string());
            }
        }
    }

    tokens
}

/// Filter stop words from tokens
pub fn filter_stop_words(tokens: &[String]) -> Vec<String> {
    tokens
        .iter()
        .filter(|t| t.len() >= 2 && !STOP_WORDS.contains(t.as_str()))
        .cloned()
        .collect()
}

/// Count word frequency
pub fn count_frequency(texts: &[String], top_n: usize) -> Vec<(String, u32)> {
    let mut freq = std::collections::HashMap::new();

    for text in texts {
        let tokens = tokenize(text);
        for token in tokens {
            *freq.entry(token).or_insert(0) += 1;
        }
    }

    let mut items: Vec<_> = freq.into_iter().collect();
    items.sort_by(|a, b| b.1.cmp(&a.1));
    items.truncate(top_n);
    items
}

/// Extract co-occurrences with target concept
pub fn extract_cooccurrences(target: &str, texts: &[String], min_freq: u32) -> Vec<(String, u32)> {
    let mut cooc = std::collections::HashMap::new();

    for text in texts {
        let tokens = tokenize(text);
        let has_target = tokens.iter().any(|t| t.contains(target) || target.contains(t));

        if has_target {
            for token in &tokens {
                if !token.contains(target) && !target.contains(token) {
                    *cooc.entry(token.clone()).or_insert(0) += 1;
                }
            }
        }
    }

    let mut items: Vec<_> = cooc.into_iter().filter(|(_, v)| *v >= min_freq).collect();
    items.sort_by(|a, b| b.1.cmp(&a.1));
    items
}

/// Extract keywords by combining frequency and co-occurrence
pub fn extract_keywords(texts: &[String], target: &str, top_n: usize) -> Vec<(String, f64)> {
    let word_freq = count_frequency(texts, top_n * 2);
    let cooc = extract_cooccurrences(target, texts, 1);

    let mut keywords = std::collections::HashMap::new();
    let all_words: std::collections::HashSet<_> = word_freq.iter()
        .map(|(w, _)| w)
        .chain(cooc.iter().map(|(w, _)| w))
        .collect();

    for word in all_words {
        let freq_score = word_freq.iter().find(|(w, _)| w == word).map(|(_, v)| *v).unwrap_or(0) as f64;
        let cooc_score = cooc.iter().find(|(w, _)| w == word).map(|(_, v)| *v).unwrap_or(0) as f64;
        keywords.insert(word.clone(), freq_score * 0.4 + cooc_score * 0.6);
    }

    let mut items: Vec<_> = keywords.into_iter().collect();
    items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    items.truncate(top_n);
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let text = "水是地球上最常见的物质之一。水的化学式是H2O。";
        let tokens = tokenize(text);
        println!("{:?}", tokens);
        assert!(tokens.len() > 0);
    }

    #[test]
    fn test_filter_stop_words() {
        let tokens = vec!["水".to_string(), "的".to_string(), "是".to_string(), "重要".to_string()];
        let filtered = filter_stop_words(&tokens);
        assert!(filtered.contains(&"水".to_string()));
        assert!(!filtered.contains(&"的".to_string()));
    }
}