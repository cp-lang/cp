use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use crate::parser::Parser;
use crate::sema::Analyzer;

pub struct Backend {
    client: Client,
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

    async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
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
            
            // Control Flow
            CompletionItem::new_simple("match".to_string(), "Pattern matching".to_string()),
            CompletionItem::new_simple("defer".to_string(), "Defer execution".to_string()),
            CompletionItem::new_simple("errdefer".to_string(), "Defer execution on error".to_string()),
            CompletionItem::new_simple("await".to_string(), "Await asynchronous operation".to_string()),
            CompletionItem::new_simple("async".to_string(), "Asynchronous function".to_string()),
            
            // Built-ins (RFC-002)
            CompletionItem::new_simple("@spawn".to_string(), "Spawn a new actor: @spawn(target, ...args)".to_string()),
            CompletionItem::new_simple("@receive".to_string(), "Receive message from mailbox: @receive()".to_string()),
            CompletionItem::new_simple("@reply".to_string(), "Send message to an actor: @reply(pid, msg)".to_string()),
            CompletionItem::new_simple("@print".to_string(), "Print to console: @print(fmt, ...args)".to_string()),
            CompletionItem::new_simple("@self".to_string(), "Get current actor PID: @self()".to_string()),
            CompletionItem::new_simple("@shared_tensor_init".to_string(), "Initialize shared off-heap tensor: @shared_tensor_init(shape)".to_string()),
            CompletionItem::new_simple("@matmul".to_string(), "High performance matrix multiplication: @matmul(a, b, out)".to_string()),
            CompletionItem::new_simple("@alloc".to_string(), "Allocate memory in agent arena".to_string()),
            CompletionItem::new_simple("@free".to_string(), "Free agent arena memory".to_string()),
            CompletionItem::new_simple("@release".to_string(), "Release reference counted shared memory: @release(obj)".to_string()),
            
            // Cap Standard Library (Imports)
            CompletionItem::new_simple("cap:io".to_string(), "Standard IO library (print)".to_string()),
            CompletionItem::new_simple("cap:mem".to_string(), "Standard Memory library (Arena, SharedTensor)".to_string()),
            CompletionItem::new_simple("cap:actor".to_string(), "Standard Actor library (spawn, receive, reply, join)".to_string()),
        ])))
    }
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    async fn on_change(&self, uri: Url, text: String) {
        let mut diagnostics = Vec::new();

        let mut parser = Parser::new(&text);
        match parser.parse_module() {
            Ok(module) => {
                let mut analyzer = Analyzer::new();
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

        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }
}

pub async fn run_lsp() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
