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
        let mut txt = self.normalize_text(txt);
        txt = self.remove_repetition(&txt);
        txt = self.remove_fillers(&txt);
        txt = self.dedupe_words(&txt);
        self.merge_lines(&txt)
    }

    fn normalize_text(&self, txt: &str) -> String {
        let txt = txt.replace('…', " ");
        let re = Regex::new(r"\.{2,}").expect("Invalid regex in normalize_text");
        re.replace_all(&txt, " ").to_string()
    }

    fn remove_repetition(&self, txt: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = txt.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let mut found = false;
            for len in 1..=4 {
                if i + len * 5 > chars.len() {
                    continue;
                }

                let chunk = &chars[i..i + len];
                let mut count = 1;
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

            if !found {
                result.push(chars[i]);
                i += 1;
            }
        }
        result
    }

    fn remove_fillers(&self, txt: &str) -> String {
        let mut sorted_fillers = crate::domain::constants::TRANSCRIPT_FILLERS.to_vec();
        sorted_fillers.sort_by_key(|a| std::cmp::Reverse(a.len()));

        let pattern_str = sorted_fillers.join("|");
        let pattern = format!(r"(^|[\s、。?!])({})(?=[\s、。?!]||$)", pattern_str);
        let re = Regex::new(&pattern).expect("Invalid regex in remove_fillers");

        let mut current_txt = txt.to_string();
        for _ in 0..20 {
            let prev_txt = current_txt.clone();
            current_txt = re
                .replace_all(&current_txt, |caps: &regex::Captures| {
                    let leading = caps.get(1).map_or("", |m| m.as_str());
                    format!("{} ", leading)
                })
                .to_string();

            if current_txt == prev_txt {
                break;
            }
        }

        let re_space = Regex::new(r"\s+").expect("Invalid regex in remove_fillers space");
        let txt = re_space.replace_all(&current_txt, " ").trim().to_string();

        let mut next_txt = String::new();
        let mut prev_char = None;
        for c in txt.chars() {
            if (c != '、' && c != '。') || Some(c) != prev_char {
                next_txt.push(c);
            }
            prev_char = Some(c);
        }

        let txt = next_txt;
        let re_start_punct =
            Regex::new(r"^[、。]+").expect("Invalid regex in remove_fillers start_punct");
        let txt = re_start_punct.replace_all(&txt, "").trim().to_string();

        let re_space_punct =
            Regex::new(r"\s+[、。]+").expect("Invalid regex in remove_fillers space_punct");
        let txt = re_space_punct.replace_all(&txt, "").to_string();

        re_space.replace_all(&txt, " ").trim().to_string()
    }

    fn dedupe_words(&self, txt: &str) -> String {
        let words: Vec<&str> = txt.split_whitespace().collect();
        let mut result = Vec::new();
        let mut i = 0;
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
        let txt = txt.replace('\n', " ");
        let re = Regex::new(r"\s+").expect("Invalid regex in merge_lines");
        re.replace_all(&txt, " ").trim().to_string()
    }
}
