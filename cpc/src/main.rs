use clap::{Parser, Subcommand};
use cpc::ast::Module;
use cpc::parser::Parser as CPParser;
use cpc::sema::Analyzer;
use cpc::codegen::Codegen;
use std::fs;
use std::path::PathBuf;
use std::collections::HashSet;

use cpc::resolver::resolve_imports;

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
        #[arg(short = 'o', long)] 
        output: Option<PathBuf>, 
        #[arg(short = 'I', long)] 
        include: Vec<PathBuf>,
        #[arg(short = 'b', long)]
        beam: bool 
    },
    Check { input: PathBuf },
    EmitTypes { input: PathBuf, #[arg(short = 'o', long)] output: Option<PathBuf> },
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
        Commands::Compile { input, output, include, beam } => {
            let mut all_items = Vec::new();
            let mut processed_sources = HashSet::new();
            
            resolve_imports(&input, &include, &mut all_items, &mut processed_sources)?;

            let module = Module { span: cpc::Span { start: 0, end: 0 }, body: all_items };

            let mut analyzer = Analyzer::new();
            analyzer.analyze_module(&module).map_err(|e| anyhow::anyhow!("Semantic Error: {}", e))?;

            let mut codegen = Codegen::new();
            codegen.is_beam_mode = beam;
            let zig_code = codegen.generate_module(&module);

            match output {
                Some(path) => fs::write(path, zig_code)?,
                None => println!("{}", zig_code),
            }
        }
        Commands::Check { input } => {
            let code = fs::read_to_string(&input)?;
            let mut parser = CPParser::new(&code);
            let module = parser.parse_module().map_err(|e| anyhow::anyhow!("Parse Error in {:?}: {}", input, e))?;
            let mut analyzer = Analyzer::new();
            analyzer.analyze_module(&module)?;
            println!("Check successful");
        }
        Commands::EmitTypes { input, output } => {
            let code = fs::read_to_string(&input)?;
            let mut parser = CPParser::new(&code);
            let module = parser.parse_module().map_err(|e| anyhow::anyhow!("Parse Error in {:?}: {}", input, e))?;
            let mut emitter = cpc::dcp::DcpEmitter::new();
            let type_defs = emitter.emit_module(&module);
            
            let out_path = output.unwrap_or_else(|| input.with_extension("d.cp"));
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&out_path, type_defs)?;
        }
        Commands::Lsp => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(cpc::lsp::run_lsp());
        }
    }
    Ok(())
}
