// The modules live in the library target (src/lib.rs); this binary uses them
// from there rather than re-declaring them, which previously compiled the
// whole crate twice.
use std::fs;
use std::io::{self, Read};
use std::env;

use etamil_compiler::http::HttpServer;
use etamil_compiler::{lexer, parser, vm};
#[cfg(feature = "llvm")]
use etamil_compiler::codegen;

// main is async so the tokio runtime is available to the server paths.
//
// Note on numbering: "backend milestone N" here refers to this repository's
// own HTTP work (1 sync server, 2 async, 3 logging, 4 auth). It is unrelated
// to the Phase 1-5 in the eTamil paper, which numbers compiler core, domain
// modules, tooling, pilots and policy. See docs/ROADMAP.md.

fn print_help() {
    println!("etamil {} — the eTamil compiler", env!("CARGO_PKG_VERSION"));
    println!();
    println!("USAGE:");
    println!("    etamil [OPTIONS] <FILE>");
    println!("    cat program.qmz | etamil [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --vm               Run on the bytecode VM (default)");
    println!("    --server           Start the synchronous HTTP server");
    println!("    --async            Currently an alias for --server");
    println!("    --llvm             LLVM backend (requires --features llvm; Linux/macOS)");
    println!("    --host <HOST>      Server bind address (default: 127.0.0.1)");
    println!("    --port <PORT>      Server port (default: 8080)");
    println!("    -h, --help         Show this message");
    println!("    -V, --version      Show the version");
    println!();
    println!("EXAMPLES:");
    println!("    etamil --vm program.qmz");
    println!("    echo \"950000\" | etamil --vm examples/basic_samples/example.qmz");
    println!("    etamil --server --port 8080 examples/backend/hello_server.qmz");
}

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let mut use_vm = true;  // Default: use VM executor
    let mut use_http_server = false;
    let mut use_async_server = false;  // Backend milestone 2: New async server flag
    let mut server_host = "127.0.0.1".to_string();
    let mut server_port = 8080u16;
    let mut filename = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--llvm" => use_vm = false,
            "--vm" => use_vm = true,
            "--server" => use_http_server = true,
            "--async" => use_async_server = true,  // Backend milestone 2: Async server mode
            "--host" => {
                if i + 1 < args.len() {
                    server_host = args[i + 1].clone();
                    i += 1;
                }
            }
            "--port" => {
                if i + 1 < args.len() {
                    server_port = args[i + 1].parse().unwrap_or(8080);
                    i += 1;
                }
            }
            "--version" | "-V" => {
                println!("etamil {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            arg if !arg.starts_with('-') => filename = Some(arg.to_string()),
            // Unknown flags used to be ignored, which meant `etamil --version`
            // fell through to reading a program from stdin and appeared to hang.
            unknown => {
                eprintln!("✗ Unknown option: {}", unknown);
                eprintln!("   Run `etamil --help` to see the available options.");
                std::process::exit(2);
            }
        }
        i += 1;
    }
    
    // 1. Load the eTamil Source Code
    let input = if let Some(fname) = filename {
        fs::read_to_string(&fname)
            .unwrap_or_else(|_| panic!("Unable to read eTamil source file: {}", fname))
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)
            .expect("Unable to read from stdin");
        buffer
    };

    // 2. Lexical Analysis
    let tokens = match lexer::tokenize(&input) {
        Ok(tokens) => tokens,
        Err(errors) => {
            eprintln!("✗ Lexical analysis failed ({} error(s)):", errors.len());
            for error in &errors {
                eprintln!("  {}", error);
            }
            std::process::exit(1);
        }
    };
    println!("✓ Lexical analysis complete ({} tokens)", tokens.len());

    // 3. Parsing (AST Construction)
    let mut parser = parser::Parser::new(tokens.iter());
    let ast = parser.parse();
    println!("✓ Parsing complete ({} statements)\n", ast.len());

    // Backend milestone 2: Check if async server mode is enabled
    if use_async_server {
        // Intended to be the concurrent server. The Axum/Tokio modules exist
        // in src/http/ but are not compiled in, so this currently falls back
        // to the synchronous server. See docs/ROADMAP.md item 4.
        println!("=== eTamil HTTP Server (--async) ===");
        println!("⚠️  The async runtime is not wired up yet; this falls back to");
        println!("   the synchronous server. See docs/ROADMAP.md.");
        println!("🚀 Starting server on {}:{}\n", server_host, server_port);
        
        if let Err(e) = run_async_server(&server_host, server_port, ast).await {
            eprintln!("❌ Async server error: {}", e);
            std::process::exit(1);
        }
    } else if use_http_server {
        // === Backend milestone 1: SYNCHRONOUS HTTP SERVER MODE (MVP) ===
        println!("=== eTamil HTTP Server (Minimum Viable Backend) ===\n");
        
        let mut server = HttpServer::new(&server_host, server_port);
        
        // For MVP, register all statements as a single handler
        // Future: parse route definitions from DSL (वழि / path directives)
        server.register_route("GET", "/", ast.clone());
        server.register_route("POST", "/", ast.clone());
        server.register_route("PUT", "/", ast.clone());
        server.register_route("DELETE", "/", ast.clone());
        
        // Also register health check endpoint
        server.register_route("GET", "/health", vec![
            parser::Stmt::Print(parser::Expr::Number(rust_decimal::Decimal::from(200))),
        ]);

        // Start the server
        if let Err(e) = server.start() {
            eprintln!("❌ Server error: {}", e);
            std::process::exit(1);
        }
    } else if use_vm {
        // === VM EXECUTION PATH ===
        println!("=== eTamil VM Executor ===\n");
        
        // Compile AST to bytecode
        let bytecode = vm::bytecode::compiler::BytecodeCompiler::compile_statements(ast);
        println!("✓ Bytecode generated ({} instructions)", bytecode.len());
        println!("=== Execution Output ===\n");
        
        // Execute bytecode
        let mut vm = vm::VM::new();
        match vm.execute(bytecode) {
            Ok(_) => println!("\n✓ Execution completed successfully"),
            Err(e) => {
                eprintln!("✗ Runtime error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // === LLVM COMPILATION PATH (Legacy) ===
        #[cfg(feature = "llvm")]
        {
            println!("=== LLVM Code Generation ===");
            
            let mut compiler = codegen::Compiler::new();
            compiler.compile(ast);
            
            println!("\nGenerated LLVM IR:");
            compiler.dump_module();
            
            match compiler.emit_ir("output.ll") {
                Ok(_) => println!("\n✓ Successfully saved LLVM IR to output.ll"),
                Err(e) => eprintln!("✗ Error writing IR: {}", e),
            }
        }
        
        #[cfg(not(feature = "llvm"))]
        {
            eprintln!("❌ Error: LLVM backend is not available on this platform.");
            eprintln!("   Platform: {}", std::env::consts::OS);
            eprintln!("   Reason: LLVM feature not enabled during build");
            eprintln!();
            eprintln!("Please use one of the following modes instead:");
            eprintln!("  --vm              VM bytecode executor (default, recommended)");
            eprintln!("  --server          HTTP sync server");
            eprintln!("  --async           HTTP async server (production)");
            eprintln!();
            eprintln!("Examples:");
            eprintln!("  etamil --vm myprogram.etamil");
            eprintln!("  etamil --async --port 8080 api.etamil");
            std::process::exit(1);
        }
    }
}

// ============================================================================
// Backend milestone 2: ASYNC SERVER IMPLEMENTATION
// ============================================================================
// This function starts the high-performance async HTTP server
// Handles concurrent requests with graceful shutdown support
// ============================================================================

async fn run_async_server(
    host: &str,
    port: u16,
    ast: Vec<parser::Stmt>,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Integrate with async_mod.rs AsyncHttpServer. Until then this is
    // the synchronous server, and --async is an alias for --server.

    // For now, fallback to sync server to prevent breaking existing functionality
    let mut server = HttpServer::new(host, port);
    server.register_route("GET", "/", ast.clone());
    server.register_route("POST", "/", ast.clone());
    server.register_route("PUT", "/", ast.clone());
    server.register_route("DELETE", "/", ast.clone());
    server.register_route("GET", "/health", vec![
        parser::Stmt::Print(parser::Expr::Number(rust_decimal::Decimal::from(200))),
    ]);

    println!("✓ Async server started (using Backend milestone 1 handler for compatibility)\n");
    server.start()?;
    Ok(())
}

