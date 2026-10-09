use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SearchMode {
    Dir,
    File,
    All,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MatchMode {
    Fuzzy,
    Exact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedArgs {
    pub mode: SearchMode,
    pub match_mode: MatchMode,
    pub keyword: String,
    pub roots: Vec<PathBuf>,
}

impl ParsedArgs {
    pub fn new(
        mode: SearchMode,
        match_mode: MatchMode,
        keyword: String,
        roots: Vec<PathBuf>,
    ) -> Self {
        Self {
            mode,
            match_mode,
            keyword,
            roots,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CommandKind {
    Run,
    Help,
    Version,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SearchStats {
    pub matched: u64,
    pub warnings: Vec<String>,
}

impl SearchStats {
    pub fn add_warning(&mut self, warning: String) {
        if self.warnings.len() < 10 {
            self.warnings.push(warning);
        }
    }
}
