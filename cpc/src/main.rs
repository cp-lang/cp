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

fn resolve_file_or_dir(base: &Path) -> Option<PathBuf> {
    let cp_file = base.with_extension("cp");
    if cp_file.exists() {
        return Some(cp_file);
    }
    if base.is_dir() {
        let index_cp = base.join("index.cp");
        if index_cp.exists() {
            return Some(index_cp);
        }
        let main_cp = base.join("main.cp");
        if main_cp.exists() {
            return Some(main_cp);
        }
        let cap_json = base.join("cap.json");
        if cap_json.exists() {
            if let Ok(content) = fs::read_to_string(&cap_json) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(main_file) = json.get("main").and_then(|v| v.as_str()) {
                        let main_path = base.join(main_file);
                        if main_path.exists() {
                            return Some(main_path);
                        }
                    }
                }
            }
        }
    }
    None
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
                let mut search_source = imp.source.clone();
                if let Some(stripped) = search_source.strip_prefix("cap:") {
                    search_source = format!("cap/{}", stripped);
                }

                let mut resolved_path = None;

                if search_source.starts_with("./") || search_source.starts_with("../") || search_source.starts_with("/") || search_source.starts_with("cap/") {
                    // Relative, absolute or cap: import
                    let base_path = path.parent().unwrap().join(&search_source);
                    resolved_path = resolve_file_or_dir(&base_path);
                    
                    // Fallback for include paths if still not found (legacy behavior)
                    if resolved_path.is_none() {
                        for inc in include_paths {
                            let p = inc.join(&search_source);
                            if let Some(res) = resolve_file_or_dir(&p) {
                                resolved_path = Some(res);
                                break;
                            }
                        }
                    }
                } else {
                    // Dependency import (search upwards for cap_modules)
                    let mut current_dir = path.parent().unwrap();
                    loop {
                        let dep_path = current_dir.join("cap_modules").join(&search_source);
                        if let Some(p) = resolve_file_or_dir(&dep_path) {
                            resolved_path = Some(p);
                            break;
                        }
                        if let Some(parent) = current_dir.parent() {
                            current_dir = parent;
                        } else {
                            break;
                        }
                    }

                    // Fallback to global installation
                    if resolved_path.is_none() {
                        let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_else(|_| String::from("~"));
                        let global_path = PathBuf::from(&home).join(".cap/global/cap_modules").join(&search_source);
                        resolved_path = resolve_file_or_dir(&global_path);
                    }
                }

                if let Some(p) = resolved_path {
                    resolve_imports(&p, include_paths, all_items, processed)?;
                } else {
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
