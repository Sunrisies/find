use std::fs;
use std::path::Path;

use crate::matcher::Matcher;
use crate::models::{SearchMode, SearchStats};

pub fn search(roots: &[std::path::PathBuf], matcher: &Matcher, mode: SearchMode) -> SearchStats {
    let mut stats = SearchStats::default();
    for root in roots {
        eprintln!("=== 搜索 {} ===", root.display());
        search_one(root, matcher, mode, &mut stats);
    }
    stats
}

/// 在单个根目录下搜索。根路径本身也会参与匹配。
pub fn search_one(root: &Path, matcher: &Matcher, mode: SearchMode, stats: &mut SearchStats) {
    if !root.exists() {
        stats.add_warning(format!("搜索根不存在: {}", root.display()));
        return;
    }

    search_dir(root, matcher, mode, stats);
}

fn search_dir(path: &Path, matcher: &Matcher, mode: SearchMode, stats: &mut SearchStats) {
    if path.is_dir() {
        if matches_kind(path, mode)
            && matcher.is_match(&path.file_name().unwrap_or_default().to_string_lossy())
        {
            println!("{}", path.display());
            stats.matched += 1;
        }

        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(err) => {
                stats.add_warning(format!("无法遍历 {}: {}", path.display(), err));
                return;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    stats.add_warning(format!("无法读取 {}: {}", path.display(), err));
                    continue;
                }
            };

            let file_name = entry.file_name();
            if file_name.to_string_lossy().starts_with('.') {
                continue;
            }

            let path = entry.path();
            if path.is_dir() {
                search_dir(&path, matcher, mode, stats);
            } else if path.is_file() && matcher.is_match(&file_name.to_string_lossy()) {
                println!("{}", path.display());
                stats.matched += 1;
            }
        }
    } else if path.is_file()
        && matcher.is_match(&path.file_name().unwrap_or_default().to_string_lossy())
    {
        println!("{}", path.display());
        stats.matched += 1;
    }
}

fn matches_kind(path: &Path, mode: SearchMode) -> bool {
    match mode {
        SearchMode::Dir => path.is_dir(),
        SearchMode::File => path.is_file(),
        SearchMode::All => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MatchMode;
    use std::path::PathBuf;

    #[test]
    fn search_one_reports_missing_root_as_warning() {
        let root = PathBuf::from("definitely-missing-path-for-find-test");
        let matcher = Matcher::new("x", MatchMode::Exact);
        let mut stats = SearchStats::default();

        search_one(&root, &matcher, SearchMode::All, &mut stats);

        assert_eq!(stats.matched, 0);
        assert_eq!(stats.warnings.len(), 1);
        assert!(stats.warnings[0].contains("搜索根不存在"));
    }

    #[test]
    fn search_one_counts_matching_files() {
        let root = std::env::temp_dir().join(format!("find-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("sub")).unwrap();
        fs::write(root.join("Main.rs"), b"").unwrap();
        fs::write(root.join("sub/main.txt"), b"").unwrap();
        fs::write(root.join(".hidden"), b"").unwrap();

        let matcher = Matcher::new("main", MatchMode::Fuzzy);
        let mut stats = SearchStats::default();

        search_one(&root, &matcher, SearchMode::File, &mut stats);

        let _ = fs::remove_dir_all(&root);
        assert_eq!(stats.matched, 2);
        assert!(stats.warnings.is_empty());
    }
}
