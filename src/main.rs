use clap::{Parser, Subcommand};
use cpc::parser::Parser as CPParser;
use cpc::sema::Analyzer;
use cpc::codegen::Codegen;
use std::fs;
use std::path::PathBuf;

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
    },
    Check { input: PathBuf },
    EmitTypes { input: PathBuf },
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
        Commands::Compile { input, output } => {
            let code = fs::read_to_string(&input)?;
            let mut parser = CPParser::new(&code);
            let module = parser.parse_module().map_err(|e| anyhow::anyhow!("Parse Error: {}", e))?;

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
        _ => {}
    }
    Ok(())
}
