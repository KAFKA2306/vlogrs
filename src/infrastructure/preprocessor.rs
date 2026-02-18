use regex::Regex;

pub struct TranscriptPreprocessor;

impl Default for TranscriptPreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl TranscriptPreprocessor {
    pub fn new() -> Self {
        Self
    }

    pub fn process(&self, txt: &str) -> String {
        let mut txt: String = self.normalize_text(txt);
        txt = self.remove_repetition(&txt);
        txt = self.remove_fillers(&txt);
        txt = self.dedupe_words(&txt);
        txt = self.merge_lines(&txt);
        txt
    }

    fn normalize_text(&self, txt: &str) -> String {
        let txt: String = txt.replace("…", " ");
        let re: Regex = Regex::new(r"\.{2,}").unwrap();
        re.replace_all(&txt, " ").to_string()
    }

    fn remove_repetition(&self, txt: &str) -> String {
        let mut result: String = String::new();
        let chars: Vec<char> = txt.chars().collect();
        let mut i: usize = 0;
        while i < chars.len() {
            let mut found: bool = false;
            for len in 1..=4 {
                if i + len * 5 <= chars.len() {
                    let chunk: &[char] = &chars[i..i + len];
                    let mut count: usize = 1;
                    while i + (count + 1) * len <= chars.len()
                        && &chars[i + count * len..i + (count + 1) * len] == chunk
                    {
                        count += 1;
                    }
                    if count >= 5 {
                        result.extend(chunk);
                        i += count * len;
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                result.push(chars[i]);
                i += 1;
            }
        }
        result
    }

    fn remove_fillers(&self, txt: &str) -> String {
        let fillers: [&str; 37] = [
            "えー",
            "あのー",
            "うーん",
            "えっと",
            "なんて",
            "まあ",
            "そうですね",
            "あー",
            "んー",
            "うん",
            "ふん",
            "あ",
            "はは",
            "ははは",
            "なんか",
            "え",
            "お",
            "ふんふん",
            "ふんふんふん",
            "うんうん",
            "うんうんうん",
            "はいはい",
            "はいはいはい",
            "はいはいはいはい",
            "おー",
            "ああ",
            "んふん",
            "そっか",
            "そっかぁ",
            "そうか",
            "そうなんだ",
            "えへへ",
            "あの",
            "あのね",
            "あのさ",
            "ん",
            "えっと",
        ];

        let mut sorted_fillers: Vec<&str> = fillers.to_vec();
        sorted_fillers.sort_by_key(|a: &&str| std::cmp::Reverse(a.len()));

        let pattern_str: String = sorted_fillers.join("|");
        let pattern: String = format!(r"(^|[\s、。?!])({})(?=[\s、。?!]||$)", pattern_str);
        let re: Regex = Regex::new(&pattern).unwrap();

        let mut current_txt: String = txt.to_string();
        for _ in 0..20 {
            let prev_txt: String = current_txt.clone();
            current_txt = re
                .replace_all(&current_txt, |caps: &regex::Captures| {
                    let leading: &str = caps.get(1).map_or("", |m: regex::Match| m.as_str());
                    format!("{} ", leading)
                })
                .to_string();

            if current_txt == prev_txt {
                break;
            }
        }

        let re_space: Regex = Regex::new(r"\s+").unwrap();
        let mut txt: String = re_space.replace_all(&current_txt, " ").trim().to_string();

        let mut next_txt: String = String::new();
        let mut prev_char: Option<char> = None;
        for c in txt.chars() {
            if c == '、' || c == '。' {
                if Some(c) != prev_char {
                    next_txt.push(c);
                }
            } else {
                next_txt.push(c);
            }
            prev_char = Some(c);
        }
        txt = next_txt;

        let re_start_punct: Regex = Regex::new(r"^[、。]+").unwrap();
        txt = re_start_punct.replace_all(&txt, "").trim().to_string();

        let re_space_punct: Regex = Regex::new(r"\s+[、。]+").unwrap();
        txt = re_space_punct.replace_all(&txt, "").to_string();

        re_space.replace_all(&txt, " ").trim().to_string()
    }

    fn dedupe_words(&self, txt: &str) -> String {
        let words: Vec<&str> = txt.split_whitespace().collect();
        let mut result: Vec<&str> = Vec::new();
        let mut i: usize = 0;
        while i < words.len() {
            result.push(words[i]);
            if i + 1 < words.len() && words[i] == words[i + 1] {
                i += 2;
            } else {
                i += 1;
            }
        }
        result.join(" ")
    }

    fn merge_lines(&self, txt: &str) -> String {
        let txt: String = txt.replace('\n', " ");
        let re: Regex = Regex::new(r"\s+").unwrap();
        re.replace_all(&txt, " ").trim().to_string()
    }
}
