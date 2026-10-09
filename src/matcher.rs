use crate::models::MatchMode;

/// 匹配器：预先把关键词处理好，避免循环里重复分配。
pub struct Matcher {
    keyword: String,
    keyword_lower: String,
    mode: MatchMode,
}

impl Matcher {
    pub fn new(keyword: &str, mode: MatchMode) -> Self {
        Self {
            keyword: keyword.to_string(),
            keyword_lower: keyword.to_lowercase(),
            mode,
        }
    }

    pub fn is_match(&self, name: &str) -> bool {
        match self.mode {
            MatchMode::Fuzzy => contains_ignore_ascii_case(name, &self.keyword_lower),
            MatchMode::Exact => name.eq_ignore_ascii_case(&self.keyword),
        }
    }
}

/// 字节级忽略 ASCII 大小写的“包含”匹配。
fn contains_ignore_ascii_case(haystack: &str, needle_lower: &str) -> bool {
    if needle_lower.is_empty() {
        return true;
    }
    if needle_lower.len() > haystack.len() {
        return false;
    }
    haystack
        .as_bytes()
        .windows(needle_lower.len())
        .any(|w| w.eq_ignore_ascii_case(needle_lower.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matcher_is_case_insensitive_for_ascii() {
        let matcher = Matcher::new("Main.rs", MatchMode::Exact);
        assert!(matcher.is_match("main.rs"));
        assert!(matcher.is_match("MAIN.RS"));
        assert!(!matcher.is_match("main.txt"));

        let fuzzy = Matcher::new("log", MatchMode::Fuzzy);
        assert!(fuzzy.is_match("app.log"));
        assert!(fuzzy.is_match("APP.LOG"));
        assert!(!fuzzy.is_match("backup.txt"));
    }

    #[test]
    fn fuzzy_match_is_bytewise_for_ascii_case() {
        let needle_lower = "é".to_lowercase();
        assert!(contains_ignore_ascii_case("café", &needle_lower));
        assert!(!contains_ignore_ascii_case("cafe", &needle_lower));
    }
}
