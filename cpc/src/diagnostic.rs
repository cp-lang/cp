use crate::Span;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiagnosticEdit {
    pub offset: usize,
    pub delete_len: usize,
    pub insert: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiagnosticJson {
    pub code: String,
    pub severity: String, // "error" | "warning"
    pub message: String,
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub span_start: usize,
    pub span_end: usize,
    pub cause: Option<String>,
    pub help: Option<String>,
    pub suggestion: Option<String>,
    pub edits: Vec<DiagnosticEdit>,
}

impl DiagnosticJson {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiagnosticReport {
    pub status: String,
    pub diagnostics: Vec<DiagnosticJson>,
}

impl DiagnosticReport {
    pub fn ok() -> Self {
        Self {
            status: "ok".to_string(),
            diagnostics: Vec::new(),
        }
    }

    pub fn ok_json() -> String {
        serde_json::to_string(&Self::ok())
            .unwrap_or_else(|_| "{\"status\":\"ok\",\"diagnostics\":[]}".to_string())
    }

    pub fn error(diag: DiagnosticJson) -> Self {
        Self {
            status: "error".to_string(),
            diagnostics: vec![diag],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub span: Option<Span>,
    pub cause: Option<String>,
    pub help: Option<String>,
    pub suggestion: Option<String>,
}

impl Diagnostic {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            span: None,
            cause: None,
            help: None,
            suggestion: None,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_cause(mut self, cause: impl Into<String>) -> Self {
        self.cause = Some(cause.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn render(&self, source_code: Option<&str>, file_path: Option<&Path>) -> String {
        let mut out = String::new();
        out.push_str(&format!("error[{}]: {}\n", self.code, self.message));

        let file_display = file_path
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<input>".to_string());

        if let (Some(code), Some(span)) = (source_code, self.span) {
            let (line_idx, col_idx, line_str) = find_line_and_col(code, span.start);
            let line_num = line_idx + 1;
            let col_num = col_idx + 1;

            out.push_str(&format!(
                "  --> {}:{}:{}\n",
                file_display, line_num, col_num
            ));
            out.push_str("   |\n");

            let line_prefix = format!("{:>4} | ", line_num);
            out.push_str(&line_prefix);
            out.push_str(line_str);
            if !line_str.ends_with('\n') {
                out.push('\n');
            }

            // Calculate underline length
            let underline_len = if span.end > span.start {
                (span.end - span.start)
                    .min(line_str.len().saturating_sub(col_idx))
                    .max(1)
            } else {
                1
            };

            let indent = " ".repeat(col_idx);
            let carets = "^".repeat(underline_len);
            out.push_str(&format!("   | {}{}\n", indent, carets));
            out.push_str("   |\n");
        } else if let Some(span) = self.span {
            out.push_str(&format!(
                "  --> {}: (span: {}..{})\n",
                file_display, span.start, span.end
            ));
        }

        if let Some(cause) = &self.cause {
            out.push_str(&format!("   = cause: {}\n", cause));
        }
        if let Some(help) = &self.help {
            out.push_str(&format!("   = help: {}\n", help));
        }
        if let Some(suggestion) = &self.suggestion {
            out.push_str(&format!("   = suggestion: {}\n", suggestion));
        }

        out
    }

    pub fn to_diagnostic_json(
        &self,
        source_code: Option<&str>,
        file_path: Option<&Path>,
    ) -> DiagnosticJson {
        let file = file_path
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<input>".to_string());

        let (line, col, span_start, span_end) = if let Some(span) = self.span {
            let (l, c) = if let Some(code) = source_code {
                let (line_idx, col_idx, _) = find_line_and_col(code, span.start);
                (line_idx + 1, col_idx + 1)
            } else {
                (1, 1)
            };
            (l, c, span.start, span.end)
        } else {
            (1, 1, 0, 0)
        };

        let edits = self.generate_edits(source_code);

        DiagnosticJson {
            code: self.code.clone(),
            severity: "error".to_string(),
            message: self.message.clone(),
            file,
            line,
            col,
            span_start,
            span_end,
            cause: self.cause.clone(),
            help: self.help.clone(),
            suggestion: self.suggestion.clone(),
            edits,
        }
    }

    pub fn to_json(&self, source_code: Option<&str>, file_path: Option<&Path>) -> String {
        let diag_json = self.to_diagnostic_json(source_code, file_path);
        let report = DiagnosticReport::error(diag_json);
        serde_json::to_string(&report).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn generate_edits(&self, source_code: Option<&str>) -> Vec<DiagnosticEdit> {
        let mut edits = Vec::new();

        // 1. Missing parentheses in control flow statements (E0101)
        if self.code == "E0101"
            || self
                .suggestion
                .as_ref()
                .map_or(false, |s| s.contains("parentheses"))
            || self.message.contains("parentheses")
        {
            if let Some(span) = self.span {
                if let Some(code) = source_code {
                    let (start, end) = find_condition_range(code, span.start);
                    edits.push(DiagnosticEdit {
                        offset: start,
                        delete_len: 0,
                        insert: "(".to_string(),
                    });
                    edits.push(DiagnosticEdit {
                        offset: end,
                        delete_len: 0,
                        insert: ")".to_string(),
                    });
                } else {
                    edits.push(DiagnosticEdit {
                        offset: span.start,
                        delete_len: 0,
                        insert: "(".to_string(),
                    });
                    edits.push(DiagnosticEdit {
                        offset: span.end,
                        delete_len: 0,
                        insert: ")".to_string(),
                    });
                }
                return edits;
            }
        }

        // 2. Abolished 'interface' keyword (E0102) -> replace with 'trait'
        if self.code == "E0102"
            || self
                .suggestion
                .as_ref()
                .map_or(false, |s| s.contains("replace 'interface' with 'trait'"))
        {
            if let Some(span) = self.span {
                edits.push(DiagnosticEdit {
                    offset: span.start,
                    delete_len: span.end.saturating_sub(span.start),
                    insert: "trait".to_string(),
                });
                return edits;
            }
        }

        // 3. Abolished 'mut' in var decl (E0104) -> remove 'mut'
        if self.code == "E0104"
            || self
                .suggestion
                .as_ref()
                .map_or(false, |s| s.contains("remove 'mut'"))
        {
            if let Some(span) = self.span {
                let del_len = if let Some(code) = source_code {
                    if code.as_bytes().get(span.end) == Some(&b' ') {
                        span.end - span.start + 1
                    } else {
                        span.end - span.start
                    }
                } else {
                    span.end - span.start
                };
                edits.push(DiagnosticEdit {
                    offset: span.start,
                    delete_len: del_len,
                    insert: String::new(),
                });
                return edits;
            }
        }

        // 4. Parameter read-only borrow (E0201) -> add '*' for mutable borrow
        if self.code == "E0201"
            && self
                .suggestion
                .as_ref()
                .map_or(false, |s| s.contains("Add '*'"))
        {
            if let Some(span) = self.span {
                edits.push(DiagnosticEdit {
                    offset: span.start,
                    delete_len: 0,
                    insert: "*".to_string(),
                });
                return edits;
            }
        }

        // 5. Const reassignment (E0204) -> change 'const' to 'let'
        if self.code == "E0204" {
            if let (Some(code), Some(target)) = (source_code, self.message.split('\'').nth(1)) {
                let pattern = format!("const {}", target);
                if let Some(pos) = code.find(&pattern) {
                    edits.push(DiagnosticEdit {
                        offset: pos,
                        delete_len: 5,
                        insert: "let".to_string(),
                    });
                    return edits;
                }
            }
        }

        // 6. Undefined symbol with standard library import suggestion (E0206)
        if self.code == "E0206" {
            if let Some(sug) = &self.suggestion {
                if sug.starts_with("import { ") {
                    edits.push(DiagnosticEdit {
                        offset: 0,
                        delete_len: 0,
                        insert: format!("{}\n", sug),
                    });
                    return edits;
                }
            }
        }

        edits
    }
}

pub fn find_condition_range(code: &str, span_start: usize) -> (usize, usize) {
    let start = span_start.min(code.len());
    let rest = &code[start..];

    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut in_str = None;
    let mut escape = false;
    let mut end_offset = None;

    for (rel_idx, ch) in rest.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        if let Some(quote) = in_str {
            if ch == quote {
                in_str = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' || ch == '`' {
            in_str = Some(ch);
            continue;
        }

        match ch {
            '(' => paren_depth += 1,
            ')' => {
                if paren_depth > 0 {
                    paren_depth -= 1;
                }
            }
            '[' => bracket_depth += 1,
            ']' => {
                if bracket_depth > 0 {
                    bracket_depth -= 1;
                }
            }
            '{' => {
                if paren_depth == 0 && bracket_depth == 0 {
                    end_offset = Some(start + rel_idx);
                    break;
                }
            }
            '\n' | ';' => {
                if paren_depth == 0 && bracket_depth == 0 {
                    end_offset = Some(start + rel_idx);
                    break;
                }
            }
            _ => {}
        }
    }

    let raw_end = end_offset.unwrap_or(code.len());
    let cond_str = &code[start..raw_end];
    let trimmed_len = cond_str.trim_end().len();
    let cond_end = start + trimmed_len;

    (start, cond_end.max(start))
}

pub fn find_line_and_col(code: &str, offset: usize) -> (usize, usize, &str) {
    let offset = offset.min(code.len());
    let before = &code[..offset];
    let line_idx = before.chars().filter(|&c| c == '\n').count();

    let line_start = before.rfind('\n').map(|pos| pos + 1).unwrap_or(0);
    let col_idx = offset - line_start;

    let line_end = code[line_start..]
        .find('\n')
        .map(|pos| line_start + pos)
        .unwrap_or(code.len());

    let line_str = &code[line_start..line_end];
    (line_idx, col_idx, line_str)
}

pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let len_b = b.chars().count();
    let mut prev = (0..=len_b).collect::<Vec<_>>();
    let mut curr = vec![0; len_b + 1];

    for (i, ca) in a.chars().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] =
                std::cmp::min(std::cmp::min(curr[j] + 1, prev[j + 1] + 1), prev[j] + cost);
        }
        prev.copy_from_slice(&curr);
    }
    prev[len_b]
}

pub fn find_closest_match<'a>(
    target: &str,
    candidates: &[&'a str],
    max_dist: usize,
) -> Option<&'a str> {
    let mut best: Option<(&'a str, usize)> = None;
    for &cand in candidates {
        let dist = levenshtein_distance(target, cand);
        if dist <= max_dist {
            if let Some((_, best_dist)) = best {
                if dist < best_dist {
                    best = Some((cand, dist));
                }
            } else {
                best = Some((cand, dist));
            }
        }
    }
    best.map(|(c, _)| c)
}

pub const STANDARD_BUILTIN_IMPORTS: &[(&str, &str)] = &[
    ("print", "cap:io"),
    ("println", "cap:io"),
    ("read_line", "cap:io"),
    ("fs", "cap:fs"),
    ("File", "cap:fs"),
    ("json", "cap:json"),
    ("Json", "cap:json"),
    ("actor", "cap:actor"),
    ("spawn", "cap:actor"),
    ("join", "cap:actor"),
    ("SharedTensor", "cap:mem"),
    ("Socket", "cap:net"),
    ("TcpServer", "cap:net"),
    ("HttpServer", "cap:http"),
    ("HttpRequest", "cap:http"),
    ("HttpResponse", "cap:http"),
];

pub fn from_compiler_error_msg(raw_msg: &str, source_code: Option<&str>) -> Diagnostic {
    let msg = raw_msg.trim_start_matches("CP Compile Error:").trim();

    let mut diag = if msg.contains("Cannot perform mutable operation")
        && msg.contains("read-only parameter")
    {
        let param_name = msg
            .split("read-only parameter '")
            .nth(1)
            .and_then(|s| s.split('\'').next())
            .unwrap_or("param");
        Diagnostic::new(
            "E0201",
            format!("cannot perform mutable operation on read-only parameter '{}'", param_name),
        )
        .with_cause(format!(
            "parameter '{}' is a read-only borrow (immutable by default in CP).",
            param_name
        ))
        .with_help("in CP, function parameters are read-only borrows by default. To modify arguments in-place, declare them as exclusive mutable borrows using '*' prefix.")
        .with_suggestion(format!(
            "parameter '{}' is a read-only borrow. Add '*' to declare it as mutable borrow: '*{}: <Type>'",
            param_name, param_name
        ))
    } else if msg.contains("read-only method") {
        Diagnostic::new(
            "E0201",
            "cannot perform mutable operation in a read-only method",
        )
        .with_cause("method receiver is read-only by default.")
        .with_help("methods that modify instance fields must declare a mutable receiver parameter '*_' or '*this'.")
        .with_suggestion("add '*_' or '*this' as the first parameter of the method: fn method(*_) { ... }")
    } else if msg.contains("read-only borrow") {
        let parts: Vec<&str> = msg.split('\'').collect();
        let var_name = if parts.len() >= 4 {
            parts[3]
        } else if parts.len() >= 2 {
            parts[1]
        } else {
            "variable"
        };
        Diagnostic::new(
            "E0201",
            format!(
                "cannot perform mutable operation through read-only borrow '{}'",
                var_name
            ),
        )
        .with_cause(format!(
            "'{}' is borrowed as read-only (immutable).",
            var_name
        ))
        .with_help("in CP, mutable operations require exclusive mutable borrows ('*...').")
        .with_suggestion(format!(
            "declare '{}' with '*' for mutable borrowing: '*{}'",
            var_name, var_name
        ))
    } else if msg.contains("Borrow conflict") {
        let target = msg
            .split("on '")
            .nth(1)
            .and_then(|s| s.split('\'').next())
            .unwrap_or("variable");
        Diagnostic::new("E0202", msg)
            .with_cause(format!("variable '{}' is already borrowed in the current scope.", target))
            .with_help("CP enforces single-writer OR multiple-readers aliasing XOR mutability at compile time.")
            .with_suggestion(format!("ensure existing borrows of '{}' go out of scope before taking a mutable borrow, or clone the value.", target))
    } else if msg.contains("borrowed in a 'for' loop") {
        Diagnostic::new("E0210", msg)
            .with_cause("collection is actively borrowed by an active for-loop iterator.")
            .with_help("mutating a container while iterating over it causes iterator invalidation undefined behavior.")
            .with_suggestion("iterate over a cloned collection or use an index-based 'while' loop")
    } else if msg.contains("Use of moved value") {
        let var_name = msg.split('\'').nth(1).unwrap_or("variable");
        Diagnostic::new("E0203", format!("use of moved value '{}'", var_name))
            .with_cause(format!("value of '{}' was moved and ownership was transferred.", var_name))
            .with_help("once a value is transferred with 'move', the original identifier cannot be accessed.")
            .with_suggestion(format!("avoid moving '{}' if you need it later, or pass by borrow '&{}'", var_name, var_name))
    } else if msg.contains("Cannot move") && msg.contains("borrowed by") {
        Diagnostic::new("E0203", msg)
            .with_cause("attempted to move a value that is currently borrowed.")
            .with_help("values cannot be moved or transferred while active borrows exist.")
            .with_suggestion("ensure all borrows end before moving the value.")
    } else {
        Diagnostic::new("E0200", format!("compile error: {}", msg))
            .with_help("check CP language ownership, borrowing and semantic rules.")
    };

    // If source code is present, try locating the symbol in source code for pinpoint span
    if let Some(code) = source_code {
        if let Some(target) = msg.split('\'').nth(1).and_then(|s| s.split('\'').next()) {
            if let Some(pos) = code.find(target) {
                diag.span = Some(crate::Span {
                    start: pos,
                    end: pos + target.len(),
                });
            }
        }
    }

    diag
}
