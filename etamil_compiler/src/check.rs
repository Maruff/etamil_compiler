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
//! - a `திரும்பு` that cannot be the declared return type;
//! - a name bound with `நிலை` that is assigned again, or has part of it
//!   changed.
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

use std::collections::{HashMap, HashSet};

use crate::parser::{DeclaredType, Expr, Param, Position, ShapeField, Stmt};

/// A type error, carrying the position of the name it concerns.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeError {
    pub line: usize,
    pub column: usize,
    pub name: String,
    pub problem: Problem,
}

/// What was wrong.
#[derive(Debug, Clone, PartialEq)]
pub enum Problem {
    /// A value that cannot be the type its name was declared.
    Mismatch {
        declared: DeclaredType,
        /// What the value turned out to be, named the way a keyword would be.
        found: String,
    },
    /// Anything else, already worded in both languages.
    Said { tamil: String, english: String },
}

impl TypeError {
    fn mismatch<N: AsRef<str>>(
        at: Position,
        name: N,
        declared: DeclaredType,
        found: String,
    ) -> Self {
        TypeError {
            line: at.line,
            column: at.column,
            name: name.as_ref().to_string(),
            problem: Problem::Mismatch { declared, found },
        }
    }

    fn said(at: Position, name: &str, tamil: String, english: String) -> Self {
        TypeError {
            line: at.line,
            column: at.column,
            name: name.to_string(),
            problem: Problem::Said { tamil, english },
        }
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.problem {
            Problem::Mismatch { declared, found } => write!(
                f,
                "வரி {}, நெடுவரிசை {}: '{}' {} என அறிவிக்கப்பட்டது, ஆனால் {} வழங்கப்பட்டது                   (line {}, column {}: '{}' is declared {}, but was given {})",
                self.line,
                self.column,
                self.name,
                declared.name(),
                found,
                self.line,
                self.column,
                self.name,
                declared.name(),
                found
            ),
            Problem::Said { tamil, english } => write!(
                f,
                "வரி {}, நெடுவரிசை {}: {}  (line {}, column {}: {})",
                self.line, self.column, tamil, self.line, self.column, english
            ),
        }
    }
}

/// What an expression is known to be.
///
/// `Unknown` is not a type — it is the absence of a claim, and it satisfies
/// every declaration. Most expressions land here, which is the point: a
/// checker that guessed would reject working programs.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Inferred {
    Number,
    Text,
    Boolean,
    Array,
    Record,
    Function,
    /// A record made as this வடிவம்.
    Shape(String),
    Unknown,
}

impl Inferred {
    fn name(&self) -> String {
        let fixed = match self {
            Inferred::Number => "ஒரு எண் (a number)",
            Inferred::Text => "ஒரு சொல் (a string)",
            Inferred::Boolean => "ஒரு ஈர்மம் (a boolean)",
            Inferred::Array => "ஒரு அணி (an array)",
            Inferred::Record => "ஒரு பொருள் (a record)",
            Inferred::Function => "ஒரு செயல் (a function)",
            Inferred::Shape(shape) => return format!("ஒரு {} (a {} record)", shape, shape),
            Inferred::Unknown => "something else",
        };
        fixed.to_string()
    }

    /// What a value is known to be because its name or its function said so.
    fn from_declared(declared: Option<&DeclaredType>) -> Self {
        match declared {
            Some(DeclaredType::Number) => Inferred::Number,
            Some(DeclaredType::Text) | Some(DeclaredType::Date) => Inferred::Text,
            Some(DeclaredType::Boolean) => Inferred::Boolean,
            Some(DeclaredType::Array) => Inferred::Array,
            Some(DeclaredType::Record) => Inferred::Record,
            Some(DeclaredType::Function) => Inferred::Function,
            Some(DeclaredType::Shape(shape)) => Inferred::Shape(shape.clone()),
            Some(DeclaredType::Any) | None => Inferred::Unknown,
        }
    }

    /// Can a value of this type stand where `declared` was promised?
    fn satisfies(&self, declared: &DeclaredType) -> bool {
        match declared {
            DeclaredType::Any => true,
            DeclaredType::Number => matches!(self, Inferred::Number | Inferred::Unknown),
            DeclaredType::Boolean => matches!(self, Inferred::Boolean | Inferred::Unknown),
            DeclaredType::Array => matches!(self, Inferred::Array | Inferred::Unknown),
            // A shaped record is still a record.
            DeclaredType::Record => {
                matches!(
                    self,
                    Inferred::Record | Inferred::Shape(_) | Inferred::Unknown
                )
            }
            DeclaredType::Function => matches!(self, Inferred::Function | Inferred::Unknown),
            // Only a record made as that shape. A plain record literal with
            // the right fields is not one: `கடன்{…}` says which fields must
            // be there, and `{…}` says nothing.
            DeclaredType::Shape(shape) => match self {
                Inferred::Shape(found) => found == shape,
                Inferred::Unknown => true,
                _ => false,
            },
            // Text accepts a number as well: every value in the language
            // renders as text, `&` concatenates whatever it is given, and
            // `உள்ளிடு` hands back text that is routinely compared with
            // numbers. Refusing `சொல் குறி = 1234;` would be a rule the rest
            // of the language does not follow.
            DeclaredType::Text => {
                matches!(self, Inferred::Text | Inferred::Number | Inferred::Unknown)
            }
            // A date is ISO-8601 text — that is the representation the whole
            // language uses, because ISO text sorts chronologically.
            DeclaredType::Date => {
                matches!(self, Inferred::Text | Inferred::Number | Inferred::Unknown)
            }
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
        shapes: HashMap::new(),
        returns: None,
        scope: Scope::default(),
        errors: Vec::new(),
    };
    // Signatures first, at every depth, so a call can be checked against a
    // function defined further down the file than the call is. Shapes the
    // same, so a shape can be used above the line that declares it.
    collect_signatures(statements, &mut checker.signatures);
    checker.collect_shapes(statements);
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

/// What a `வடிவம்` declared.
#[derive(Debug, Clone)]
struct ShapeInfo {
    fields: Vec<ShapeField>,
    /// Each method's signature, and whether it is called on a record. The
    /// signature includes `இது` when it takes one.
    methods: HashMap<String, (Signature, bool)>,
    at: Position,
}

impl ShapeInfo {
    fn field(&self, name: &str) -> Option<&ShapeField> {
        self.fields.iter().find(|field| field.name == name)
    }

    fn field_names(&self) -> String {
        self.fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
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
                        returns: returns.clone(),
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

/// Which names in one scope are fixed with `நிலை`, and which are ordinary.
///
/// A scope is the whole program's top level, or one function body. There is no
/// block scope in the language — a name assigned inside an `எனில்` is the same
/// variable after it — so there is none here either.
#[derive(Default, Clone)]
struct Scope {
    /// Bound with `நிலை`, and where.
    fixed: HashMap<String, Position>,
    /// Assigned as an ordinary variable at least once.
    assigned: HashSet<String>,
}

struct Checker {
    /// Types the program has committed to, by name.
    declared: HashMap<String, DeclaredType>,
    /// What each `செயல்` promised about its parameters and its result.
    signatures: HashMap<String, Signature>,
    /// Every `வடிவம்` the program declares, by name.
    shapes: HashMap<String, ShapeInfo>,
    /// The return type of the function being checked, if it declared one.
    returns: Option<(String, DeclaredType, Position)>,
    /// What `நிலை` has fixed in the scope being checked.
    scope: Scope,
    errors: Vec<TypeError>,
}

impl Checker {
    fn check_block(&mut self, statements: &[Stmt]) {
        for statement in statements {
            self.check_stmt(statement);
        }
    }

    /// Every shape at the top level, and a refusal for any that is not.
    ///
    /// Only the top level, because a shape is a name the whole program shares
    /// — like a named `செயல்`, but without the excuse of recursion to be
    /// declared anywhere else. One inside an `எனில்` would exist or not
    /// depending on a branch that has not been taken yet.
    fn collect_shapes(&mut self, statements: &[Stmt]) {
        for statement in statements {
            let Stmt::ShapeDef {
                name,
                fields,
                methods,
                at,
            } = statement
            else {
                continue;
            };

            if let Some(first) = self.shapes.get(name) {
                let line = first.at.line;
                self.errors.push(TypeError::said(
                    *at,
                    name,
                    format!("'{}' ஏற்கனவே வரி {} இல் ஒரு வடிவம்", name, line),
                    format!("'{}' is already a shape, on line {}", name, line),
                ));
                continue;
            }
            if self.signatures.contains_key(name) {
                self.errors.push(TypeError::said(
                    *at,
                    name,
                    format!("'{}' ஒரு வடிவமும் செயலும் ஆக முடியாது", name),
                    format!(
                        "'{}' cannot be both a shape and a function: `{}(…)` has to mean one of them",
                        name, name
                    ),
                ));
            }

            let mut seen: HashSet<&str> = HashSet::new();
            for field in fields {
                if !seen.insert(&field.name) {
                    self.errors.push(TypeError::said(
                        field.at,
                        &field.name,
                        format!("'{}' என்ற புலம் இருமுறை உள்ளது", field.name),
                        format!("the field '{}' is declared twice in '{}'", field.name, name),
                    ));
                }
            }

            let mut signatures = HashMap::new();
            for method in methods {
                if seen.contains(method.name.as_str()) {
                    self.errors.push(TypeError::said(
                        method.at,
                        &method.name,
                        format!("'{}' புலமும் முறையும் ஆக முடியாது", method.name),
                        format!(
                            "'{}' cannot be both a field and a method of '{}'",
                            method.name, name
                        ),
                    ));
                }
                if signatures.contains_key(&method.name) {
                    self.errors.push(TypeError::said(
                        method.at,
                        &method.name,
                        format!("'{}' என்ற முறை இருமுறை உள்ளது", method.name),
                        format!(
                            "the method '{}' is written twice in '{}'",
                            method.name, name
                        ),
                    ));
                }
                let takes_self = method.takes_self();
                let mut params = method.params.clone();
                if takes_self {
                    params[0].declared = Some(DeclaredType::Shape(name.clone()));
                }
                signatures.insert(
                    method.name.clone(),
                    (
                        Signature {
                            params,
                            returns: method.returns.clone(),
                        },
                        takes_self,
                    ),
                );
            }

            self.shapes.insert(
                name.clone(),
                ShapeInfo {
                    fields: fields.clone(),
                    methods: signatures,
                    at: *at,
                },
            );
        }

        self.refuse_nested_shapes(statements, true);
    }

    fn refuse_nested_shapes(&mut self, statements: &[Stmt], top: bool) {
        for statement in statements {
            match statement {
                Stmt::ShapeDef { name, at, .. } if !top => {
                    self.errors.push(TypeError::said(
                        *at,
                        name,
                        "வடிவம் நிரலின் மேல் மட்டத்தில் மட்டுமே அறிவிக்கப்படலாம்".to_string(),
                        format!(
                            "a shape is declared at the top level of a program, \
                             not inside a block or a function: move '{}' out",
                            name
                        ),
                    ));
                }
                Stmt::ShapeDef { methods, .. } => {
                    for method in methods {
                        self.refuse_nested_shapes(&method.body, false);
                    }
                }
                Stmt::FunctionDef { body, .. }
                | Stmt::Loop { body, .. }
                | Stmt::ForEach { body, .. }
                | Stmt::DefineRoute { handler: body, .. }
                | Stmt::Schedule { body, .. } => self.refuse_nested_shapes(body, false),
                Stmt::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    self.refuse_nested_shapes(then_branch, false);
                    if let Some(branch) = else_branch {
                        self.refuse_nested_shapes(branch, false);
                    }
                }
                _ => {}
            }
        }
    }

    /// A type naming a shape must name one that exists.
    fn check_type(&mut self, declared: Option<&DeclaredType>, at: Position) {
        if let Some(DeclaredType::Shape(shape)) = declared
            && !self.shapes.contains_key(shape)
        {
            self.errors.push(TypeError::said(
                at,
                shape,
                format!("'{}' என்ற வடிவம் இல்லை", shape),
                format!("there is no shape named '{}'", shape),
            ));
        }
    }

    fn check_stmt(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Assign {
                name,
                value,
                declared,
                at,
                immutable,
            } => {
                self.check_type(declared.as_ref(), *at);
                self.check_assign(name, value, declared.clone(), *at, *immutable);
                if *immutable {
                    self.fix(name, *at);
                } else {
                    self.rebind(name, Some(*at));
                }
            }

            Stmt::FunctionDef {
                name,
                params,
                returns,
                body,
                at,
            } => self.check_function(name, params, returns.clone(), body, *at),

            // A shape's field types must exist, and each method is a function
            // whose `இது` is a record of this shape that it cannot change —
            // Rust's `&self`. A method that means to change a record returns
            // a new one: `கடன்{அசல்: புதியது, ..இது}`.
            Stmt::ShapeDef {
                name,
                fields,
                methods,
                ..
            } => {
                for field in fields {
                    self.check_type(field.declared.as_ref(), field.at);
                }
                for method in methods {
                    let mut params = method.params.clone();
                    if method.takes_self() {
                        let own = DeclaredType::Shape(name.clone());
                        match &params[0].declared {
                            Some(declared) if *declared != own => {
                                self.errors.push(TypeError::said(
                                    params[0].at,
                                    &params[0].name,
                                    format!("இது எப்போதும் ஒரு '{}'", name),
                                    format!(
                                        "இது is always the '{}' the method was called on",
                                        name
                                    ),
                                ));
                            }
                            _ => {}
                        }
                        params[0].declared = Some(own);
                        params[0].immutable = true;
                    }
                    self.check_function(
                        &format!("{}.{}", name, method.name),
                        &params,
                        method.returns.clone(),
                        &method.body,
                        method.at,
                    );
                }
            }

            // A திரும்பு is held to the function's declared return type. Its
            // position is the function's name: the AST carries no span for a
            // statement here, and the declaration is the thing being broken.
            Stmt::Return(Some(value)) => {
                // A செயல் returned as a value is checked like any other.
                self.check_calls(value);
                if let Some((name, declared, at)) = self.returns.clone() {
                    let found = self.infer(value);
                    if !found.satisfies(&declared) {
                        self.errors
                            .push(TypeError::mismatch(at, name, declared, found.name()));
                    }
                }
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.check_calls(condition);
                self.check_block(then_branch);
                if let Some(branch) = else_branch {
                    self.check_block(branch);
                }
            }
            Stmt::Loop { condition, body } => {
                self.check_calls(condition);
                self.check_block(body);
            }
            Stmt::ForEach {
                var,
                collection,
                body,
            } => {
                self.check_calls(collection);
                // The loop variable takes whatever the collection holds, which
                // is not known here, so it carries no declaration.
                self.declared.remove(var);
                self.rebind(var, None);
                self.check_block(body);
            }
            // Changing one element or one field changes the value the name
            // holds, so a நிலை name refuses it as surely as an assignment.
            Stmt::SetIndex {
                name,
                index,
                value,
                at,
            } => {
                self.check_calls(index);
                self.check_calls(value);
                self.change_part(name, *at, format!("{}[…]", name));
                // `க["அசல்"] = …` names its field as plainly as `க.அசல்` does.
                if let Expr::String(field) = index {
                    self.check_field_set(name, field, value, *at);
                }
            }
            Stmt::SetField {
                name,
                field,
                value,
                at,
            } => {
                self.check_calls(value);
                self.change_part(name, *at, format!("{}.{}", name, field));
                self.check_field_set(name, field, value, *at);
            }
            Stmt::Input(Expr::Variable(name)) => self.rebind(name, None),
            Stmt::Expression(value) | Stmt::Print(value) => self.check_calls(value),
            // Each handler runs as its own program after the prelude, so it
            // sees what the prelude fixed and keeps whatever it fixes itself.
            Stmt::DefineRoute { handler: body, .. } | Stmt::Schedule { body, .. } => {
                let outer_scope = self.scope.clone();
                self.check_block(body);
                self.scope = outer_scope;
            }

            // These bind a name to a value whose type the host decides — rows
            // from a query, text from a file — so any earlier declaration no
            // longer describes it.
            Stmt::FileRead { variable, .. }
            | Stmt::ReadCSV { variable, .. }
            | Stmt::GetRequestBody { variable }
            | Stmt::GetRequestParam { variable, .. }
            | Stmt::GetHeader { variable, .. } => {
                self.declared.remove(variable);
                self.rebind(variable, None);
            }
            Stmt::DBQuery { result_var, .. } => {
                self.declared.remove(result_var);
                self.rebind(result_var, None);
            }

            // Nothing here can contradict a declaration.
            _ => {}
        }
    }

    /// `க.புலம் = மதிப்பு;` on a name declared as a shape: the field must be
    /// one the shape has, and the value one it can hold.
    fn check_field_set(&mut self, name: &str, field: &str, value: &Expr, at: Position) {
        let Some(DeclaredType::Shape(shape)) = self.declared.get(name).cloned() else {
            return;
        };
        let Some(info) = self.shapes.get(&shape).cloned() else {
            return;
        };
        match info.field(field) {
            Some(declared) => {
                if let Some(declared) = &declared.declared {
                    let found = self.infer(value);
                    if !found.satisfies(declared) {
                        self.errors.push(TypeError::mismatch(
                            at,
                            format!("{}.{}", shape, field),
                            declared.clone(),
                            found.name(),
                        ));
                    }
                }
            }
            None => self.no_such_field(&shape, &info, field, at),
        }
    }

    fn no_such_field(&mut self, shape: &str, info: &ShapeInfo, field: &str, at: Position) {
        if info.methods.contains_key(field) {
            self.errors.push(TypeError::said(
                at,
                field,
                format!("'{}' ஒரு முறை; அதை அழைக்கவும்", field),
                format!(
                    "'{}' is a method of '{}', not a field: call it, as .{}(…)",
                    field, shape, field
                ),
            ));
            return;
        }
        self.errors.push(TypeError::said(
            at,
            field,
            format!("'{}' இல் '{}' என்ற புலம் இல்லை", shape, field),
            format!(
                "'{}' has no field '{}'; its fields are: {}",
                shape,
                field,
                info.field_names()
            ),
        ));
    }

    /// A function body, named or written as a value.
    ///
    /// Checked in its own scope, so a name inside cannot inherit an outer
    /// declaration for a variable that is not the same variable. What it
    /// *does* start with is its parameters, each carrying the type it was
    /// declared as — which is the point of declaring one.
    fn check_function(
        &mut self,
        name: &str,
        params: &[Param],
        returns: Option<DeclaredType>,
        body: &[Stmt],
        at: Position,
    ) {
        for param in params {
            self.check_type(param.declared.as_ref(), param.at);
            self.refuse_shape_name(&param.name, param.at);
        }
        self.check_type(returns.as_ref(), at);

        let outer = std::mem::take(&mut self.declared);
        let outer_returns = self.returns.take();
        // Assigning to an outer name inside a function makes a local, so an
        // outer நிலை does not reach in: the function cannot change it, which
        // is what it promised. A செயல் written as a value carries copies of
        // the locals it uses, and assigning to one changes only its copy.
        let outer_scope = std::mem::take(&mut self.scope);

        for param in params {
            if let Some(declared) = &param.declared {
                self.declared.insert(param.name.clone(), declared.clone());
            }
            if param.immutable {
                self.scope.fixed.insert(param.name.clone(), param.at);
            } else {
                self.scope.assigned.insert(param.name.clone());
            }
        }
        self.returns = returns.map(|declared| (name.to_string(), declared, at));

        self.check_block(body);

        self.declared = outer;
        self.returns = outer_returns;
        self.scope = outer_scope;
    }

    /// A variable cannot take a shape's name: `கடன்(x)` and `கடன்.புதிது()`
    /// have to mean the shape, wherever they are written.
    fn refuse_shape_name(&mut self, name: &str, at: Position) {
        if self.shapes.contains_key(name) {
            self.errors.push(TypeError::said(
                at,
                name,
                format!("'{}' ஒரு வடிவத்தின் பெயர்; மாறிக்கு வேறு பெயர் தேவை", name),
                format!(
                    "'{}' is the name of a shape, so a variable cannot have it too",
                    name
                ),
            ));
        }
    }

    /// `நிலை பெயர் = …;` — fix a name for the rest of its scope.
    ///
    /// Once per scope. Rust would allow a second `let` to shadow the first;
    /// with no block scope here, a second `நிலை` would be the same name
    /// changing value halfway down a function, which is the thing it exists to
    /// rule out. The same goes for a name that was an ordinary variable first.
    fn fix(&mut self, name: &str, at: Position) {
        self.refuse_shape_name(name, at);
        if let Some(first) = self.scope.fixed.get(name).copied() {
            self.errors.push(TypeError::said(
                at,
                name,
                format!(
                    "'{}' ஏற்கனவே வரி {} இல் நிலையாக அறிவிக்கப்பட்டது",
                    name, first.line
                ),
                format!(
                    "'{}' is already bound with நிலை on line {}; a நிலை name is bound once",
                    name, first.line
                ),
            ));
            return;
        }
        if self.scope.assigned.contains(name) {
            self.errors.push(TypeError::said(
                at,
                name,
                format!("'{}' ஏற்கனவே ஒரு மாறியாகப் பயன்பட்டது", name),
                format!(
                    "'{}' was already assigned as an ordinary variable here; \
                     a name is either நிலை or not, for the whole of its scope",
                    name
                ),
            ));
            return;
        }
        self.scope.fixed.insert(name.to_string(), at);
    }

    /// Bind a name again: an assignment, a loop variable, a query's rows.
    ///
    /// `at` is where it happens, when the statement knows. Those that do not
    /// carry a position point at the நிலை instead, which is the other half of
    /// the disagreement.
    fn rebind(&mut self, name: &str, at: Option<Position>) {
        if let Some(at) = at.or_else(|| self.shapes.get(name).map(|info| info.at)) {
            self.refuse_shape_name(name, at);
        }
        match self.scope.fixed.get(name).copied() {
            Some(declared) => self.errors.push(TypeError::said(
                at.unwrap_or(declared),
                name,
                format!(
                    "'{}' நிலையானது (வரி {}); அதற்கு மீண்டும் மதிப்பிட முடியாது",
                    name, declared.line
                ),
                format!(
                    "cannot assign twice to '{}': it is bound with நிலை on line {}",
                    name, declared.line
                ),
            )),
            None => {
                self.scope.assigned.insert(name.to_string());
            }
        }
    }

    /// `பெயர்[i] = …;` or `பெயர்.புலம் = …;` on a name fixed with நிலை.
    ///
    /// Values are copied, never shared, so there is no second name through
    /// which a fixed value could be reached and changed. Refusing this and
    /// rebinding is therefore all it takes for நிலை to hold the whole value
    /// still, not only the name.
    fn change_part(&mut self, name: &str, at: Position, shown: String) {
        if let Some(declared) = self.scope.fixed.get(name).copied() {
            self.errors.push(TypeError::said(
                at,
                name,
                format!(
                    "'{}' நிலையானது (வரி {}); அதன் பகுதியையும் மாற்ற முடியாது",
                    name, declared.line
                ),
                format!(
                    "cannot change '{}': '{}' is bound with நிலை on line {}, \
                     so its elements and fields are fixed too",
                    shown, name, declared.line
                ),
            ));
        }
    }

    /// Arguments against the parameters they fill.
    ///
    /// An argument's error points at the *parameter*, because a call carries
    /// no span of its own for each argument — and the parameter is where the
    /// promise was made, so it is a useful thing to be shown either way.
    fn check_arguments(&mut self, args: &[Expr], params: &[Param]) {
        for (argument, param) in args.iter().zip(params.iter()) {
            let Some(declared) = &param.declared else {
                continue;
            };
            let found = self.infer(argument);
            if !found.satisfies(declared) {
                self.errors.push(TypeError::mismatch(
                    param.at,
                    param.name.clone(),
                    declared.clone(),
                    found.name(),
                ));
            }
        }
    }

    /// Every call inside an expression, against what the function promised,
    /// and every field and method against the shape it is read from.
    fn check_calls(&mut self, expr: &Expr) {
        match expr {
            Expr::Call { name, args } => {
                for argument in args {
                    self.check_calls(argument);
                }

                if let Some(signature) = self.signatures.get(name).cloned() {
                    self.check_arguments(args, &signature.params);
                } else if self.shapes.contains_key(name) && args.len() != 1 {
                    // `கடன்(பதிவு)` — the one thing a shape's name called does.
                    let position = self.shapes[name].at;
                    self.errors.push(TypeError::said(
                        position,
                        name,
                        format!("'{}(…)' க்கு ஒரு பொருள் மட்டுமே தேவை", name),
                        format!(
                            "'{}(…)' makes a {} out of one record, and takes exactly one argument",
                            name, name
                        ),
                    ));
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
            Expr::Field { base, name, at } => {
                self.check_calls(base);
                if let Inferred::Shape(shape) = self.infer(base)
                    && let Some(info) = self.shapes.get(&shape).cloned()
                    && info.field(name).is_none()
                {
                    self.no_such_field(&shape, &info, name, *at);
                }
            }
            Expr::MethodCall {
                receiver,
                name,
                args,
                at,
            } => {
                self.check_calls(receiver);
                for argument in args {
                    self.check_calls(argument);
                }
                self.check_method_call(receiver, name, args, *at);
            }
            Expr::ShapeLiteral {
                shape,
                fields,
                base,
                at,
            } => self.check_shape_literal(shape, fields, base.as_deref(), *at),
            Expr::Lambda {
                params,
                returns,
                body,
                at,
            } => self.check_function("செயல்", params, returns.clone(), body, *at),
            Expr::CallValue { callee, args } => {
                self.check_calls(callee);
                for argument in args {
                    self.check_calls(argument);
                }
            }
            Expr::Number(_)
            | Expr::String(_)
            | Expr::Boolean(_)
            | Expr::Null
            | Expr::Variable(_) => {}
        }
    }

    /// Is `name` a shape here, rather than a variable that happens to share
    /// its name? A declared variable wins, as it does at runtime.
    fn names_shape(&self, receiver: &Expr) -> Option<String> {
        match receiver {
            Expr::Variable(name)
                if self.shapes.contains_key(name) && !self.declared.contains_key(name) =>
            {
                Some(name.clone())
            }
            _ => None,
        }
    }

    fn check_method_call(&mut self, receiver: &Expr, name: &str, args: &[Expr], at: Position) {
        // `கடன்.புதிது(…)` — a function of the shape, called on the shape. A
        // method that takes இது can be called this way too, with the record
        // as its first argument, as Rust allows `Loan::interest(&loan, 2)`.
        if let Some(shape) = self.names_shape(receiver) {
            let info = self.shapes[&shape].clone();
            match info.methods.get(name) {
                Some((signature, _)) => self.check_arguments(args, &signature.params),
                None => self.errors.push(TypeError::said(
                    at,
                    name,
                    format!("'{}' இல் '{}' என்ற செயல் இல்லை", shape, name),
                    format!("'{}' has no function '{}'", shape, name),
                )),
            }
            return;
        }

        let Inferred::Shape(shape) = self.infer(receiver) else {
            return;
        };
        let Some(info) = self.shapes.get(&shape).cloned() else {
            return;
        };
        match info.methods.get(name) {
            Some((signature, true)) => self.check_arguments(args, &signature.params[1..]),
            Some((_, false)) => self.errors.push(TypeError::said(
                at,
                name,
                format!("'{}' ஒரு பதிவின் மேல் அழைக்கப்படுவதில்லை", name),
                format!(
                    "'{}' takes no இது, so it is not called on a record: call it as {}.{}(…)",
                    name, shape, name
                ),
            )),
            // No method of that name: a field holding a function is called
            // instead, at runtime. A field declared as anything else cannot be.
            None => match info.field(name) {
                Some(field)
                    if matches!(
                        field.declared,
                        None | Some(DeclaredType::Function) | Some(DeclaredType::Any)
                    ) => {}
                Some(_) => self.errors.push(TypeError::said(
                    at,
                    name,
                    format!("'{}.{}' ஒரு செயல் அல்ல", shape, name),
                    format!(
                        "'{}.{}' is a field that does not hold a function, so it cannot be called",
                        shape, name
                    ),
                )),
                None => self.errors.push(TypeError::said(
                    at,
                    name,
                    format!("'{}' இல் '{}' என்ற முறை இல்லை", shape, name),
                    format!("'{}' has no method or field '{}'", shape, name),
                )),
            },
        }
    }

    /// `கடன்{அசல்: …, வீதம்: …}`: every field the shape has, none it does not,
    /// each holding what it was declared to — or `..பழையது` for the rest.
    fn check_shape_literal(
        &mut self,
        shape: &str,
        fields: &[(String, Expr)],
        base: Option<&Expr>,
        at: Position,
    ) {
        for (_, value) in fields {
            self.check_calls(value);
        }
        if let Some(base) = base {
            self.check_calls(base);
        }

        let Some(info) = self.shapes.get(shape).cloned() else {
            self.errors.push(TypeError::said(
                at,
                shape,
                format!("'{}' என்ற வடிவம் இல்லை", shape),
                format!("there is no shape named '{}'", shape),
            ));
            return;
        };

        let mut written: HashSet<&str> = HashSet::new();
        for (field, value) in fields {
            if !written.insert(field) {
                self.errors.push(TypeError::said(
                    at,
                    field,
                    format!("'{}' இருமுறை கொடுக்கப்பட்டது", field),
                    format!("the field '{}' is given twice", field),
                ));
                continue;
            }
            match info.field(field) {
                Some(declared) => {
                    if let Some(declared) = &declared.declared {
                        let found = self.infer(value);
                        if !found.satisfies(declared) {
                            self.errors.push(TypeError::mismatch(
                                at,
                                format!("{}.{}", shape, field),
                                declared.clone(),
                                found.name(),
                            ));
                        }
                    }
                }
                None => self.no_such_field(shape, &info, field, at),
            }
        }

        match base {
            Some(base) => {
                let found = self.infer(base);
                let own = DeclaredType::Shape(shape.to_string());
                if !found.satisfies(&own) {
                    self.errors.push(TypeError::mismatch(
                        at,
                        format!("..{}", shape),
                        own,
                        found.name(),
                    ));
                }
            }
            None => {
                let missing: Vec<&str> = info
                    .fields
                    .iter()
                    .map(|field| field.name.as_str())
                    .filter(|name| !written.contains(name))
                    .collect();
                if !missing.is_empty() {
                    let list = missing.join(", ");
                    self.errors.push(TypeError::said(
                        at,
                        shape,
                        format!("'{}' க்கு {} புலங்களும் தேவை", shape, list),
                        format!(
                            "a {} needs every one of its fields; missing: {}",
                            shape, list
                        ),
                    ));
                }
            }
        }
    }

    fn check_assign(
        &mut self,
        name: &str,
        value: &Expr,
        declared: Option<DeclaredType>,
        at: Position,
        immutable: bool,
    ) {
        self.check_calls(value);

        // A declaration on this statement wins; failing that, one the program
        // made earlier still applies.
        let expected = declared
            .clone()
            .or_else(|| self.declared.get(name).cloned());

        let found = self.infer(value);
        if let Some(expected) = expected
            && !found.satisfies(&expected)
        {
            self.errors
                .push(TypeError::mismatch(at, name, expected, found.name()));
        }

        // Recorded even when it was wrong, so the rest of the file is checked
        // against what the author said rather than against the mistake.
        if let Some(declared) = declared {
            self.declared.insert(name.to_string(), declared);
        } else if immutable && let Inferred::Shape(shape) = found {
            // `நிலை க = கடன்{…};` says what க is as plainly as `கடன் க` would,
            // and since நிலை can never be assigned again it stays true — so
            // `க.அசால்` is caught with no declaration written.
            self.declared
                .insert(name.to_string(), DeclaredType::Shape(shape));
        }
    }

    fn infer(&self, expr: &Expr) -> Inferred {
        match expr {
            Expr::Number(_) => Inferred::Number,
            Expr::String(_) => Inferred::Text,
            Expr::Boolean(_) => Inferred::Boolean,
            Expr::ArrayLiteral(_) => Inferred::Array,
            Expr::RecordLiteral(_) => Inferred::Record,
            Expr::ShapeLiteral { shape, .. } => Inferred::Shape(shape.clone()),

            // Arithmetic yields a number whatever went in, because the VM
            // converts its operands.
            Expr::BinaryOp { .. } => Inferred::Number,
            Expr::Comparison { .. } | Expr::Logical { .. } | Expr::Not(_) => Inferred::Boolean,
            Expr::Concat { .. } => Inferred::Text,

            // இன்மை is the absent value and stands anywhere, so it makes no
            // claim rather than being its own type.
            Expr::Null => Inferred::Unknown,

            Expr::Variable(name) => Inferred::from_declared(self.declared.get(name)),

            // A call is whatever the function promised to return. Without a
            // declaration it is still unconstrained, which is most calls — and
            // a shape's name called is a result, which is not the shape.
            Expr::Call { name, .. } => Inferred::from_declared(
                self.signatures
                    .get(name)
                    .and_then(|signature| signature.returns.as_ref()),
            ),

            // A field of a known shape is what the shape declared it as.
            Expr::Field { base, name, .. } => match self.infer(base) {
                Inferred::Shape(shape) => Inferred::from_declared(
                    self.shapes
                        .get(&shape)
                        .and_then(|info| info.field(name))
                        .and_then(|field| field.declared.as_ref()),
                ),
                _ => Inferred::Unknown,
            },

            // A method returns what it declared.
            Expr::MethodCall { receiver, name, .. } => {
                let shape = match self.names_shape(receiver) {
                    Some(shape) => Some(shape),
                    None => match self.infer(receiver) {
                        Inferred::Shape(shape) => Some(shape),
                        _ => None,
                    },
                };
                Inferred::from_declared(
                    shape
                        .and_then(|shape| self.shapes.get(&shape))
                        .and_then(|info| info.methods.get(name))
                        .and_then(|(signature, _)| signature.returns.as_ref()),
                )
            }

            Expr::Lambda { .. } => Inferred::Function,
            // What a function value returns is not known until it is called.
            Expr::CallValue { .. } => Inferred::Unknown,

            // Indexing a collection says nothing about what is inside it.
            Expr::Index { .. } | Expr::Try(_) => Inferred::Unknown,
        }
    }
}
