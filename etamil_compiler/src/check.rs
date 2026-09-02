// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
//! Type checking.
//!
//! Type keywords used to be parsed and thrown away, so `சொல் x = 5;` was
//! accepted and nothing was ever enforced. Now that a declaration survives
//! parsing — and now that spans give an error somewhere to point — a declared
//! type is held to.
//!
//! The checker is deliberately narrow. It rejects what the author has said is
//! wrong, and stays silent everywhere else:
//!
//! - a value assigned to a declaration whose type it cannot be;
//! - a later assignment to a variable that was declared with a type;
//! - an argument that cannot be the type its parameter was declared;
//! - a `திரும்பு` that cannot be the declared return type.
//!
//! It does **not** invent constraints the language does not have. Arithmetic
//! on text is legal on purpose, because `உள்ளிடு` yields text and the VM
//! converts it when it is used as a number; flagging that would break the
//! language's own headline example.
//!
//! A signature is optional in every part, so a `செயல்` that declares nothing
//! is checked exactly as it was before. What is *not* optional is that a
//! declaration which is written gets held to: an unenforced type reads as a
//! guarantee, and silence is not approval — it is the absence of a claim.

use std::collections::HashMap;

use crate::parser::{DeclaredType, Expr, Param, Position, Stmt};

/// A type error, carrying the position of the name it concerns.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeError {
    pub line: usize,
    pub column: usize,
    pub name: String,
    pub declared: DeclaredType,
    /// What the value turned out to be, named the way a keyword would be.
    pub found: &'static str,
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "வரி {}, நெடுவரிசை {}: '{}' {} என அறிவிக்கப்பட்டது, ஆனால் {} வழங்கப்பட்டது  \
             (line {}, column {}: '{}' is declared {}, but was given {})",
            self.line,
            self.column,
            self.name,
            self.declared.name(),
            self.found,
            self.line,
            self.column,
            self.name,
            self.declared.name(),
            self.found
        )
    }
}

/// What an expression is known to be.
///
/// `Unknown` is not a type — it is the absence of a claim, and it satisfies
/// every declaration. Most expressions land here, which is the point: a
/// checker that guessed would reject working programs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Inferred {
    Number,
    Text,
    Boolean,
    Array,
    Record,
    Unknown,
}

impl Inferred {
    fn name(self) -> &'static str {
        match self {
            Inferred::Number => "ஒரு எண் (a number)",
            Inferred::Text => "ஒரு சொல் (a string)",
            Inferred::Boolean => "ஒரு ஈர்மம் (a boolean)",
            Inferred::Array => "ஒரு அணி (an array)",
            Inferred::Record => "ஒரு பொருள் (a record)",
            Inferred::Unknown => "something else",
        }
    }

    /// Can a value of this type stand where `declared` was promised?
    fn satisfies(self, declared: DeclaredType) -> bool {
        match declared {
            DeclaredType::Any => true,
            DeclaredType::Number => matches!(self, Inferred::Number | Inferred::Unknown),
            DeclaredType::Boolean => matches!(self, Inferred::Boolean | Inferred::Unknown),
            DeclaredType::Array => matches!(self, Inferred::Array | Inferred::Unknown),
            DeclaredType::Record => matches!(self, Inferred::Record | Inferred::Unknown),
            // Text accepts a number as well: every value in the language
            // renders as text, `&` concatenates whatever it is given, and
            // `உள்ளிடு` hands back text that is routinely compared with
            // numbers. Refusing `சொல் குறி = 1234;` would be a rule the rest
            // of the language does not follow.
            DeclaredType::Text => matches!(
                self,
                Inferred::Text | Inferred::Number | Inferred::Unknown
            ),
            // A date is ISO-8601 text — that is the representation the whole
            // language uses, because ISO text sorts chronologically.
            DeclaredType::Date => matches!(
                self,
                Inferred::Text | Inferred::Number | Inferred::Unknown
            ),
        }
    }
}

/// Check a program, reporting every type error rather than only the first.
///
/// A wrong declaration is usually one of several in a file, and stopping at
/// the first would make fixing them a sequence of recompiles.
pub fn check(statements: &[Stmt]) -> Result<(), Vec<TypeError>> {
    let mut checker = Checker {
        declared: HashMap::new(),
        signatures: HashMap::new(),
        returns: None,
        errors: Vec::new(),
    };
    // Signatures first, at every depth, so a call can be checked against a
    // function defined further down the file than the call is.
    collect_signatures(statements, &mut checker.signatures);
    checker.check_block(statements);

    if checker.errors.is_empty() {
        Ok(())
    } else {
        Err(checker.errors)
    }
}

/// What a `செயல்` promised: a type per parameter where it said, and a return
/// type where it said.
#[derive(Debug, Clone)]
struct Signature {
    params: Vec<Param>,
    returns: Option<DeclaredType>,
}

fn collect_signatures(statements: &[Stmt], into: &mut HashMap<String, Signature>) {
    for statement in statements {
        match statement {
            Stmt::FunctionDef {
                name,
                params,
                returns,
                body,
                ..
            } => {
                into.insert(
                    name.clone(),
                    Signature {
                        params: params.clone(),
                        returns: *returns,
                    },
                );
                collect_signatures(body, into);
            }
            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => {
                collect_signatures(then_branch, into);
                if let Some(branch) = else_branch {
                    collect_signatures(branch, into);
                }
            }
            Stmt::Loop { body, .. }
            | Stmt::ForEach { body, .. }
            | Stmt::DefineRoute { handler: body, .. }
            | Stmt::Schedule { body, .. } => collect_signatures(body, into),
            _ => {}
        }
    }
}

struct Checker {
    /// Types the program has committed to, by name.
    declared: HashMap<String, DeclaredType>,
    /// What each `செயல்` promised about its parameters and its result.
    signatures: HashMap<String, Signature>,
    /// The return type of the function being checked, if it declared one.
    returns: Option<(String, DeclaredType, Position)>,
    errors: Vec<TypeError>,
}

impl Checker {
    fn check_block(&mut self, statements: &[Stmt]) {
        for statement in statements {
            self.check_stmt(statement);
        }
    }

    fn check_stmt(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Assign { name, value, declared, at } => {
                self.check_assign(name, value, *declared, *at);
            }

            // A function body is checked in its own scope, so a name inside
            // cannot inherit an outer declaration for a variable that is not
            // the same variable. What it *does* start with is its parameters,
            // each carrying the type it was declared as — which is the point
            // of declaring one.
            Stmt::FunctionDef {
                name,
                params,
                returns,
                body,
                at,
            } => {
                let outer = std::mem::take(&mut self.declared);
                let outer_returns = self.returns.take();

                for param in params {
                    if let Some(declared) = param.declared {
                        self.declared.insert(param.name.clone(), declared);
                    }
                }
                self.returns = returns.map(|declared| (name.clone(), declared, *at));

                self.check_block(body);

                self.declared = outer;
                self.returns = outer_returns;
            }

            // A திரும்பு is held to the function's declared return type. Its
            // position is the function's name: the AST carries no span for a
            // statement here, and the declaration is the thing being broken.
            Stmt::Return(Some(value)) => {
                if let Some((name, declared, at)) = self.returns.clone() {
                    let found = self.infer(value);
                    if !found.satisfies(declared) {
                        self.errors.push(TypeError {
                            line: at.line,
                            column: at.column,
                            name,
                            declared,
                            found: found.name(),
                        });
                    }
                }
            }

            Stmt::If { then_branch, else_branch, .. } => {
                self.check_block(then_branch);
                if let Some(branch) = else_branch {
                    self.check_block(branch);
                }
            }
            Stmt::Loop { body, .. } => self.check_block(body),
            Stmt::ForEach { var, body, .. } => {
                // The loop variable takes whatever the collection holds, which
                // is not known here, so it carries no declaration.
                self.declared.remove(var);
                self.check_block(body);
            }
            Stmt::Expression(value) | Stmt::Print(value) => self.check_calls(value),
            Stmt::DefineRoute { handler, .. } => self.check_block(handler),
            Stmt::Schedule { body, .. } => self.check_block(body),

            // These bind a name to a value whose type the host decides — rows
            // from a query, text from a file — so any earlier declaration no
            // longer describes it.
            Stmt::FileRead { variable, .. }
            | Stmt::ReadCSV { variable, .. }
            | Stmt::GetRequestBody { variable }
            | Stmt::GetRequestParam { variable, .. }
            | Stmt::GetHeader { variable, .. } => {
                self.declared.remove(variable);
            }
            Stmt::DBQuery { result_var, .. } => {
                self.declared.remove(result_var);
            }

            // Nothing here can contradict a declaration.
            _ => {}
        }
    }

    /// Every call inside an expression, against what the function promised.
    ///
    /// An argument's error points at the *parameter*, because `Expr::Call`
    /// carries no span — and the parameter is where the promise was made, so
    /// it is a useful thing to be shown either way.
    fn check_calls(&mut self, expr: &Expr) {
        match expr {
            Expr::Call { name, args } => {
                for argument in args {
                    self.check_calls(argument);
                }

                if let Some(signature) = self.signatures.get(name).cloned() {
                    for (argument, param) in args.iter().zip(signature.params.iter()) {
                        let Some(declared) = param.declared else {
                            continue;
                        };
                        let found = self.infer(argument);
                        if !found.satisfies(declared) {
                            self.errors.push(TypeError {
                                line: param.at.line,
                                column: param.at.column,
                                name: param.name.clone(),
                                declared,
                                found: found.name(),
                            });
                        }
                    }
                }
            }
            Expr::BinaryOp { left, right, .. }
            | Expr::Comparison { left, right, .. }
            | Expr::Concat { left, right }
            | Expr::Logical { left, right, .. } => {
                self.check_calls(left);
                self.check_calls(right);
            }
            Expr::Not(inner) | Expr::Try(inner) => self.check_calls(inner),
            Expr::ArrayLiteral(items) => {
                for item in items {
                    self.check_calls(item);
                }
            }
            Expr::RecordLiteral(fields) => {
                for (_, value) in fields {
                    self.check_calls(value);
                }
            }
            Expr::Index { base, index } => {
                self.check_calls(base);
                self.check_calls(index);
            }
            Expr::Field { base, .. } => self.check_calls(base),
            Expr::Number(_)
            | Expr::String(_)
            | Expr::Boolean(_)
            | Expr::Null
            | Expr::Variable(_) => {}
        }
    }

    fn check_assign(
        &mut self,
        name: &str,
        value: &Expr,
        declared: Option<DeclaredType>,
        at: Position,
    ) {
        self.check_calls(value);

        // A declaration on this statement wins; failing that, one the program
        // made earlier still applies.
        let expected = declared.or_else(|| self.declared.get(name).copied());

        if let Some(expected) = expected {
            let found = self.infer(value);
            if !found.satisfies(expected) {
                self.errors.push(TypeError {
                    line: at.line,
                    column: at.column,
                    name: name.to_string(),
                    declared: expected,
                    found: found.name(),
                });
            }
        }

        // Recorded even when it was wrong, so the rest of the file is checked
        // against what the author said rather than against the mistake.
        if let Some(declared) = declared {
            self.declared.insert(name.to_string(), declared);
        }
    }

    fn infer(&self, expr: &Expr) -> Inferred {
        match expr {
            Expr::Number(_) => Inferred::Number,
            Expr::String(_) => Inferred::Text,
            Expr::Boolean(_) => Inferred::Boolean,
            Expr::ArrayLiteral(_) => Inferred::Array,
            Expr::RecordLiteral(_) => Inferred::Record,

            // Arithmetic yields a number whatever went in, because the VM
            // converts its operands.
            Expr::BinaryOp { .. } => Inferred::Number,
            Expr::Comparison { .. } | Expr::Logical { .. } | Expr::Not(_) => Inferred::Boolean,
            Expr::Concat { .. } => Inferred::Text,

            // இன்மை is the absent value and stands anywhere, so it makes no
            // claim rather than being its own type.
            Expr::Null => Inferred::Unknown,

            Expr::Variable(name) => match self.declared.get(name) {
                Some(DeclaredType::Number) => Inferred::Number,
                Some(DeclaredType::Text) | Some(DeclaredType::Date) => Inferred::Text,
                Some(DeclaredType::Boolean) => Inferred::Boolean,
                Some(DeclaredType::Array) => Inferred::Array,
                Some(DeclaredType::Record) => Inferred::Record,
                Some(DeclaredType::Any) | None => Inferred::Unknown,
            },

            // A call is whatever the function promised to return. Without a
            // declaration it is still unconstrained, which is most calls.
            Expr::Call { name, .. } => match self
                .signatures
                .get(name)
                .and_then(|signature| signature.returns)
            {
                Some(DeclaredType::Number) => Inferred::Number,
                Some(DeclaredType::Text) | Some(DeclaredType::Date) => Inferred::Text,
                Some(DeclaredType::Boolean) => Inferred::Boolean,
                Some(DeclaredType::Array) => Inferred::Array,
                Some(DeclaredType::Record) => Inferred::Record,
                Some(DeclaredType::Any) | None => Inferred::Unknown,
            },

            // Indexing a collection says nothing about what is inside it.
            Expr::Index { .. } | Expr::Field { .. } | Expr::Try(_) => Inferred::Unknown,
        }
    }
}
