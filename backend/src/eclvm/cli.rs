//! # ECL CLI Tool
//!
//! Kommandozeilen-Tool zum Testen und Ausführen von ECL-Policies.
//!
//! ## Usage
//!
//! ```bash
//! # REPL starten
//! ecl repl
//!
//! # Datei ausführen
//! ecl run policy.ecl --context context.json
//!
//! # Zu Bytecode kompilieren
//! ecl compile policy.ecl -o policy.eclc
//!
//! # Expression evaluieren
//! ecl eval "2 + 3 * 4"
//! ```

use anyhow::{Context, Result};
use clap::{Parser as ClapParser, Subcommand};
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::eclvm::bytecode::{OpCode, Value};
use crate::eclvm::compiler::Compiler;
use crate::eclvm::optimizer::Optimizer;
use crate::eclvm::parser::Parser as EclParser;
use crate::eclvm::runtime::host::StubHost;
use crate::eclvm::runtime::vm::ECLVM;

/// ECL - Erynoa Configuration Language CLI
#[derive(ClapParser)]
#[command(name = "ecl")]
#[command(author = "Erynoa Team")]
#[command(version = "1.0.0")]
#[command(about = "ECL Policy Testing and Development Tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start interactive REPL
    Repl {
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Evaluate a single expression
    Eval {
        /// The expression to evaluate
        expression: String,

        /// Show bytecode
        #[arg(short, long)]
        bytecode: bool,
    },

    /// Compile ECL file to bytecode
    Compile {
        /// Input ECL file
        input: PathBuf,

        /// Output bytecode file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Enable optimizations
        #[arg(short = 'O', long)]
        optimize: bool,

        /// Show disassembly
        #[arg(short, long)]
        disasm: bool,
    },

    /// Run ECL policy with context
    Run {
        /// Input ECL file
        input: PathBuf,

        /// Context JSON file
        #[arg(short, long)]
        context: Option<PathBuf>,

        /// Gas limit
        #[arg(short, long, default_value = "10000")]
        gas_limit: u64,

        /// Show execution trace
        #[arg(short, long)]
        trace: bool,
    },

    /// Check ECL syntax without running
    Check {
        /// Input ECL file
        input: PathBuf,
    },

    /// Format ECL file (pretty print)
    Fmt {
        /// Input ECL file
        input: PathBuf,

        /// Write to file instead of stdout
        #[arg(short, long)]
        write: bool,
    },
}

/// REPL State
struct ReplState {
    history: Vec<String>,
    variables: HashMap<String, Value>,
    verbose: bool,
}

impl ReplState {
    fn new(verbose: bool) -> Self {
        Self {
            history: Vec::new(),
            variables: HashMap::new(),
            verbose,
        }
    }
}

/// Hauptfunktion für das CLI
pub fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Repl { verbose } => run_repl(verbose),
        Commands::Eval {
            expression,
            bytecode,
        } => eval_expression(&expression, bytecode).map(|_| ()),
        Commands::Compile {
            input,
            output,
            optimize,
            disasm,
        } => compile_file(&input, output.as_ref(), optimize, disasm),
        Commands::Run {
            input,
            context,
            gas_limit,
            trace,
        } => run_file(&input, context.as_ref(), gas_limit, trace),
        Commands::Check { input } => check_file(&input),
        Commands::Fmt { input, write } => format_file(&input, write),
    }
}

/// Interaktive REPL
fn run_repl(verbose: bool) -> Result<()> {
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║       ECL REPL - Erynoa Configuration Language    ║".cyan()
    );
    println!(
        "{}",
        "║  Type 'help' for commands, 'quit' to exit         ║".cyan()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════╝".cyan()
    );
    println!();

    let mut rl = DefaultEditor::new()?;
    let mut state = ReplState::new(verbose);

    loop {
        let readline = rl.readline(&format!("{} ", "ecl>".green().bold()));

        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(line);
                state.history.push(line.to_string());

                match handle_repl_command(line, &mut state) {
                    Ok(should_quit) => {
                        if should_quit {
                            println!("{}", "Goodbye!".yellow());
                            break;
                        }
                    }
                    Err(e) => {
                        println!("{}: {}", "Error".red().bold(), e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "^C".yellow());
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "Goodbye!".yellow());
                break;
            }
            Err(err) => {
                println!("{}: {:?}", "Error".red(), err);
                break;
            }
        }
    }

    Ok(())
}

/// REPL Kommando verarbeiten
fn handle_repl_command(line: &str, state: &mut ReplState) -> Result<bool> {
    // Spezielle Kommandos
    match line.to_lowercase().as_str() {
        "quit" | "exit" | "q" => return Ok(true),
        "help" | "h" | "?" => {
            print_help();
            return Ok(false);
        }
        "clear" => {
            print!("\x1B[2J\x1B[1;1H");
            return Ok(false);
        }
        "history" => {
            for (i, cmd) in state.history.iter().enumerate() {
                println!("{:4}: {}", i + 1, cmd);
            }
            return Ok(false);
        }
        "vars" => {
            if state.variables.is_empty() {
                println!("{}", "No variables defined".dimmed());
            } else {
                for (name, value) in &state.variables {
                    println!("  {} = {}", name.cyan(), format_value(value));
                }
            }
            return Ok(false);
        }
        _ => {}
    }

    // Kommandos mit Argumenten
    if line.starts_with(":type ") {
        let expr = &line[6..];
        return eval_and_show_type(expr);
    }

    if line.starts_with(":bytecode ") || line.starts_with(":bc ") {
        let expr = if line.starts_with(":bc ") {
            &line[4..]
        } else {
            &line[10..]
        };
        return show_bytecode(expr);
    }

    if line.starts_with(":load ") {
        let path = &line[6..];
        return load_file(path.trim());
    }

    // Expression evaluieren
    eval_expression(line, state.verbose)
}

fn print_help() {
    println!(
        "{}",
        "═══════════════════════════════════════════════════".cyan()
    );
    println!(
        "{}",
        "                   ECL REPL Help                   "
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "═══════════════════════════════════════════════════".cyan()
    );
    println!();
    println!("{}", "Commands:".yellow().bold());
    println!("  {}         - Exit the REPL", "quit, exit, q".green());
    println!("  {}         - Show this help", "help, h, ?".green());
    println!("  {}              - Clear screen", "clear".green());
    println!("  {}            - Show command history", "history".green());
    println!(
        "  {}               - Show defined variables",
        "vars".green()
    );
    println!();
    println!("{}", "Special Commands:".yellow().bold());
    println!(
        "  {}      - Show type of expression",
        ":type <expr>".green()
    );
    println!(
        "  {} - Show bytecode for expression",
        ":bytecode <expr>".green()
    );
    println!("  {}       - Load and run ECL file", ":load <file>".green());
    println!();
    println!("{}", "Examples:".yellow().bold());
    println!("  {} - Arithmetic", "2 + 3 * 4".dimmed());
    println!("  {} - Comparison", "5 > 3 && true".dimmed());
    println!(
        "  {} - Trust vector",
        "trust_norm([0.8, 0.7, 0.9, 0.6, 0.8, 0.7])".dimmed()
    );
    println!();
}

/// Expression evaluieren
fn eval_expression(expr: &str, show_bytecode: bool) -> Result<bool> {
    // Parsen
    let ast = EclParser::parse_expr(expr).context("Failed to parse expression")?;

    // Kompilieren
    let mut compiler = Compiler::new();
    let program = compiler
        .compile_expr(&ast)
        .context("Failed to compile expression")?;

    if show_bytecode {
        println!("{}", "Bytecode:".yellow());
        for (i, op) in program.iter().enumerate() {
            println!("  {:04}: {:?}", i, op);
        }
        println!();
    }

    // Optimieren
    let optimized = Optimizer::new().optimize(program);

    // Ausführen
    let host = StubHost::new();
    let mut vm = ECLVM::new_unlimited(optimized, &host);
    let result = vm.run().context("Execution failed")?;

    println!("{} {}", "=>".green(), format_value(&result.value));

    Ok(false)
}

/// Typ einer Expression anzeigen
fn eval_and_show_type(expr: &str) -> Result<bool> {
    let ast = EclParser::parse_expr(expr).context("Failed to parse expression")?;

    let mut compiler = Compiler::new();
    let program = compiler.compile_expr(&ast)?;

    let host = StubHost::new();
    let mut vm = ECLVM::new_unlimited(program, &host);
    let result = vm.run()?;

    println!(
        "{} {} : {}",
        "=>".green(),
        format_value(&result.value),
        result.value.type_name().cyan()
    );

    Ok(false)
}

/// Bytecode anzeigen
fn show_bytecode(expr: &str) -> Result<bool> {
    let ast = EclParser::parse_expr(expr)?;

    let mut compiler = Compiler::new();
    let program = compiler.compile_expr(&ast)?;

    println!("{}", "Unoptimized:".yellow());
    disassemble(&program);

    let optimized = Optimizer::new().optimize(program.clone());
    if optimized.len() < program.len() {
        println!();
        println!("{}", "Optimized:".green());
        disassemble(&optimized);
        println!(
            "  {} {} → {} instructions",
            "Reduced:".dimmed(),
            program.len(),
            optimized.len()
        );
    }

    Ok(false)
}

/// Disassembly ausgeben
fn disassemble(program: &[OpCode]) {
    for (i, op) in program.iter().enumerate() {
        let gas = op.gas_cost();
        println!(
            "  {:04}  {:40} {}",
            i,
            format!("{:?}", op),
            format!("(gas: {})", gas).dimmed()
        );
    }
}

/// Datei laden und ausführen
fn load_file(path: &str) -> Result<bool> {
    let content = fs::read_to_string(path).context(format!("Failed to read file: {}", path))?;

    println!("{} {}", "Loading:".yellow(), path);

    let (maybe_ast, diagnostics) = EclParser::parse_with_diagnostics(&content);

    if diagnostics.has_errors() {
        for diag in diagnostics.errors() {
            println!("{}: {}", "Error".red().bold(), diag.message);
        }
        return Ok(false);
    }

    if let Some(ast) = maybe_ast {
        let compiler = Compiler::new();
        let program = compiler.compile(&ast)?;
        let optimized = Optimizer::new().optimize(program);

        let host = StubHost::new();
        let mut vm = ECLVM::new(optimized, 10000, &host);
        let result = vm.run()?;

        println!("{} {}", "=>".green(), format_value(&result.value));
        println!(
            "  {} {}",
            "Gas used:".dimmed(),
            result.gas_used.to_string().yellow()
        );
    }

    Ok(false)
}

/// Datei kompilieren
fn compile_file(
    input: &PathBuf,
    output: Option<&PathBuf>,
    optimize: bool,
    disasm: bool,
) -> Result<()> {
    let content = fs::read_to_string(input)?;

    println!(
        "{} {}",
        "Compiling:".yellow(),
        input.display().to_string().cyan()
    );

    let (maybe_ast, diagnostics) = EclParser::parse_with_diagnostics(&content);

    // Diagnostics ausgeben (Warnings würden hier angezeigt, derzeit nur Errors in DiagnosticCollector)

    if diagnostics.has_errors() {
        for diag in diagnostics.errors() {
            println!("{}: {}", "Error".red().bold(), diag.message);
        }
        return Err(anyhow::anyhow!("Compilation failed with errors"));
    }

    let ast = maybe_ast.ok_or_else(|| anyhow::anyhow!("No AST produced"))?;

    let compiler = Compiler::new();
    let program = compiler.compile(&ast)?;

    let final_program = if optimize {
        println!("{}", "  Optimizing...".dimmed());
        let opt = Optimizer::new().optimize(program.clone());
        println!(
            "  {} {} → {} instructions",
            "Reduced:".dimmed(),
            program.len(),
            opt.len()
        );
        opt
    } else {
        program
    };

    if disasm {
        println!();
        println!("{}", "Disassembly:".yellow());
        disassemble(&final_program);
    }

    // Output schreiben
    if let Some(out_path) = output {
        let encoded = bincode::serialize(&final_program)?;
        fs::write(out_path, &encoded)?;
        println!(
            "{} {} ({} bytes)",
            "Written:".green(),
            out_path.display(),
            encoded.len()
        );
    }

    println!("{}", "✓ Compilation successful".green().bold());
    Ok(())
}

/// Datei ausführen
fn run_file(
    input: &PathBuf,
    context: Option<&PathBuf>,
    gas_limit: u64,
    _trace: bool,
) -> Result<()> {
    let content = fs::read_to_string(input)?;

    // Kontext laden
    let host = if let Some(ctx_path) = context {
        let ctx_content = fs::read_to_string(ctx_path)?;
        let ctx: JsonValue = serde_json::from_str(&ctx_content)?;
        build_host_from_context(&ctx)?
    } else {
        StubHost::new()
    };

    println!(
        "{} {}",
        "Running:".yellow(),
        input.display().to_string().cyan()
    );
    println!("  {} {}", "Gas limit:".dimmed(), gas_limit);

    let ast = EclParser::parse(&content)?;

    let compiler = Compiler::new();
    let program = compiler.compile(&ast)?;
    let optimized = Optimizer::new().optimize(program);

    let mut vm = ECLVM::new(optimized, gas_limit, &host);
    let result = vm.run()?;

    println!();
    println!(
        "{} {}",
        "Result:".green().bold(),
        format_value(&result.value)
    );
    println!(
        "  {} {} / {}",
        "Gas:".dimmed(),
        result.gas_used.to_string().yellow(),
        gas_limit
    );

    Ok(())
}

/// Syntax prüfen
fn check_file(input: &PathBuf) -> Result<()> {
    let content = fs::read_to_string(input)?;

    println!(
        "{} {}",
        "Checking:".yellow(),
        input.display().to_string().cyan()
    );

    let (maybe_ast, diagnostics) = EclParser::parse_with_diagnostics(&content);
    let _ = maybe_ast; // AST wird nur bei check nicht benötigt

    if !diagnostics.has_errors() {
        println!("{}", "✓ No errors found".green().bold());
        Ok(())
    } else {
        for diag in diagnostics.errors() {
            println!(
                "  {} {} (line {})",
                "✗".red(),
                diag.message,
                diag.labels.first().map(|l| l.span.line).unwrap_or(0)
            );
        }
        Err(anyhow::anyhow!("Errors found"))
    }
}

/// Datei formatieren
fn format_file(input: &PathBuf, write: bool) -> Result<()> {
    let content = fs::read_to_string(input)?;

    // Parsen
    let ast = EclParser::parse(&content)?;

    // Formatieren (vereinfacht - echtes Pretty-Printing in v2)
    let formatted = format!("{:#?}", ast);

    if write {
        // In v2: echtes Formatting
        println!("{}", "Formatting not yet implemented".yellow());
    } else {
        println!("{}", formatted);
    }

    Ok(())
}

/// Host aus JSON Context bauen
fn build_host_from_context(ctx: &JsonValue) -> Result<StubHost> {
    let mut host = StubHost::new();

    if let Some(trust) = ctx.get("trust") {
        if let Some(arr) = trust.as_array() {
            if arr.len() == 6 {
                let tv: [f64; 6] = [
                    arr[0].as_f64().unwrap_or(0.5),
                    arr[1].as_f64().unwrap_or(0.5),
                    arr[2].as_f64().unwrap_or(0.5),
                    arr[3].as_f64().unwrap_or(0.5),
                    arr[4].as_f64().unwrap_or(0.5),
                    arr[5].as_f64().unwrap_or(0.5),
                ];
                host = host.with_trust("requester", tv);
            }
        }
    }

    if let Some(balance) = ctx.get("balance") {
        if let Some(n) = balance.as_u64() {
            host = host.with_balance("requester", n);
        }
    }

    if let Some(credentials) = ctx.get("credentials") {
        if let Some(arr) = credentials.as_array() {
            for cred in arr {
                if let Some(schema) = cred.as_str() {
                    host = host.with_credential("requester", schema);
                }
            }
        }
    }

    Ok(host)
}

/// Wert formatieren für Ausgabe
fn format_value(value: &Value) -> String {
    match value {
        Value::Null => "null".dimmed().to_string(),
        Value::Bool(b) => {
            if *b {
                "true".green().to_string()
            } else {
                "false".red().to_string()
            }
        }
        Value::Number(n) => n.to_string().yellow().to_string(),
        Value::String(s) => format!("\"{}\"", s).cyan().to_string(),
        Value::DID(d) => format!("<DID:{}>", d).magenta().to_string(),
        Value::TrustVector(tv) => format!(
            "[R:{:.2} I:{:.2} C:{:.2} P:{:.2} V:{:.2} Ω:{:.2}]",
            tv[0], tv[1], tv[2], tv[3], tv[4], tv[5]
        )
        .blue()
        .to_string(),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(|v| format_value(v)).collect();
            format!("[{}]", items.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_value_number() {
        let v = Value::Number(42.0);
        let formatted = format_value(&v);
        assert!(formatted.contains("42"));
    }

    #[test]
    fn test_format_value_bool() {
        let v = Value::Bool(true);
        let formatted = format_value(&v);
        assert!(formatted.contains("true"));
    }

    #[test]
    fn test_format_value_trust_vector() {
        let v = Value::TrustVector([0.8, 0.7, 0.9, 0.6, 0.8, 0.7]);
        let formatted = format_value(&v);
        assert!(formatted.contains("R:0.80"));
        assert!(formatted.contains("Ω:0.70"));
    }
}
