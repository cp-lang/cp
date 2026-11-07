use clap::{Parser, Subcommand};
use cpc::ast::Module;
use cpc::codegen::Codegen;
use cpc::parser::Parser as CPParser;
use cpc::sema::Analyzer;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use cpc::resolver::resolve_imports;

#[derive(Parser)]
#[command(name = "cpc", about = "CP Language Compiler", version)]
struct Cli {
    /// Input file to compile directly
    #[arg(value_name = "INPUT")]
    input: Option<PathBuf>,

    /// Output file path
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// Include paths for module resolution
    #[arg(short = 'I', long)]
    include: Vec<PathBuf>,

    /// Enable BEAM runtime mode
    #[arg(short = 'b', long)]
    beam: bool,

    /// Compile as library module (omit main entry point)
    #[arg(long)]
    lib: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile CP source to Zig (default action)
    Compile {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
        #[arg(short = 'I', long)]
        include: Vec<PathBuf>,
        #[arg(short = 'b', long)]
        beam: bool,
        #[arg(long)]
        lib: bool,
    },
    /// Perform static syntax and borrow-checker check
    Check { input: PathBuf },
    /// Emit .d.cp type declarations
    EmitTypes {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },
    /// Run Language Server Protocol daemon
    Lsp,
}

fn compile_file(
    input: &PathBuf,
    output: Option<&PathBuf>,
    include: &[PathBuf],
    beam: bool,
    lib: bool,
) -> anyhow::Result<()> {
    let mut all_items = Vec::new();
    let mut processed_sources = HashSet::new();

    if lib {
        let code = fs::read_to_string(input)?;
        let mut parser = CPParser::new(&code);
        let parsed_module = parser.parse_module().map_err(|e| {
            anyhow::anyhow!("{}", e.to_diagnostic().render(Some(&code), Some(input)))
        })?;
        all_items = parsed_module.body;
    } else {
        resolve_imports(input, include, &mut all_items, &mut processed_sources)?;
    }

    let mut module = Module {
        span: cpc::Span { start: 0, end: 0 },
        body: all_items,
    };

    let mut analyzer = Analyzer::new();
    analyzer.analyze_module(&mut module).map_err(|e| {
        let known = analyzer.get_known_symbols();
        let code = fs::read_to_string(input).ok();
        anyhow::anyhow!(
            "{}",
            e.to_diagnostic(&known).render(code.as_deref(), Some(input))
        )
    })?;

    let zig_code_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut codegen = Codegen::new();
        codegen.is_beam_mode = beam;
        codegen.is_lib_mode = lib;
        codegen.source_dir = input.parent().map(|p| p.to_path_buf());
        codegen.generate_module(&module)
    }));

    let zig_code = match zig_code_result {
        Ok(code) => code,
        Err(err) => {
            let msg = if let Some(s) = err.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = err.downcast_ref::<String>() {
                s.clone()
            } else {
                "Internal compiler error".to_string()
            };
            let code = fs::read_to_string(input).ok();
            let diag = cpc::diagnostic::from_compiler_error_msg(&msg, code.as_deref());
            return Err(anyhow::anyhow!(
                "{}",
                diag.render(code.as_deref(), Some(input))
            ));
        }
    };

    match output {
        Some(path) => {
            if let Ok(existing) = fs::read_to_string(path) {
                if existing == zig_code {
                    return Ok(());
                }
            }
            fs::write(path, zig_code)?;
        }
        None => println!("{}", zig_code),
    }
    Ok(())
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        let err_str = e.to_string();
        if err_str.starts_with("error[") {
            eprint!("{}", err_str);
        } else {
            eprintln!("Error: {}", err_str);
        }
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Compile {
                input,
                output,
                include,
                beam,
                lib,
            } => {
                compile_file(&input, output.as_ref(), &include, beam, lib)?;
            }
            Commands::Check { input } => {
                let code = fs::read_to_string(&input)?;
                let mut parser = CPParser::new(&code);
                let mut module = parser.parse_module().map_err(|e| {
                    anyhow::anyhow!("{}", e.to_diagnostic().render(Some(&code), Some(&input)))
                })?;
                let mut analyzer = Analyzer::new();
                analyzer.analyze_module(&mut module).map_err(|e| {
                    let known = analyzer.get_known_symbols();
                    anyhow::anyhow!(
                        "{}",
                        e.to_diagnostic(&known).render(Some(&code), Some(&input))
                    )
                })?;
                println!("Check successful");
            }
            Commands::EmitTypes { input, output } => {
                let code = fs::read_to_string(&input)?;
                let mut parser = CPParser::new(&code);
                let module = parser
                    .parse_module()
                    .map_err(|e| anyhow::anyhow!("Parse Error in {:?}: {}", input, e))?;
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
    } else if let Some(input) = cli.input {
        compile_file(&input, cli.output.as_ref(), &cli.include, cli.beam, cli.lib)?;
    } else {
        use clap::CommandFactory;
        Cli::command().print_help()?;
        println!();
    }
    Ok(())
}
