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
    Check {
        input: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Emit .d.cp type declarations
    EmitTypes {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },
    /// Demangle runtime stack trace using source maps
    Demangle {
        /// Optional stack trace file (reads stdin if omitted)
        input: Option<PathBuf>,
        /// Optional explicit source map path
        #[arg(short = 'm', long)]
        map: Option<PathBuf>,
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
        let parsed_module = parser
            .parse_module()
            .map_err(|e| anyhow::anyhow!("{}", e.render_all(Some(&code), Some(input))))?;
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

    let code_str = fs::read_to_string(input).ok();
    let zig_code_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut codegen = Codegen::new();
        codegen.is_beam_mode = beam;
        codegen.is_lib_mode = lib;
        codegen.source_dir = input.parent().map(|p| p.to_path_buf());
        if let Some(ref c) = code_str {
            codegen.init_source(Some(&input.display().to_string()), Some(c));
        }
        let code = codegen.generate_module(&module);
        (code, codegen.source_map_entries)
    }));

    let (zig_code, map_entries) = match zig_code_result {
        Ok(res) => res,
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
            let mut need_write = true;
            if let Ok(existing) = fs::read_to_string(path) {
                if existing == zig_code {
                    need_write = false;
                }
            }
            if need_write {
                fs::write(path, &zig_code)?;
            }
            let mut sm = cpc::source_map::SourceMap::new(
                input.display().to_string(),
                path.display().to_string(),
            );
            sm.mappings = map_entries;
            let map_path = path.with_extension("zig.map");
            let _ = fs::write(map_path, sm.to_json());
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
            Commands::Check { input, json } => {
                let code = match fs::read_to_string(&input) {
                    Ok(c) => c,
                    Err(e) => {
                        if json {
                            let diag = cpc::diagnostic::Diagnostic::new(
                                "E0001",
                                format!("failed to read input file '{}': {}", input.display(), e),
                            );
                            println!("{}", diag.to_json(None, Some(&input)));
                            std::process::exit(1);
                        } else {
                            return Err(anyhow::anyhow!("failed to read file {:?}: {}", input, e));
                        }
                    }
                };
                let mut parser = CPParser::new(&code);
                let mut module = match parser.parse_module() {
                    Ok(m) => m,
                    Err(e) => {
                        if json {
                            println!("{}", e.to_json_report(Some(&code), Some(&input)));
                            std::process::exit(1);
                        } else {
                            return Err(anyhow::anyhow!(
                                "{}",
                                e.render_all(Some(&code), Some(&input))
                            ));
                        }
                    }
                };
                let mut analyzer = Analyzer::new();
                if let Err(e) = analyzer.analyze_module(&mut module) {
                    let known = analyzer.get_known_symbols();
                    let diag = e.to_diagnostic(&known);
                    if json {
                        println!("{}", diag.to_json(Some(&code), Some(&input)));
                        std::process::exit(1);
                    } else {
                        return Err(anyhow::anyhow!(
                            "{}",
                            diag.render(Some(&code), Some(&input))
                        ));
                    }
                }
                if json {
                    println!("{}", cpc::diagnostic::DiagnosticReport::ok_json());
                    std::process::exit(0);
                } else {
                    println!("Check successful");
                }
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
            Commands::Demangle { input, map } => {
                let trace = match input {
                    Some(p) => fs::read_to_string(&p)?,
                    None => {
                        use std::io::Read;
                        let mut buf = String::new();
                        std::io::stdin().read_to_string(&mut buf)?;
                        buf
                    }
                };
                let sm = match map {
                    Some(m_path) => Some(cpc::source_map::SourceMap::from_file(&m_path)?),
                    None => None,
                };
                let demangled = cpc::source_map::demangle_stack_trace(&trace, sm.as_ref());
                print!("{}", demangled);
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
