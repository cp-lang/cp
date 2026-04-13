use clap::{Parser, Subcommand};
use cpc::ast::{Module, ModuleItem, Decl};
use cpc::parser::Parser as CPParser;
use cpc::sema::Analyzer;
use cpc::codegen::Codegen;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashSet;

#[derive(Parser)]
#[command(name = "cp-compiler")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Compile {
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(short = 'I', long)]
        include: Vec<PathBuf>,
    },
    Check { input: PathBuf },
    EmitTypes { input: PathBuf },
    Lsp,
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Compile { input, output, include } => {
            let mut all_items = Vec::new();
            let mut processed_sources = HashSet::new();
            
            resolve_imports(&input, &include, &mut all_items, &mut processed_sources)?;

            let module = Module { span: cpc::Span { start: 0, end: 0 }, body: all_items };

            let mut analyzer = Analyzer::new();
            analyzer.analyze_module(&module).map_err(|e| anyhow::anyhow!("Semantic Error: {}", e))?;

            let mut codegen = Codegen::new();
            let zig_code = codegen.generate_module(&module);

            match output {
                Some(path) => fs::write(path, zig_code)?,
                None => println!("{}", zig_code),
            }
        }
        Commands::Check { input } => {
            let code = fs::read_to_string(&input)?;
            let mut parser = CPParser::new(&code);
            let module = parser.parse_module().map_err(|e| anyhow::anyhow!("Parse Error: {}", e))?;
            let mut analyzer = Analyzer::new();
            analyzer.analyze_module(&module)?;
            println!("Check successful");
        }
        Commands::EmitTypes { input: _ } => {
            // TODO
        }
        Commands::Lsp => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(cpc::lsp::run_lsp());
        }
    }
    Ok(())
}

fn resolve_imports(
    path: &Path,
    include_paths: &[PathBuf],
    all_items: &mut Vec<ModuleItem>,
    processed: &mut HashSet<PathBuf>,
) -> anyhow::Result<()> {
    let abs_path = fs::canonicalize(path)?;
    if processed.contains(&abs_path) {
        return Ok(());
    }
    processed.insert(abs_path.clone());

    let code = fs::read_to_string(path)?;
    let mut parser = CPParser::new(&code);
    let module = parser.parse_module().map_err(|e| anyhow::anyhow!("Parse Error in {:?}: {}", path, e))?;

    for item in module.body {
        match item {
            ModuleItem::Import(imp) => {
                // Try to resolve the source
                let mut resolved_path = None;
                
                // 1. Relative to current file
                let rel_path = path.parent().unwrap().join(&imp.source).with_extension("cp");
                if rel_path.exists() {
                    resolved_path = Some(rel_path);
                } else {
                    // 2. In include paths (like con_modules)
                    for inc in include_paths {
                        let p = inc.join(&imp.source).with_extension("cp");
                        if p.exists() {
                            resolved_path = Some(p);
                            break;
                        }
                    }
                }

                if let Some(p) = resolved_path {
                    resolve_imports(&p, include_paths, all_items, processed)?;
                } else {
                    // If not found, it might be a standard library or external binary
                    // For now, keep the import item
                    all_items.push(ModuleItem::Import(imp));
                }
            }
            _ => {
                all_items.push(item);
            }
        }
    }

    Ok(())
}
