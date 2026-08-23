use crate::lexer::{Spanned, Token};
use rust_decimal::Decimal;
use std::iter::Peekable;
use std::slice::Iter;

/// A parse error, carrying the position of the token that caused it.
///
/// The parser used to `panic!` with `Expected Semicolon` and no location,
/// which was the biggest usability gap in the language for anyone learning
/// it. Messages are bilingual, as the lexer's already were.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub line: usize,
    pub column: usize,
    /// What the parser was looking for.
    pub expected: String,
    /// The source text actually found, empty at the end of input.
    pub found: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.found.is_empty() {
            write!(
                f,
                "வரி {}, நெடுவரிசை {}: {} எதிர்பார்க்கப்பட்டது, உள்ளீடு முடிந்தது  \
                 (line {}, column {}: expected {}, but the input ended)",
                self.line, self.column, self.expected, self.line, self.column, self.expected
            )
        } else {
            write!(
                f,
                "வரி {}, நெடுவரிசை {}: {} எதிர்பார்க்கப்பட்டது, '{}' கிடைத்தது  \
                 (line {}, column {}: expected {}, found '{}')",
                self.line,
                self.column,
                self.expected,
                self.found,
                self.line,
                self.column,
                self.expected,
                self.found
            )
        }
    }
}

// --- Abstract Syntax Tree (AST) Nodes ---

/// Where something was written, carried on the AST nodes that can be rejected
/// after parsing, so a type error points at a place rather than only naming a
/// variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

/// A type written in the source: `எண் வருவாய் = 100000;`
///
/// Type keywords used to be parsed and thrown away, so `சொல் x = 5;` was
/// accepted. Keeping the declaration is what lets the checker hold an
/// assignment to it.
///
/// There is one numeric type. `எண்` and `பின்னம்` both mean Number, because
/// every value in the language is already a fixed-point decimal — a separate
/// integer type would be a second decision, not a consequence of this one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclaredType {
    Number,
    Text,
    Boolean,
    Array,
    Record,
    Date,
    /// No constraint: used where a type keyword exists that the checker has
    /// nothing to say about yet.
    Any,
}

/// One parameter of a `செயல்`, with the type it was declared as if it was.
///
/// `at` is where the name was written, so a call that passes the wrong thing
/// can be pointed at the parameter it disagrees with.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub declared: Option<DeclaredType>,
    pub at: Position,
}

impl DeclaredType {
    /// The keyword an author would have written, for error messages.
    pub fn name(&self) -> &'static str {
        match self {
            DeclaredType::Number => "எண் (eN, a number)",
            DeclaredType::Text => "சொல் (col, a string)",
            DeclaredType::Boolean => "ஈர்ம (Irma, a boolean)",
            DeclaredType::Array => "அணி (aNi, an array)",
            DeclaredType::Record => "பொருள் (poruL, a record)",
            DeclaredType::Date => "தேதி (qEqi, a date)",
            DeclaredType::Any => "any type",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(Decimal),
    String(String),
    Boolean(bool),
    Null,
    Variable(String),
    BinaryOp {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Comparison {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
    Concat {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    // maRRum / allaqu — both operands are evaluated (no short-circuiting)
    Logical {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    // illY
    Not(Box<Expr>),
    // name(arg, ...)
    Call {
        name: String,
        args: Vec<Expr>,
    },
    // அணி — a column: [a, b, c]
    ArrayLiteral(Vec<Expr>),
    // பொருள் — a row: {peyar: "ravi", vayaqu: 20}
    RecordLiteral(Vec<(String, Expr)>),
    // a[i]
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },
    // r.peyar
    Field {
        base: Box<Expr>,
        name: String,
    },
    // expr? — unwrap a சரி, or return the தவறு to the caller
    Try(Box<Expr>),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Stmt {
    Assign {
        name: String,
        value: Expr,
        /// The type the author wrote, if any: `எண் வருவாய் = 100000;`
        declared: Option<DeclaredType>,
        /// Where the name was written, for the checker to point at.
        at: Position,
    },
    // ceyal name(params) returns { body }
    FunctionDef {
        name: String,
        params: Vec<Param>,
        /// The type the function promises to return, if it said.
        returns: Option<DeclaredType>,
        body: Vec<Stmt>,
        /// Where the function's name was written, so a திரும்பு that breaks
        /// the declared return type has somewhere to point.
        at: Position,
    },
    // qirumpu value;
    Return(Option<Expr>),
    // a[i] = value;  — the base must be a plain variable for now
    SetIndex {
        name: String,
        index: Expr,
        value: Expr,
    },
    // r.peyar = value;
    SetField {
        name: String,
        field: String,
        value: Expr,
    },
    // A bare expression evaluated for its effect, e.g. a call statement.
    Expression(Expr),
    // iRakku "path.qmz"; — resolved before compilation, see module.rs
    Import(String),
    Print(Expr),
    Input(Expr),
    // (cond) eZil { then } iZREl { else }
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    // (cond) cuRRu { body }
    Loop {
        condition: Expr,
        body: Vec<Stmt>,
    },
    // ovvoru item il collection { body }
    ForEach {
        var: String,
        collection: Expr,
        body: Vec<Stmt>,
    },
    // File I/O Operations
    FileOpen {
        filename: Expr,
        mode: String, // "read", "write", "append"
    },
    FileClose {
        filename: Expr,
    },
    FileWrite {
        filename: Expr,
        data: Expr,
    },
    FileRead {
        filename: Expr,
        variable: String,
    },
    ReadCSV {
        filename: Expr,
        variable: String,
    },
    WriteCSV {
        filename: Expr,
        data: Expr,
    },
    // Database Operations
    DBConnect {
        db_type: String,
        connection_string: Expr,
        /// The name this connection is known by. Defaults to the driver name,
        /// which is how a program that names nothing keeps working.
        handle: Option<String>,
    },
    DBDisconnect {
        db_type: String,
    },
    // தளம்_வினா "sql", [params], result;
    DBQuery {
        query: Expr,
        params: Expr,
        result_var: String,
        /// Which connection to ask. `None` means the only open one.
        handle: Option<String>,
    },
    // தளம்_செய் "sql", [params];
    DBExecute {
        command: Expr,
        params: Expr,
        /// Which connection to run this on. `None` means the only open one.
        handle: Option<String>,
    },
    DBInsert {
        table: String,
        data: Expr,
    },
    DBUpdate {
        table: String,
        data: Expr,
        condition: Option<Expr>,
    },
    DBDelete {
        table: String,
        condition: Expr,
    },
    CreateTable {
        table: String,
        schema: Expr,
    },
    Select {
        columns: Vec<String>,
        from_table: String,
        where_clause: Option<Expr>,
    },
    // REST API Operations
    DefineRoute {
        method: String,      // GET, POST, PUT, DELETE, etc.
        path: Expr,          // "/api/users"
        handler: Vec<Stmt>,  // Handler code block
    },
    StartServer {
        host: Expr,
        port: Expr,
    },
    // இடைவெளி 60 { … } — run the block every 60 seconds. Lifted out of the
    // program at startup like வழி, and needs a server for the same reason.
    Schedule {
        seconds: Expr,
        body: Vec<Stmt>,
    },
    StopServer,
    // பதில் 200, உடல்;  or with headers as an ordinary record:
    // பதில் 200, உடல், {"Content-Type": "text/html"};
    SendResponse {
        status_code: Expr,
        body: Expr,
        headers: Option<Expr>,
    },
    SendJSON {
        data: Expr,
        status_code: Option<Expr>,
    },
    GetRequestBody {
        variable: String,
    },
    GetRequestParam {
        param_name: Expr,
        variable: String,
    },
    GetHeader {
        header_name: Expr,
        variable: String,
    },
    SetHeader {
        header_name: Expr,
        value: Expr,
    },
}

// --- Parser Implementation ---

pub struct Parser<'a> {
    tokens: Peekable<Iter<'a, Spanned>>,
    /// Where the last consumed token was, so an unexpected end of input can
    /// still be reported somewhere the author recognizes.
    last: (usize, usize),
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Iter<'a, Spanned>) -> Self {
        Parser {
            tokens: tokens.peekable(),
            last: (1, 1),
        }
    }

    /// Entry point: parse the whole token stream into statements.
    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while self.tokens.peek().is_some() {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    // --- Position and error helpers ---------------------------------------

    /// Look at the next token without consuming it.
    ///
    /// `Iter::peek` hands back a `&&Spanned`; copying the inner reference out
    /// frees the result from the borrow on `self`, so an error can be built
    /// from it in the same expression.
    fn peek_spanned(&mut self) -> Option<&'a Spanned> {
        self.tokens.peek().copied()
    }

    fn peek_token(&mut self) -> Option<&'a Token> {
        self.peek_spanned().map(|spanned| &spanned.token)
    }

    fn advance(&mut self) -> Option<&'a Spanned> {
        let spanned = self.tokens.next();
        if let Some(spanned) = spanned {
            self.last = (spanned.line, spanned.column);
        }
        spanned
    }

    /// Consume the next token, or report what was wanted instead.
    fn take(&mut self, expected: &str) -> Result<&'a Spanned, ParseError> {
        match self.advance() {
            Some(spanned) => Ok(spanned),
            None => Err(self.at_end(expected)),
        }
    }

    fn mismatch(&self, spanned: &Spanned, expected: &str) -> ParseError {
        ParseError {
            line: spanned.line,
            column: spanned.column,
            expected: expected.to_string(),
            found: spanned.text.clone(),
        }
    }

    fn at_end(&self, expected: &str) -> ParseError {
        ParseError {
            line: self.last.0,
            column: self.last.1,
            expected: expected.to_string(),
            found: String::new(),
        }
    }

    /// A readable name for a token the parser was expecting. Punctuation reads
    /// the same in either language; a keyword shows both spellings.
    fn describe(token: &Token) -> String {
        match token {
            Token::Semicolon => "';'",
            Token::Comma => "','",
            Token::Colon => "':'",
            Token::Assign => "'='",
            Token::LParen => "'('",
            Token::RParen => "')'",
            Token::LBrace => "'{'",
            Token::RBrace => "'}'",
            Token::LBracket => "'['",
            Token::RBracket => "']'",
            Token::In => "இல் (il)",
            Token::From => "இதனில் (iqaZil)",
            Token::Where => "விதி (viqi)",
            Token::If => "எனில் (eZil)",
            Token::Loop => "சுற்று (cuRRu)",
            Token::Else => "இன்றேல் (iZREl)",
            other => return format!("{:?}", other),
        }
        .to_string()
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        match self.peek_spanned() {
            Some(spanned) if spanned.token == expected => {
                self.advance();
                Ok(())
            }
            Some(spanned) => Err(self.mismatch(spanned, &Self::describe(&expected))),
            None => Err(self.at_end(&Self::describe(&expected))),
        }
    }

    fn matches(&mut self, expected: Token) -> bool {
        match self.peek_spanned() {
            Some(spanned) if spanned.token == expected => {
                self.advance();
                true
            }
            _ => false,
        }
    }

    // --- Names -------------------------------------------------------------

    /// A name as the author wrote it.
    ///
    /// This is what keeping the source text was for. The spelling used to be
    /// discarded and only the token kept, so `வங்கி = 5` created a variable
    /// called `Bank`: a Tamil author's chosen name was silently anglicised,
    /// printing such a record emitted English field names into Tamil output,
    /// and looking a field up by string needed the token name rather than the
    /// written one.
    ///
    /// The documented consequence: `{வரி: 1}` and `{vari: 1}` are now
    /// *different* fields. A field name is data, not a language construct.
    fn name_of(&self, spanned: &Spanned) -> String {
        match &spanned.token {
            // A quoted key carries its parsed contents, not its quotes.
            Token::String(text) => text.clone(),
            _ => spanned.text.clone(),
        }
    }

    /// Consume a token that must be usable as a name.
    fn take_name(&mut self, expected: &str) -> Result<String, ParseError> {
        let spanned = self.take(expected)?;
        let usable = matches!(spanned.token, Token::String(_))
            || (Self::is_identifier_like(&spanned.token) && !Self::is_type_token(&spanned.token));

        if !usable {
            return Err(self.mismatch(spanned, expected));
        }
        Ok(self.name_of(spanned))
    }

    /// The canonical English name of a keyword.
    ///
    /// Kept for the two places where a keyword names something the host must
    /// recognize rather than something the author invented: a database type,
    /// which `db::open` matches on, and an HTTP method, which the router
    /// matches on. Those are not the author's names, so they do not follow the
    /// author's spelling.
    /// A trailing `, name` naming which connection a statement means.
    ///
    /// Absent for every program written before handles existed, which is why
    /// it is optional rather than a new required slot.
    fn optional_handle(&mut self, expected: &str) -> Result<Option<String>, ParseError> {
        if !self.matches(Token::Comma) {
            return Ok(None);
        }
        let spanned = self.take(expected)?;
        Ok(Some(Self::token_name(&spanned.token)))
    }

    fn token_name(token: &Token) -> String {
        match token {
            Token::Identifier(name) => name.clone(),
            Token::SQL => "SQL".to_string(),
            Token::NoSQL => "NoSQL".to_string(),
            Token::SQLite => "SQLite".to_string(),
            Token::MySQL => "MySQL".to_string(),
            Token::PostgreSQL => "PostgreSQL".to_string(),
            Token::MongoDB => "MongoDB".to_string(),
            Token::Redis => "Redis".to_string(),
            Token::JSONdb => "JSONdb".to_string(),
            Token::HttpGet => "GET".to_string(),
            Token::HttpPost => "POST".to_string(),
            Token::HttpPut => "PUT".to_string(),
            Token::HttpDelete => "DELETE".to_string(),
            Token::HttpPatch => "PATCH".to_string(),
            Token::HttpOptions => "OPTIONS".to_string(),
            Token::HttpHead => "HEAD".to_string(),
            other => format!("{:?}", other),
        }
    }

    // --- Statements --------------------------------------------------------

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        let first = self.take("a statement")?;

        // An optional type declaration: eN, piZZam, col and the rest. The
        // declared type is kept so the checker can hold assignments to it.
        let (declared, current) = if Self::is_type_token(&first.token) {
            let name = self.take("a name after the type")?;
            (Some(Self::type_of(&first.token)), name)
        } else {
            (None, first)
        };

        if Self::is_identifier_like(&current.token) && !Self::is_type_token(&current.token) {
            let name = self.name_of(current);
            let at = Position {
                line: current.line,
                column: current.column,
            };

            // A call used as a statement, e.g. `paqivu_ceyal(x);`
            if self.peek_token() == Some(&Token::LParen) {
                let call = self.finish_name_or_call(name)?;
                self.expect(Token::Semicolon)?;
                return Ok(Stmt::Expression(call));
            }

            // a[i] = value;
            if self.matches(Token::LBracket) {
                let index = self.parse_expression()?;
                self.expect(Token::RBracket)?;
                self.expect(Token::Assign)?;
                let value = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                return Ok(Stmt::SetIndex { name, index, value });
            }

            // r.field = value;
            if self.matches(Token::Dot) {
                let field = self.take_name("a field name")?;
                self.expect(Token::Assign)?;
                let value = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                return Ok(Stmt::SetField { name, field, value });
            }

            // A declaration with no initializer.
            if self.matches(Token::Semicolon) {
                return Ok(Stmt::Assign {
                    name,
                    value: Expr::Number(Decimal::ZERO),
                    declared,
                    at,
                });
            }

            self.expect(Token::Assign)?;
            let value = self.parse_expression()?;
            self.expect(Token::Semicolon)?;
            return Ok(Stmt::Assign { name, value, declared, at });
        }

        match &current.token {
            Token::Function => {
                let at = match self.peek_spanned() {
                    Some(spanned) => Position {
                        line: spanned.line,
                        column: spanned.column,
                    },
                    None => Position {
                        line: self.last.0,
                        column: self.last.1,
                    },
                };
                let name = self.take_name("a function name")?;
                self.expect(Token::LParen)?;
                let mut params = Vec::new();
                if !self.matches(Token::RParen) {
                    loop {
                        params.push(self.parse_param()?);
                        if !self.matches(Token::Comma) {
                            break;
                        }
                    }
                    self.expect(Token::RParen)?;
                }

                // An optional return type, between the parameter list and the
                // body. Nothing else can appear there, so it needs no marker.
                let returns = match self.peek_token() {
                    Some(token) if Self::is_type_token(token) => {
                        let token = self.take("a return type")?;
                        Some(Self::type_of(&token.token))
                    }
                    _ => None,
                };

                self.expect(Token::LBrace)?;
                let body = self.parse_block()?;
                Ok(Stmt::FunctionDef {
                    name,
                    params,
                    returns,
                    body,
                    at,
                })
            }
            Token::Import => {
                let path = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                match path {
                    Expr::String(path) => Ok(Stmt::Import(path)),
                    _ => Err(self.at_end("a quoted file path after இறக்கு (iRakku)")),
                }
            }
            Token::ForEach => {
                let var = self.take_name("a loop variable")?;
                self.expect(Token::In)?;
                let collection = self.parse_expression()?;
                self.expect(Token::LBrace)?;
                let body = self.parse_block()?;
                Ok(Stmt::ForEach { var, collection, body })
            }
            Token::Return => {
                if self.matches(Token::Semicolon) {
                    Ok(Stmt::Return(None))
                } else {
                    let value = self.parse_expression()?;
                    self.expect(Token::Semicolon)?;
                    Ok(Stmt::Return(Some(value)))
                }
            }
            Token::Print => {
                let value = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Print(value))
            }
            Token::Input => {
                let value = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Input(value))
            }
            Token::FileOpen => {
                let filename = self.parse_expression()?;
                let mode = if self.matches(Token::Comma) {
                    let mode = self.parse_expression()?;
                    Self::expr_to_string(mode).to_lowercase()
                } else {
                    "read".to_string()
                };
                self.expect(Token::Semicolon)?;
                Ok(Stmt::FileOpen { filename, mode })
            }
            Token::FileClose => {
                let filename = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::FileClose { filename })
            }
            Token::FileWrite => {
                let filename = self.parse_expression()?;
                self.expect(Token::Comma)?;
                let data = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::FileWrite { filename, data })
            }
            Token::FileRead => {
                let filename = self.parse_expression()?;
                self.expect(Token::Comma)?;
                let variable = self.take_name("a variable to read into")?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::FileRead { filename, variable })
            }
            Token::ReadCSV => {
                let filename = self.parse_expression()?;
                self.expect(Token::Comma)?;
                let variable = self.take_name("a variable to read into")?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::ReadCSV { filename, variable })
            }
            Token::WriteCSV => {
                let filename = self.parse_expression()?;
                self.expect(Token::Comma)?;
                let data = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::WriteCSV { filename, data })
            }
            Token::DBConnect => {
                // The database type is the host's name for a backend, not a
                // name the author invented, so it keeps its canonical spelling.
                let db_type = Self::token_name(&self.take("a database type")?.token);
                self.expect(Token::Comma)?;
                let connection_string = self.parse_expression()?;
                // An optional name for this connection, so a second one can be
                // opened and told apart. Trailing, because the grammar is
                // fixed-arity and a trailing name therefore cannot be mistaken
                // for anything else.
                let handle = self.optional_handle("a name for this connection")?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::DBConnect {
                    db_type,
                    connection_string,
                    handle,
                })
            }
            Token::DBDisconnect => {
                let db_type = Self::token_name(&self.take("a database type")?.token);
                self.expect(Token::Semicolon)?;
                Ok(Stmt::DBDisconnect { db_type })
            }
            Token::DBQuery => {
                let query = self.parse_expression()?;
                self.expect(Token::Comma)?;
                let params = self.parse_expression()?;
                self.expect(Token::Comma)?;
                let result_var = self.take_name("a variable to hold the rows")?;
                let handle = self.optional_handle("the connection to query")?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::DBQuery {
                    query,
                    params,
                    result_var,
                    handle,
                })
            }
            Token::DBExecute => {
                let command = self.parse_expression()?;
                self.expect(Token::Comma)?;
                let params = self.parse_expression()?;
                let handle = self.optional_handle("the connection to run this on")?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::DBExecute {
                    command,
                    params,
                    handle,
                })
            }
            Token::DBInsert => {
                let table = self.take_name("a table name")?;
                self.expect(Token::Comma)?;
                let data = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::DBInsert { table, data })
            }
            Token::DBUpdate => {
                let table = self.take_name("a table name")?;
                self.expect(Token::Comma)?;
                let data = self.parse_expression()?;
                let condition = if self.matches(Token::Comma) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.expect(Token::Semicolon)?;
                Ok(Stmt::DBUpdate { table, data, condition })
            }
            Token::DBDelete => {
                let table = self.take_name("a table name")?;
                self.expect(Token::Comma)?;
                let condition = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::DBDelete { table, condition })
            }
            Token::CreateTable => {
                let table = self.take_name("a table name")?;
                self.expect(Token::Comma)?;
                let schema = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::CreateTable { table, schema })
            }
            Token::Select => {
                let mut columns = Vec::new();
                loop {
                    columns.push(self.take_name("a column name")?);
                    if !self.matches(Token::Comma) {
                        break;
                    }
                }
                self.expect(Token::From)?;
                let from_table = self.take_name("a table name")?;
                let where_clause = if self.matches(Token::Where) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Select { columns, from_table, where_clause })
            }
            Token::Route => {
                // The HTTP method is matched by the router, so like a database
                // type it keeps its canonical spelling rather than the
                // author's.
                let method = Self::token_name(&self.take("an HTTP method")?.token);
                self.expect(Token::Comma)?;
                let path = self.parse_expression()?;
                self.expect(Token::LBrace)?;
                let handler = self.parse_block()?;
                Ok(Stmt::DefineRoute { method, path, handler })
            }
            Token::Every => {
                let seconds = self.parse_expression()?;
                self.expect(Token::LBrace)?;
                let body = self.parse_block()?;
                Ok(Stmt::Schedule { seconds, body })
            }
            Token::StartServer => {
                let host = self.parse_expression()?;
                self.expect(Token::Comma)?;
                let port = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::StartServer { host, port })
            }
            Token::StopServer => {
                self.expect(Token::Semicolon)?;
                Ok(Stmt::StopServer)
            }
            Token::Response => {
                let status_code = self.parse_expression()?;
                self.expect(Token::Comma)?;
                let body = self.parse_expression()?;
                let headers = if self.matches(Token::Comma) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.expect(Token::Semicolon)?;
                Ok(Stmt::SendResponse { status_code, body, headers })
            }
            Token::JSONBody => {
                let data = self.parse_expression()?;
                let status_code = if self.matches(Token::Comma) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.expect(Token::Semicolon)?;
                Ok(Stmt::SendJSON { data, status_code })
            }
            Token::LParen => {
                let condition = self.parse_expression()?;
                self.expect(Token::RParen)?;

                let keyword = self.take("எனில் (eZil) or சுற்று (cuRRu) after a condition")?;
                match keyword.token {
                    Token::If => self.parse_if_remainder(condition),
                    Token::Loop => self.parse_loop_remainder(condition),
                    _ => Err(self.mismatch(
                        keyword,
                        "எனில் (eZil) or சுற்று (cuRRu) after a condition",
                    )),
                }
            }
            _ => Err(self.mismatch(current, "a statement")),
        }
    }

    /// Statements up to a closing brace, which is consumed.
    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut body = Vec::new();
        loop {
            if self.matches(Token::RBrace) {
                return Ok(body);
            }
            if self.tokens.peek().is_none() {
                return Err(self.at_end("'}'"));
            }
            body.push(self.parse_statement()?);
        }
    }

    fn parse_if_remainder(&mut self, condition: Expr) -> Result<Stmt, ParseError> {
        self.expect(Token::LBrace)?;
        let then_branch = self.parse_block()?;

        let else_branch = if self.matches(Token::Else) {
            self.expect(Token::LBrace)?;
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Stmt::If { condition, then_branch, else_branch })
    }

    fn parse_loop_remainder(&mut self, condition: Expr) -> Result<Stmt, ParseError> {
        self.expect(Token::LBrace)?;
        let body = self.parse_block()?;
        Ok(Stmt::Loop { condition, body })
    }

    // --- Expressions -------------------------------------------------------
    //
    // Precedence, loosest first:
    //   or < and < not < comparison < additive < term < factor

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;
        while self.peek_token() == Some(&Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::Logical {
                op: "||".to_string(),
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_not()?;
        while self.peek_token() == Some(&Token::And) {
            self.advance();
            let right = self.parse_not()?;
            left = Expr::Logical {
                op: "&&".to_string(),
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> Result<Expr, ParseError> {
        if self.peek_token() == Some(&Token::Not) {
            self.advance();
            return Ok(Expr::Not(Box::new(self.parse_not()?)));
        }
        self.parse_comparison()
    }

    /// One comparison, not a chain.
    ///
    /// This used to loop, which made comparison left-associative: `அ > ஆ > இ`
    /// parsed as `(அ > ஆ) > இ`, a Boolean compared against a number. `3 > 2 > 1`
    /// was **false**, and nothing said so.
    ///
    /// A second comparison operator is now an error rather than that. Chaining
    /// it the way Python does — `அ > ஆ மற்றும் ஆ > இ` — is what a language full
    /// of tax slabs actually wants, and it is the obvious next step. It needs
    /// the middle operand evaluated exactly once, and there is no way to say
    /// "once" in this AST, so `f() > g() > h()` would call `g` twice. Trading a
    /// wrong answer for a subtler wrong answer is not a fix.
    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_additive()?;

        let op = match self.peek_token() {
            Some(Token::GreaterThan) => ">",
            Some(Token::LessThan) => "<",
            Some(Token::Equals) => "==",
            Some(Token::NotEquals) => "!=",
            Some(Token::GreaterThanOrEqual) => ">=",
            Some(Token::LessThanOrEqual) => "<=",
            _ => return Ok(left),
        };
        self.advance();
        let right = self.parse_additive()?;
        let comparison = Expr::Comparison {
            left: Box::new(left),
            op: op.to_string(),
            right: Box::new(right),
        };

        // A second one in a row is the mistake this arm exists to catch.
        if matches!(
            self.peek_token(),
            Some(Token::GreaterThan)
                | Some(Token::LessThan)
                | Some(Token::Equals)
                | Some(Token::NotEquals)
                | Some(Token::GreaterThanOrEqual)
                | Some(Token::LessThanOrEqual)
        ) {
            let spanned = self.peek_spanned().cloned();
            let expected = "ஒரே ஒப்பீடு  (one comparison at a time: write (அ > ஆ) மற்றும் (ஆ > இ))";
            return Err(match spanned {
                Some(spanned) => self.mismatch(&spanned, expected),
                None => self.at_end(expected),
            });
        }

        Ok(comparison)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_term()?;
        loop {
            let op = match self.peek_token() {
                Some(Token::Plus) => "+",
                Some(Token::Minus) => "-",
                Some(Token::Ampersand) => "&",
                _ => break,
            };
            self.advance();
            let right = self.parse_term()?;
            left = if op == "&" {
                Expr::Concat {
                    left: Box::new(left),
                    right: Box::new(right),
                }
            } else {
                Expr::BinaryOp {
                    op: op.to_string(),
                    left: Box::new(left),
                    right: Box::new(right),
                }
            };
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_factor()?;
        loop {
            let op = match self.peek_token() {
                Some(Token::Multiply) => "*",
                Some(Token::Divide) => "/",
                _ => break,
            };
            self.advance();
            let right = self.parse_factor()?;
            left = Expr::BinaryOp {
                op: op.to_string(),
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    /// A primary expression followed by any number of `[i]`, `.name` and `?`.
    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.matches(Token::LBracket) {
                let index = self.parse_expression()?;
                self.expect(Token::RBracket)?;
                expr = Expr::Index {
                    base: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.matches(Token::Dot) {
                let name = self.take_name("a field name")?;
                expr = Expr::Field {
                    base: Box::new(expr),
                    name,
                };
            } else if self.matches(Token::Question) {
                expr = Expr::Try(Box::new(expr));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        // `இல்லை` as an operand, so that `"விடை: " & இல்லை காலியா(அ)` parses.
        // It was reachable only from `parse_not`, which sits above comparison
        // and therefore above `&`, so this was a parse error rather than an
        // expression. Binding it tightly here changes no existing meaning:
        // `இல்லை அ > ஆ` still goes through `parse_not` first, and still means
        // `இல்லை (அ > ஆ)` rather than `(இல்லை அ) > ஆ`.
        if self.peek_token() == Some(&Token::Not) {
            self.advance();
            return Ok(Expr::Not(Box::new(self.parse_factor()?)));
        }

        let spanned = self.take("a value")?;

        match &spanned.token {
            // அணி — an array literal: [a, b, c]
            Token::LBracket => {
                let mut items = Vec::new();
                if !self.matches(Token::RBracket) {
                    loop {
                        items.push(self.parse_expression()?);
                        if !self.matches(Token::Comma) {
                            break;
                        }
                    }
                    self.expect(Token::RBracket)?;
                }
                Ok(Expr::ArrayLiteral(items))
            }
            // பொருள் — a record literal: {peyar: "ravi", vayaqu: 20}
            Token::LBrace => {
                let mut fields = Vec::new();
                if !self.matches(Token::RBrace) {
                    loop {
                        let key = self.take_name("a field name")?;
                        self.expect(Token::Colon)?;
                        let value = self.parse_expression()?;
                        fields.push((key, value));
                        if !self.matches(Token::Comma) {
                            break;
                        }
                    }
                    self.expect(Token::RBrace)?;
                }
                Ok(Expr::RecordLiteral(fields))
            }
            // Unary minus, compiled as 0 - x.
            Token::Minus => {
                let operand = self.parse_factor()?;
                Ok(Expr::BinaryOp {
                    op: "-".to_string(),
                    left: Box::new(Expr::Number(Decimal::ZERO)),
                    right: Box::new(operand),
                })
            }
            Token::Number(n) => Ok(Expr::Number(*n)),
            Token::Percentage(n) => Ok(Expr::Number(*n)),
            Token::String(s) => Ok(Expr::String(s.clone())),
            Token::True => Ok(Expr::Boolean(true)),
            Token::False => Ok(Expr::Boolean(false)),
            Token::Null => Ok(Expr::Null),
            Token::LParen => {
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            // An identifier, or a financial keyword used as a name.
            token if Self::is_identifier_like(token) && !Self::is_type_token(token) => {
                let name = self.name_of(spanned);
                self.finish_name_or_call(name)
            }
            _ => Err(self.mismatch(spanned, "a value")),
        }
    }

    /// A name already consumed: a call if `(` follows, otherwise a variable.
    fn finish_name_or_call(&mut self, name: String) -> Result<Expr, ParseError> {
        if !self.matches(Token::LParen) {
            return Ok(Expr::Variable(name));
        }

        let mut args = Vec::new();
        if !self.matches(Token::RParen) {
            loop {
                args.push(self.parse_expression()?);
                if !self.matches(Token::Comma) {
                    break;
                }
            }
            self.expect(Token::RParen)?;
        }
        Ok(Expr::Call { name, args })
    }

    // --- Token classification ---------------------------------------------

    fn is_identifier_like(token: &Token) -> bool {
        match token {
            Token::Number(_) | Token::Percentage(_) | Token::String(_) => false,
            Token::If | Token::Else | Token::Loop | Token::Print | Token::Input => false,
            Token::And | Token::Or | Token::Not => false,
            Token::True | Token::False | Token::Null => false,
            Token::Function | Token::Return => false,
            Token::ForEach | Token::In | Token::Import => false,
            Token::Assign | Token::Plus | Token::Minus | Token::Multiply | Token::Divide | Token::Ampersand => false,
            Token::Question | Token::Dot | Token::Colon => false,
            Token::LBracket | Token::RBracket => false,
            Token::LParen | Token::RParen | Token::LBrace | Token::RBrace | Token::Comma | Token::Semicolon => false,
            Token::GreaterThan | Token::LessThan | Token::Equals | Token::NotEquals | Token::GreaterThanOrEqual | Token::LessThanOrEqual => false,
            Token::File | Token::CSV | Token::Read | Token::Write | Token::Open | Token::Close => false,
            Token::FileOpen | Token::FileClose | Token::FileRead | Token::FileWrite | Token::ReadCSV | Token::WriteCSV => false,
            // Database operations
            Token::Database | Token::DBConnect | Token::DBDisconnect | Token::DBQuery | Token::DBExecute | Token::DBSearch | Token::DBInsert | Token::DBUpdate | Token::DBDelete => false,
            Token::Table | Token::Collection | Token::Row | Token::Column | Token::Key | Token::PrimaryKey | Token::ForeignKey | Token::Index => false,
            Token::CreateTable | Token::AlterTable | Token::DropTable => false,
            Token::Select | Token::From | Token::Where | Token::OrderBy | Token::GroupBy | Token::Join | Token::Left | Token::Right | Token::Inner | Token::Outer | Token::Distinct | Token::Limit | Token::Offset => false,
            // Database types
            Token::SQL | Token::NoSQL | Token::SQLite | Token::MySQL | Token::PostgreSQL | Token::MongoDB | Token::Redis | Token::JSONdb => false,
            // REST API operations
            Token::Route | Token::Every | Token::StartServer | Token::StopServer | Token::Response | Token::JSONBody => false,
            Token::Request | Token::Endpoint | Token::API | Token::Header | Token::Body | Token::Param | Token::QueryParam | Token::PathParam => false,
            Token::URL | Token::Host | Token::Port | Token::Method | Token::StatusCode | Token::StatusMessage | Token::Auth | Token::BearerToken | Token::ContentType | Token::Serve => false,
            Token::HttpGet | Token::HttpPost | Token::HttpPut | Token::HttpDelete | Token::HttpPatch | Token::HttpOptions | Token::HttpHead => false,
            // Security
            Token::Encrypt | Token::Decrypt | Token::Password | Token::EncryptionKey => false,
            // Financial and accounting keywords ARE usable as names: வருவாய்,
            // வரி and the rest are the domain nouns programs are written
            // about. They have no statement syntax of their own, and listing
            // them here made `எண் வருவாய்;` — the language's own headline
            // example — a parse error.
            _ => true,
        }
    }

    fn is_type_token(token: &Token) -> bool {
        matches!(
            token,
            Token::IntegerType
                | Token::FloatType
                | Token::StringType
                | Token::BoolType
                | Token::TextType
                | Token::ArrayType
                | Token::DataType
                | Token::ObjectType
                | Token::DateType
        )
    }

    /// One parameter: an optional type keyword, then the name. The same order
    /// a variable declaration uses, so `எண் தொகை` reads the same in both places.
    fn parse_param(&mut self) -> Result<Param, ParseError> {
        let declared = match self.peek_token() {
            Some(token) if Self::is_type_token(token) => {
                let token = self.take("a parameter type")?;
                Some(Self::type_of(&token.token))
            }
            _ => None,
        };

        let spanned = self.take("a parameter name")?;
        if !Self::is_identifier_like(&spanned.token) || Self::is_type_token(&spanned.token) {
            return Err(self.mismatch(spanned, "a parameter name"));
        }
        let at = Position {
            line: spanned.line,
            column: spanned.column,
        };
        Ok(Param {
            name: self.name_of(spanned),
            declared,
            at,
        })
    }

    /// The declared type a type keyword names.
    fn type_of(token: &Token) -> DeclaredType {
        match token {
            Token::IntegerType | Token::FloatType => DeclaredType::Number,
            Token::StringType | Token::TextType => DeclaredType::Text,
            Token::BoolType => DeclaredType::Boolean,
            Token::ArrayType => DeclaredType::Array,
            Token::ObjectType | Token::DataType => DeclaredType::Record,
            Token::DateType => DeclaredType::Date,
            // is_type_token gates every caller, so this is unreachable in
            // practice; treated as unconstrained rather than panicking.
            _ => DeclaredType::Any,
        }
    }

    fn expr_to_string(expr: Expr) -> String {
        match expr {
            Expr::String(s) => s,
            Expr::Variable(name) => name,
            Expr::Number(n) => n.to_string(),
            Expr::Boolean(b) => b.to_string(),
            Expr::Null => "nil".to_string(),
            Expr::BinaryOp { op, .. } => op,
            Expr::Comparison { op, .. } => op,
            Expr::Logical { op, .. } => op,
            Expr::Not(_) => "not".to_string(),
            Expr::Call { name, .. } => name,
            Expr::ArrayLiteral(_) => "array".to_string(),
            Expr::RecordLiteral(_) => "record".to_string(),
            Expr::Index { .. } => "index".to_string(),
            Expr::Field { name, .. } => name,
            Expr::Try(_) => "try".to_string(),
            Expr::Concat { .. } => "concat".to_string(),
        }
    }
}
