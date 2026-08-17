// The modules live in the library target (src/lib.rs); this binary uses them
// from there rather than re-declaring them, which previously compiled the
// whole crate twice.
use std::io::{self, Read};
use std::env;
use std::path::Path;

use etamil_compiler::http::HttpServer;
use etamil_compiler::{module, parser, vm};
#[cfg(feature = "llvm")]
use etamil_compiler::codegen;

// main is deliberately *not* async.
//
// It used to be #[tokio::main], on the grounds that the server paths needed a
// runtime — but nothing here ever did: the VM is synchronous and the HTTP
// server is thread-per-request over std::net. The only tokio users in the
// crate are http/cache.rs and http/resilience.rs, neither of which is
// reachable from this binary, and whose tests carry their own runtime.
//
// It was not harmless. Being inside a runtime makes every blocking database
// driver panic — `Cannot start a runtime from within a runtime` — because
// they call block_on internally. That is exactly the arrangement
// docs/ARCHITECTURE.md chose blocking drivers *for*.
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

fn main() {
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
    
    // 1-3. Load, lex, parse, and resolve any இறக்கு imports.
    let loaded = match &filename {
        Some(fname) => module::load_file(Path::new(fname)),
        None => {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .expect("Unable to read from stdin");
            module::load_source(&buffer, Path::new("."))
        }
    };

    let ast = match loaded {
        Ok(ast) => ast,
        Err(message) => {
            eprintln!("✗ {}", message);
            std::process::exit(1);
        }
    };
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
        
        if let Err(e) = run_async_server(&server_host, server_port, ast) {
            eprintln!("❌ Async server error: {}", e);
            std::process::exit(1);
        }
    } else if use_http_server {
        // === Backend milestone 1: SYNCHRONOUS HTTP SERVER MODE (MVP) ===
        println!("=== eTamil HTTP Server (Minimum Viable Backend) ===\n");
        
        let mut server = HttpServer::new(&server_host, server_port);
        
        // For MVP, register all statements as a single handler
        // Future: parse route definitions from DSL (वழि / path directives)
        // Split the program into route definitions and everything else. The
        // remainder is a prelude — imports, functions, setup — compiled into
        // every handler so a route can call what the file defines.
        let (routes, prelude): (Vec<parser::Stmt>, Vec<parser::Stmt>) = ast
            .clone()
            .into_iter()
            .partition(|s| matches!(s, parser::Stmt::DefineRoute { .. }));

        if routes.is_empty() {
            // No வழி statements: the whole program answers every request,
            // which is how server programs behaved before routing existed.
            println!("ℹ️  No வழி routes found; serving the whole program on /");
            for method in ["GET", "POST", "PUT", "DELETE"] {
                server.register_route(method, "/", prelude.clone());
            }
        } else {
            for route in routes {
                if let parser::Stmt::DefineRoute { method, path, handler } = route {
                    let path = match path {
                        parser::Expr::String(literal) => literal,
                        other => {
                            eprintln!("✗ வழி needs a literal path, got {:?}", other);
                            std::process::exit(1);
                        }
                    };
                    let mut program = prelude.clone();
                    program.extend(handler);
                    server.register_route(&method, &path, program);
                }
            }
        }
        
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

            // The LLVM backend covers less of the language than the VM. If it
            // met anything it cannot build, refuse rather than emit IR that
            // silently computes something else.
            if !compiler.unsupported().is_empty() {
                eprintln!("✗ The LLVM backend cannot compile this program.");
                eprintln!("  Unsupported here, though the VM handles them:");
                let mut seen: Vec<&String> = compiler.unsupported().iter().collect();
                seen.sort();
                seen.dedup();
                for item in seen {
                    eprintln!("    - {}", item);
                }
                eprintln!();
                eprintln!("  Run it on the VM instead:  etamil --vm <file>");
                std::process::exit(1);
            }
            
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

fn run_async_server(
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

