use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceMapEntry {
    pub zig_line: u32,
    pub zig_col: u32,
    pub cp_file: String,
    pub cp_line: u32,
    pub cp_col: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMap {
    pub version: u32,
    pub cp_source: String,
    pub zig_target: String,
    pub mappings: Vec<SourceMapEntry>,
}

impl SourceMap {
    pub fn new(cp_source: String, zig_target: String) -> Self {
        Self {
            version: 1,
            cp_source,
            zig_target,
            mappings: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: SourceMapEntry) {
        self.mappings.push(entry);
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn from_json(json_str: &str) -> anyhow::Result<Self> {
        let map: Self = serde_json::from_str(json_str)?;
        Ok(map)
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        Self::from_json(&content)
    }

    /// Lookup best matching CP source entry for a given Zig line number.
    pub fn lookup(&self, zig_line: u32) -> Option<&SourceMapEntry> {
        if self.mappings.is_empty() {
            return None;
        }
        let mut best: Option<&SourceMapEntry> = None;
        for entry in &self.mappings {
            if entry.zig_line <= zig_line {
                match best {
                    Some(b) if entry.zig_line >= b.zig_line => {
                        best = Some(entry);
                    }
                    None => {
                        best = Some(entry);
                    }
                    _ => {}
                }
            }
        }
        best
    }
}

/// Helper function to demangle a stack trace string using source maps.
pub fn demangle_stack_trace(trace: &str, explicit_map: Option<&SourceMap>) -> String {
    let mut map_cache: BTreeMap<String, Option<SourceMap>> = BTreeMap::new();
    let mut result_lines = Vec::new();
    let lines: Vec<&str> = trace.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        if let Some((prefix, zig_file, zig_line_num, _zig_col_num, rest)) = parse_trace_line(line) {
            let map = if let Some(m) = explicit_map {
                Some(m)
            } else {
                map_cache
                    .entry(zig_file.clone())
                    .or_insert_with(|| find_source_map_for_zig(&zig_file))
                    .as_ref()
            };

            if let Some(sm) = map {
                if let Some(entry) = sm.lookup(zig_line_num) {
                    let cp_file_display = &entry.cp_file;
                    let short_cp = Path::new(cp_file_display)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(cp_file_display);

                    let mut new_rest = rest.clone();
                    let zig_short = Path::new(&zig_file)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&zig_file);
                    new_rest = new_rest.replace(zig_short, short_cp);

                    let demangled_header = format!(
                        "{}{}:{}:{}:{}",
                        prefix, cp_file_display, entry.cp_line, entry.cp_col, new_rest
                    );
                    result_lines.push(demangled_header);

                    // Check if the next line is source code preview and caret
                    let mut handled_preview = false;
                    if i + 1 < lines.len() && !lines[i + 1].contains("0x") {
                        if let Ok(cp_content) = fs::read_to_string(cp_file_display) {
                            let cp_lines: Vec<&str> = cp_content.lines().collect();
                            if entry.cp_line > 0 && (entry.cp_line as usize) <= cp_lines.len() {
                                let code_line = cp_lines[(entry.cp_line - 1) as usize];
                                result_lines.push(format!("    {}", code_line));
                                let col_indent =
                                    " ".repeat(entry.cp_col.saturating_sub(1) as usize + 4);
                                result_lines.push(format!("{}^", col_indent));
                                handled_preview = true;
                                i += 1;
                                if i + 1 < lines.len() && lines[i + 1].trim().starts_with('^') {
                                    i += 1;
                                }
                            }
                        }
                    }

                    if !handled_preview {
                        // Keep going normally
                    }
                    i += 1;
                    continue;
                }
            }
        }

        result_lines.push(line.to_string());
        i += 1;
    }

    result_lines.join("\n")
}

fn parse_trace_line(line: &str) -> Option<(String, String, u32, u32, String)> {
    let colon_indices: Vec<usize> = line.match_indices(':').map(|(idx, _)| idx).collect();
    if colon_indices.len() < 3 {
        return None;
    }

    for (k, &c1) in colon_indices.iter().enumerate() {
        if k + 2 >= colon_indices.len() {
            break;
        }
        let c2 = colon_indices[k + 1];
        let c3 = colon_indices[k + 2];

        let file_part = &line[..c1];
        let line_part = &line[c1 + 1..c2];
        let col_part = &line[c2 + 1..c3];
        let rest_part = &line[c3 + 1..];

        if let (Ok(l_num), Ok(c_num)) = (line_part.parse::<u32>(), col_part.parse::<u32>()) {
            let trimmed_file = file_part.trim();
            if trimmed_file.ends_with(".zig") {
                let prefix_len = file_part.len() - file_part.trim_start().len();
                let prefix = &file_part[..prefix_len];
                return Some((
                    prefix.to_string(),
                    trimmed_file.to_string(),
                    l_num,
                    c_num,
                    rest_part.to_string(),
                ));
            }
        }
    }

    None
}

fn find_source_map_for_zig(zig_path: &str) -> Option<SourceMap> {
    let p = Path::new(zig_path);
    let candidates = vec![
        p.with_extension("zig.map"),
        p.with_extension("map"),
        PathBuf::from(format!("{}.map", zig_path)),
    ];
    for cand in candidates {
        if cand.exists() {
            if let Ok(map) = SourceMap::from_file(&cand) {
                return Some(map);
            }
        }
    }
    None
}
