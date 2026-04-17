use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use crate::parser::Parser;
use crate::sema::{Analyzer, SymbolKind};
use crate::resolver::resolve_dependencies;
use crate::ast::{Module, Type};
use tokio::sync::Mutex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::path::PathBuf;

pub struct Backend {
    client: Client,
    // (Text, Analyzer)
    state: Arc<Mutex<HashMap<Url, (String, Analyzer)>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "CP LSP Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.on_change(params.text_document.uri, params.text_document.text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.on_change(params.text_document.uri, params.content_changes[0].text.clone()).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        let state = self.state.lock().await;
        if let Some((text, analyzer)) = state.get(&uri) {
            if let Some(offset) = position_to_offset(text, position) {
                // Find symbol at offset
                for (span, kind) in &analyzer.symbol_map {
                    if offset >= span.start && offset <= span.end {
                        let content = format_symbol_kind(kind);
                        return Ok(Some(Hover {
                            contents: HoverContents::Scalar(MarkedString::String(content)),
                            range: None,
                        }));
                    }
                }
            }
        }
        
        Ok(None)
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![
            // Declarations
            CompletionItem::new_simple("fn".to_string(), "Function definition".to_string()),
            CompletionItem::new_simple("let".to_string(), "Variable declaration".to_string()),
            CompletionItem::new_simple("const".to_string(), "Constant declaration".to_string()),
            CompletionItem::new_simple("class".to_string(), "Class definition".to_string()),
            CompletionItem::new_simple("trait".to_string(), "Trait definition".to_string()),
            CompletionItem::new_simple("impl".to_string(), "Trait implementation".to_string()),
            CompletionItem::new_simple("enum".to_string(), "Enum definition".to_string()),
            CompletionItem::new_simple("error".to_string(), "Error set definition".to_string()),
            CompletionItem::new_simple("match".to_string(), "Pattern matching".to_string()),
            CompletionItem::new_simple("defer".to_string(), "Defer execution".to_string()),
            CompletionItem::new_simple("errdefer".to_string(), "Defer execution on error".to_string()),
            CompletionItem::new_simple("await".to_string(), "Await asynchronous operation".to_string()),
            CompletionItem::new_simple("async".to_string(), "Asynchronous function".to_string()),
            CompletionItem::new_simple("@spawn".to_string(), "Spawn a new actor".to_string()),
            CompletionItem::new_simple("@receive".to_string(), "Receive message".to_string()),
            CompletionItem::new_simple("@reply".to_string(), "Send message".to_string()),
            CompletionItem::new_simple("@print".to_string(), "Print to console".to_string()),
            CompletionItem::new_simple("@self".to_string(), "Get current PID".to_string()),
            CompletionItem::new_simple("cap:io".to_string(), "Standard IO library".to_string()),
            CompletionItem::new_simple("cap:mem".to_string(), "Standard Memory library".to_string()),
            CompletionItem::new_simple("cap:actor".to_string(), "Standard Actor library".to_string()),
        ])))
    }
}

fn position_to_offset(text: &str, position: Position) -> Option<usize> {
    let mut line = 0;
    let mut col = 0;
    for (i, c) in text.char_indices() {
        if line == position.line && col == position.character {
            return Some(i);
        }
        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    None
}

fn format_symbol_kind(kind: &SymbolKind) -> String {
    match kind {
        SymbolKind::Var { ty, .. } => format!("(variable) {}", format_type(ty)),
        SymbolKind::Func { params, ret, is_async } => {
            let mut s = if *is_async { "async fn (".to_string() } else { "fn (".to_string() };
            for (i, p) in params.iter().enumerate() {
                s.push_str(&format_type(p));
                if i < params.len() - 1 { s.push_str(", "); }
            }
            s.push_str(")");
            if let Some(r) = ret {
                s.push_str(&format!(": {}", format_type(r)));
            }
            s
        },
        SymbolKind::Class(c) => format!("class {}", c.ident.sym),
        SymbolKind::Trait(t) => format!("trait {}", t.ident.sym),
        SymbolKind::Enum(e) => format!("enum {}", e.ident.sym),
    }
}

fn format_type(ty: &Type) -> String {
    match ty {
        Type::I32 => "i32".to_string(),
        Type::U64 => "u64".to_string(),
        Type::F32 => "f32".to_string(),
        Type::F64 => "f64".to_string(),
        Type::USize => "usize".to_string(),
        Type::Bool => "bool".to_string(),
        Type::String => "string".to_string(),
        Type::Void => "void".to_string(),
        Type::Any => "any".to_string(),
        Type::PID => "PID".to_string(),
        Type::Ref(id) => id.sym.clone(),
        Type::Array(inner) => format!("[]{}", format_type(inner)),
        Type::Optional(inner) => format!("?{}", format_type(inner)),
        Type::ErrorUnion(inner) => format!("!{}", format_type(inner)),
        _ => "unknown".to_string(),
    }
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self { 
            client,
            state: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn on_change(&self, uri: Url, text: String) {
        let mut diagnostics = Vec::new();
        let mut analyzer = Analyzer::new();

        let mut parser = Parser::new(&text);
        match parser.parse_module() {
            Ok(module) => {
                // Resolve dependencies (load .d.cp files)
                if let Ok(path) = uri.to_file_path() {
                    let mut items = Vec::new();
                    let mut processed = HashSet::new();
                    
                    // We don't want to resolve the current file from disk, 
                    // we already have it in memory.
                    // But we want to resolve its dependencies.
                    let include_paths = vec![
                        PathBuf::from("/data/cps/cap/cap_modules"),
                        PathBuf::from("/data/cps/lib"),
                    ];
                    
                    if let Err(e) = resolve_dependencies(&path, &include_paths, &mut items, &mut processed) {
                         self.client.log_message(MessageType::LOG, format!("Dependency resolution failed: {}", e)).await;
                    }

                    // Pre-fill analyzer with dependency items
                    let dep_module = Module { span: crate::Span { start: 0, end: 0 }, body: items };
                    if let Err(_e) = analyzer.analyze_module(&dep_module) {
                         // Some dependency analysis errors, maybe log?
                    }
                    
                    // Now analyze the current file's items
                    if let Err(e) = analyzer.analyze_module(&module) {
                        diagnostics.push(Diagnostic {
                            range: Range::default(),
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: e.to_string(),
                            source: Some("cpc-sema".to_string()),
                            ..Default::default()
                        });
                    }
                }
            }
            Err(e) => {
                diagnostics.push(Diagnostic {
                    range: Range::default(),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: e.to_string(),
                    source: Some("cpc-parser".to_string()),
                    ..Default::default()
                });
            }
        }

        // Store analysis result
        {
            let mut state = self.state.lock().await;
            state.insert(uri.clone(), (text, analyzer));
        }

        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }
}

pub async fn run_lsp() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
