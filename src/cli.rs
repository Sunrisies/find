use std::env;
use std::path::{Path, PathBuf};

use crate::models::{CommandKind, MatchMode, ParsedArgs, SearchMode};

/// 不填搜索根时默认搜索当前目录。
pub fn current_dir_root() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// 把盘符补成 Windows 可用根路径，例如 `C:` -> `C:\`。
fn normalize_drive(s: &str) -> PathBuf {
    let s = s.trim();
    if is_drive(s) && s.len() == 2 && s.ends_with(':') {
        PathBuf::from(format!("{}\\", s))
    } else {
        PathBuf::from(s)
    }
}

/// 判断一个参数是不是盘符形式：`C:`、`C:\`、`C:/`。
fn is_drive(s: &str) -> bool {
    let b = s.as_bytes();
    match b.len() {
        2 => b[0].is_ascii_alphabetic() && b[1] == b':',
        3 => b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/'),
        _ => false,
    }
}

/// 搜索根可以是盘符，也可以是已存在的目录；文件本身不作为根。
fn is_search_root(arg: &str) -> bool {
    is_drive(arg) || Path::new(arg).is_dir()
}

/// 判断命令行参数是否应被当前阶段视为搜索标志。
fn should_parse_as_flag(arg: &str, keyword_seen: bool, after_double_dash: bool) -> bool {
    !after_double_dash && !keyword_seen && arg.starts_with('-') && arg.len() > 1
}

/// 同一个维度（类型 or 匹配）只能出现一次。
fn set_once<T>(slot: &mut Option<T>, value: T, dim: &str, c: char) -> Result<(), String> {
    if slot.is_some() {
        return Err(format!("{}字符重复: `{}`", dim, c));
    }
    *slot = Some(value);
    Ok(())
}

/// 解析缩写标志，如 `-df`、`-ef`、`-a`。
pub fn parse_flag(flag: &str) -> Result<(SearchMode, MatchMode), String> {
    let flag = flag.trim_start_matches('-');
    if flag.is_empty() {
        return Err("标志为空".to_string());
    }
    if flag.len() > 2 {
        return Err(format!("标志过长: `-{}`（最多两个字符）", flag));
    }

    let mut mode: Option<SearchMode> = None;
    let mut match_mode: Option<MatchMode> = None;

    for c in flag.chars() {
        match c.to_ascii_lowercase() {
            'd' => set_once(&mut mode, SearchMode::Dir, "类型", 'd')?,
            'f' => set_once(&mut mode, SearchMode::File, "类型", 'f')?,
            'a' => set_once(&mut mode, SearchMode::All, "类型", 'a')?,
            'z' => set_once(&mut match_mode, MatchMode::Fuzzy, "匹配", 'z')?,
            'e' => set_once(&mut match_mode, MatchMode::Exact, "匹配", 'e')?,
            c => return Err(format!("未知标志字符: `{}`（类型: d/f/a，匹配: z/e）", c)),
        }
    }

    Ok((
        mode.unwrap_or(SearchMode::All),
        match_mode.unwrap_or(MatchMode::Fuzzy),
    ))
}

/// 解析命令行参数。
///
/// 支持任意顺序、任意多个搜索根：
///   - `find <关键词> [根...]`
///   - `find -e <关键词> [根...]`
///   - `find <根> <关键词>`
///   - `find -- <关键词> [根...]`
pub fn parse_args(args: &[String]) -> Result<ParsedArgs, String> {
    let mut mode: Option<SearchMode> = None;
    let mut match_mode: Option<MatchMode> = None;
    let mut keyword: Option<String> = None;
    let mut roots: Vec<PathBuf> = Vec::new();
    let mut after_double_dash = false;

    for arg in &args[1..] {
        if !after_double_dash && arg == "--" {
            after_double_dash = true;
            continue;
        }

        if keyword.is_some() {
            // 关键词后的普通参数一律视为搜索根，包括以 `-` 开头的目录名。
            roots.push(normalize_drive(arg));
            continue;
        }

        if !after_double_dash && is_search_root(arg) && roots.is_empty() {
            roots.push(normalize_drive(arg));
            continue;
        }

        if should_parse_as_flag(arg, keyword.is_some(), after_double_dash) {
            if mode.is_some() || match_mode.is_some() {
                return Err("标志只能出现一次".to_string());
            }
            let (m, mm) = parse_flag(arg)?;
            mode = Some(m);
            match_mode = Some(mm);
            continue;
        }

        if let Some(existing) = &keyword {
            return Err(format!(
                "出现了多个关键词: `{}` 和 `{}`（使用 `--` 可把关键词放在最前面）",
                existing, arg
            ));
        }
        keyword = Some(arg.clone());
    }

    let keyword = keyword.ok_or_else(|| "缺少关键词".to_string())?;
    if keyword.is_empty() {
        return Err("关键词不能为空".to_string());
    }

    Ok(ParsedArgs::new(
        mode.unwrap_or(SearchMode::All),
        match_mode.unwrap_or(MatchMode::Fuzzy),
        keyword,
        if roots.is_empty() {
            vec![current_dir_root()]
        } else {
            roots
        },
    ))
}

pub fn print_help(prog: &str) {
    println!("用法: {} [标志] <关键词> [搜索根...]", prog);
    println!();
    println!("  标志（可省略，省略时为全部类型 + 模糊匹配）：");
    println!("    类型:  d 目录   f 文件   a 全部");
    println!("    匹配:  z 模糊   e 精确（省略默认模糊）");
    println!("    两字符顺序无关，如 -dz = -zd，-fe = -ef");
    println!();
    println!("  示例标志:");
    println!("    -d  -dz   目录 模糊");
    println!("    -de -ed   目录 精确");
    println!("    -f  -fz   文件 模糊");
    println!("    -fe -ef   文件 精确");
    println!("    -a  -az   全部 模糊");
    println!("    -ae -ea   全部 精确");
    println!();
    println!("  搜索根：不填默认搜索当前目录；可以是目录、盘符或已存在路径。");
    println!("  参数顺序任意。关键词带 '-' 时，用 `--` 放最前面：");
    println!("    {} -- -weird-name", prog);
    println!();
    println!("  默认跳过隐藏文件/目录；精确匹配会忽略 ASCII 大小写。");
    println!();
    println!("示例:");
    println!("  {} main.rs", prog);
    println!("  {} -d node_modules", prog);
    println!("  {} -ef C: main.rs", prog);
    println!("  {} config -dz D:", prog);
    println!("  {} -az log C: D: E:", prog);
    println!("  {} src Cargo.toml", prog);
    println!("  {} -- -my-file.txt", prog);
}

pub fn command_kind(args: &[String]) -> CommandKind {
    for arg in &args[1..] {
        if arg == "--" {
            return CommandKind::Run;
        }
        if arg == "--help" || arg == "-h" {
            return CommandKind::Help;
        }
        if arg == "--version" || arg == "-V" {
            return CommandKind::Version;
        }
    }
    CommandKind::Run
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_flag_accepts_type_and_match_combos() {
        assert_eq!(
            parse_flag("-az").unwrap(),
            (SearchMode::All, MatchMode::Fuzzy)
        );
        assert_eq!(
            parse_flag("-fe").unwrap(),
            (SearchMode::File, MatchMode::Exact)
        );
        assert_eq!(
            parse_flag("-ea").unwrap(),
            (SearchMode::All, MatchMode::Exact)
        );
        assert_eq!(
            parse_flag("-a").unwrap(),
            (SearchMode::All, MatchMode::Fuzzy)
        );
    }

    #[test]
    fn parse_flag_rejects_unknown_and_duplicate_chars() {
        assert!(parse_flag("-x").is_err());
        assert!(parse_flag("-dd").is_err());
        assert!(parse_flag("-zz").is_err());
        assert!(parse_flag("-ade").is_err());
    }

    #[test]
    fn parse_args_default_root_is_current_dir() {
        let args = vec!["find".to_string(), "keyword".to_string()];
        let parsed = parse_args(&args).unwrap();

        assert_eq!(parsed.keyword, "keyword");
        assert_eq!(parsed.roots, vec![current_dir_root()]);
        assert_eq!(parsed.mode, SearchMode::All);
        assert_eq!(parsed.match_mode, MatchMode::Fuzzy);
    }

    #[test]
    fn parse_args_supports_arbitrary_order_and_extra_roots() {
        let current = current_dir_root();
        let args = vec![
            "find".to_string(),
            "keyword".to_string(),
            current.to_string_lossy().to_string(),
        ];

        let parsed = parse_args(&args).unwrap();

        assert_eq!(parsed.keyword, "keyword");
        assert_eq!(parsed.roots, vec![current]);
    }

    #[test]
    fn parse_args_treats_existing_dir_before_keyword_as_root() {
        let current = current_dir_root();
        let args = vec![
            "find".to_string(),
            current.to_string_lossy().to_string(),
            "keyword".to_string(),
        ];

        let parsed = parse_args(&args).unwrap();

        assert_eq!(parsed.keyword, "keyword");
        assert_eq!(parsed.roots, vec![current]);
    }

    #[test]
    fn parse_args_accepts_multiple_roots_after_keyword() {
        let first_root = current_dir_root();
        let args = vec![
            "find".to_string(),
            "one".to_string(),
            first_root.to_string_lossy().to_string(),
        ];

        let parsed = parse_args(&args).unwrap();

        assert_eq!(parsed.keyword, "one");
        assert_eq!(parsed.roots, vec![first_root]);
    }

    #[test]
    fn parse_args_supports_double_dash_before_keyword() {
        let args = vec!["find".to_string(), "--".to_string(), "-my-file".to_string()];

        let parsed = parse_args(&args).unwrap();

        assert_eq!(parsed.keyword, "-my-file");
    }

    #[test]
    fn command_kind_supports_help_and_version() {
        assert_eq!(
            command_kind(&["find".to_string(), "--help".to_string()]),
            CommandKind::Help
        );
        assert_eq!(
            command_kind(&["find".to_string(), "--version".to_string()]),
            CommandKind::Version
        );
    }

    #[test]
    fn parse_args_rejects_empty_keyword() {
        let err = parse_args(&["find".to_string(), "".to_string()]).unwrap_err();
        assert!(err.contains("关键词不能为空"), "unexpected error: {err}");
    }

    #[test]
    fn parse_args_rejects_duplicate_flags() {
        let args = vec![
            "find".to_string(),
            "-a".to_string(),
            "-f".to_string(),
            "keyword".to_string(),
        ];
        assert!(parse_args(&args).unwrap_err().contains("标志只能出现一次"));
    }
}
