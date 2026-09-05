// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
//! An interactive shell: type an expression, see what it comes to.
//!
//! The point is to make the language answerable without a file. Two things
//! have to survive between lines for that to be worth anything — the variables
//! a session has built up, and the functions it has defined — and they are
//! kept differently.
//!
//! Variables live in the VM, which is created once and reused, so `a = 5` on
//! one line is still 5 on the next. Functions live in the *bytecode*, which is
//! new every line, so their definitions are kept here and recompiled with each
//! input. Recompiling a `செயல்` costs nothing and registers it again; that is
//! why definitions are the only statements replayed. Replaying the rest would
//! run a session's side effects once per line.

use std::io::{self, BufRead, Write};
use std::path::Path;

use crate::parser::Stmt;
use crate::vm::{BytecodeCompiler, VM};

const BANNER: &str = "eTamil — வணக்கம். :help for help, :quit to leave.";

const HELP: &str = "\
  <expression>     evaluate and print it — `1 + 2`, `நீளம்(\"வணக்கம்\")`
  <statements>;    run them — `a = 5;` or a whole செயல் definition
  :vars            what this session has defined
  :help            this
  :quit            leave (Ctrl-D also works)

A line ending in an unclosed { keeps reading until the braces balance, so a
செயல் or a சுற்று can be typed across several lines.";

/// Run the shell until end of input.
pub fn run() -> ! {
    println!("{}", BANNER);

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    let mut vm = VM::new();
    // Definitions only: see the note at the top of this file.
    let mut definitions: Vec<Stmt> = Vec::new();
    let mut pending = String::new();

    loop {
        let continuing = !pending.is_empty();
        print!("{}", if continuing { "…  " } else { "» " });
        let _ = io::stdout().flush();

        let line = match lines.next() {
            Some(Ok(line)) => line,
            // Ctrl-D, or a pipe running dry: a session that ends is not an
            // error, and neither is one that ends mid-definition.
            _ => {
                println!();
                std::process::exit(0);
            }
        };

        if !continuing {
            match line.trim() {
                "" => continue,
                ":quit" | ":q" => std::process::exit(0),
                ":help" | ":h" => {
                    println!("{}", HELP);
                    continue;
                }
                ":vars" => {
                    show_session(&vm, &definitions);
                    continue;
                }
                _ => {}
            }
        }

        pending.push_str(&line);
        pending.push('\n');

        // An unclosed brace means the author is still typing. Braces inside a
        // string are not braces, which is why this counts through quotes.
        if open_braces(&pending) > 0 {
            continue;
        }

        let source = std::mem::take(&mut pending);
        evaluate(&mut vm, &mut definitions, &source);
    }
}

/// How many `{` are still unclosed, ignoring any inside a string.
fn open_braces(source: &str) -> i32 {
    let mut depth = 0;
    let mut in_string = false;
    let mut escaped = false;

    for ch in source.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {}
        }
    }
    depth.max(0)
}

/// Compile one input against the session's definitions and run it.
fn evaluate(vm: &mut VM, definitions: &mut Vec<Stmt>, source: &str) {
    // load_source rather than the parser directly, so that இறக்கு works: an
    // import is resolved to the statements it brings in.
    let parsed = match crate::module::load_source(source, Path::new(".")) {
        Ok(parsed) => parsed,
        Err(as_statements) => {
            // `1 + 2` is not a statement, and in a file that is right — there
            // is nothing to do with the answer. Here there is: show it. So a
            // line the parser rejects is offered back to it as a question
            // before being called a mistake.
            let asked = format!("அச்சு ({});", source.trim().trim_end_matches(';'));
            match crate::module::load_source(&asked, Path::new(".")) {
                Ok(parsed) => parsed,
                // Report what was wrong with what they typed, not with the
                // question this made of it.
                Err(_) => {
                    eprintln!("✗ {}", as_statements);
                    return;
                }
            }
        }
    };

    // A bare expression is a question, so answer it. Anything else is an
    // instruction, and instructions are carried out quietly.
    let statements: Vec<Stmt> = if let [Stmt::Expression(expr)] = parsed.as_slice() {
        vec![Stmt::Print(expr.clone())]
    } else {
        parsed.clone()
    };

    let mut program = definitions.clone();
    program.extend(statements);

    let bytecode = BytecodeCompiler::compile_statements(program);

    // The VM is reused, so it resumes where the last program ended; a new
    // program starts at its own beginning.
    vm.instruction_pointer = 0;
    if let Err(e) = vm.execute(bytecode) {
        eprintln!("✗ {}", e);
        // A failed line still defined nothing, so the session is unchanged.
        return;
    }

    // Remember what this line defined, for the next one.
    for statement in parsed {
        if matches!(statement, Stmt::FunctionDef { .. }) {
            definitions.push(statement);
        }
    }
}

/// What the session holds: its variables, and the functions it can call.
fn show_session(vm: &VM, definitions: &[Stmt]) {
    let mut names: Vec<&String> = vm.variables.keys().collect();
    names.sort();

    if names.is_empty() {
        println!("  (no variables yet)");
    } else {
        for name in names {
            let value = &vm.variables[name];
            println!("  {} = {}", name, value);
        }
    }

    let functions: Vec<&str> = definitions
        .iter()
        .filter_map(|statement| match statement {
            Stmt::FunctionDef { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();

    if !functions.is_empty() {
        println!("  செயல்: {}", functions.join(", "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn braces_inside_a_string_are_not_braces() {
        // Otherwise `அச்சு "{";` would leave the shell waiting forever for a
        // closing brace the author never meant to open.
        assert_eq!(open_braces(r#"அச்சு "{";"#), 0);
        assert_eq!(open_braces(r#"அச்சு "\"{";"#), 0);
        assert_eq!(open_braces("செயல் அ() {"), 1);
        assert_eq!(open_braces("செயல் அ() { திரும்பு 1; }"), 0);
    }

    #[test]
    fn a_closed_definition_over_several_lines_balances() {
        let typed = "செயல் இரட்டை(எ) {\n    திரும்பு எ * 2;\n}\n";
        assert_eq!(open_braces(typed), 0);
    }

    #[test]
    fn an_unbalanced_close_does_not_go_negative() {
        // A stray `}` should not make the shell think it is owed braces.
        assert_eq!(open_braces("}"), 0);
    }
}
