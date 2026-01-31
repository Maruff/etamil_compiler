use crate::lexer::Token;
use std::iter::Peekable;
use std::slice::Iter;

// --- Abstract Syntax Tree (AST) Nodes ---

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    String(String),
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
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Stmt {
    Assign {
        name: String,
        value: Expr,
    },
    Print(Expr),
    Input(Expr),
    // (cond) enil { then } inREl { else }
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
    },
    DBDisconnect {
        db_type: String,
    },
    DBQuery {
        query: Expr,
        result_var: Option<String>,
    },
    DBExecute {
        command: Expr,
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
    StopServer,
    SendResponse {
        status_code: Expr,
        body: Expr,
        headers: Option<Vec<(String, Expr)>>,
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
    tokens: Peekable<Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Iter<'a, Token>) -> Self {
        Parser {
            tokens: tokens.peekable(),
        }
    }

    /// Entry point: Parses the entire token stream into statements
    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while self.tokens.peek().is_some() {
            statements.push(self.parse_statement());
        }
        statements
    }

    fn parse_statement(&mut self) -> Stmt {
        let token = self.tokens.next().expect("Unexpected end of input");

        // Skip optional type declaration (eN, pinnam, col, etc.)
        let current_token = if self.is_type_token(&token) {
            self.tokens.next().expect("Expected identifier after type")
        } else {
            token
        };

        if self.is_identifier_like(&current_token) {
            let name = match &current_token {
                Token::Identifier(n) => n.clone(),
                _ => self.token_name(&current_token),
            };
            
            // Check if it's just a declaration (semicolon) or assignment
            if self.tokens.peek() == Some(&&Token::Semicolon) {
                self.tokens.next(); // consume semicolon
                // Variable declaration without initialization - assign 0
                return Stmt::Assign { name, value: Expr::Number(0.0) };
            }
            
            self.expect(Token::Assign);
            let value = self.parse_expression();
            self.expect(Token::Semicolon);
            return Stmt::Assign { name, value };
        }

        match current_token {
            Token::Print => {
                let val = self.parse_expression();
                self.expect(Token::Semicolon);
                Stmt::Print(val)
            }
            Token::Input => {
                let val = self.parse_expression();
                self.expect(Token::Semicolon);
                Stmt::Input(val)
            }
            Token::FileOpen => {
                // கோப்பு_திற "file.txt";  (optional: , "read")
                let filename = self.parse_expression();
                let mode = if self.matches(Token::Comma) {
                    let mode_expr = self.parse_expression();
                    self.expr_to_string(mode_expr).to_lowercase()
                } else {
                    "read".to_string()
                };
                self.expect(Token::Semicolon);
                Stmt::FileOpen { filename, mode }
            }
            Token::FileClose => {
                // கோப்பு_பின்வை "file.txt";
                let filename = self.parse_expression();
                self.expect(Token::Semicolon);
                Stmt::FileClose { filename }
            }
            Token::FileWrite => {
                // கோப்பு_எழுது "file.txt", "data to write";
                let filename = self.parse_expression();
                self.expect(Token::Comma);
                let data = self.parse_expression();
                self.expect(Token::Semicolon);
                Stmt::FileWrite { filename, data }
            }
            Token::FileRead => {
                // கோப்பு_படி "file.txt", varName;
                let filename = self.parse_expression();
                self.expect(Token::Comma);
                let var_token = self.tokens.next().expect("Expected variable name");
                let variable = match &var_token {
                    Token::Identifier(n) => n.clone(),
                    _ => self.token_name(&var_token),
                };
                self.expect(Token::Semicolon);
                Stmt::FileRead { filename, variable }
            }
            Token::ReadCSV => {
                // CSV_படி "data.csv", varName;
                let filename = self.parse_expression();
                self.expect(Token::Comma);
                let var_token = self.tokens.next().expect("Expected variable name");
                let variable = match &var_token {
                    Token::Identifier(n) => n.clone(),
                    _ => self.token_name(&var_token),
                };
                self.expect(Token::Semicolon);
                Stmt::ReadCSV { filename, variable }
            }
            Token::WriteCSV => {
                // CSV_எழுது "data.csv", data;
                let filename = self.parse_expression();
                self.expect(Token::Comma);
                let data = self.parse_expression();
                self.expect(Token::Semicolon);
                Stmt::WriteCSV { filename, data }
            }
            Token::DBConnect => {
                // தரவுசேமி_இணை SQL "connection_string";
                let db_type_token = self.tokens.next().expect("Expected database type");
                let db_type = self.token_name(&db_type_token);
                self.expect(Token::Comma);
                let connection_string = self.parse_expression();
                self.expect(Token::Semicolon);
                Stmt::DBConnect { db_type, connection_string }
            }
            Token::DBDisconnect => {
                // தரவுசேமி_பிரிந்து SQL;
                let db_type_token = self.tokens.next().expect("Expected database type");
                let db_type = self.token_name(&db_type_token);
                self.expect(Token::Semicolon);
                Stmt::DBDisconnect { db_type }
            }
            Token::DBQuery => {
                // தளம்_வினா "SELECT * FROM table", result_var;
                let query = self.parse_expression();
                let result_var = if self.matches(Token::Comma) {
                    let var_token = self.tokens.next().expect("Expected variable name");
                    Some(match &var_token {
                        Token::Identifier(n) => n.clone(),
                        _ => self.token_name(&var_token),
                    })
                } else {
                    None
                };
                self.expect(Token::Semicolon);
                Stmt::DBQuery { query, result_var }
            }
            Token::DBExecute => {
                // தரவுசேமி_செய் "SQL command";
                let command = self.parse_expression();
                self.expect(Token::Semicolon);
                Stmt::DBExecute { command }
            }
            Token::DBInsert => {
                // தரவுசேமி_செருக table_name, data;
                let table_token = self.tokens.next().expect("Expected table name");
                let table = self.token_name(&table_token);
                self.expect(Token::Comma);
                let data = self.parse_expression();
                self.expect(Token::Semicolon);
                Stmt::DBInsert { table, data }
            }
            Token::DBUpdate => {
                // தரவுசேமி_புதுப்பி table_name, data, condition;
                let table_token = self.tokens.next().expect("Expected table name");
                let table = self.token_name(&table_token);
                self.expect(Token::Comma);
                let data = self.parse_expression();
                let condition = if self.matches(Token::Comma) {
                    Some(self.parse_expression())
                } else {
                    None
                };
                self.expect(Token::Semicolon);
                Stmt::DBUpdate { table, data, condition }
            }
            Token::DBDelete => {
                // தரவுசேமி_நீக்கு table_name, condition;
                let table_token = self.tokens.next().expect("Expected table name");
                let table = self.token_name(&table_token);
                self.expect(Token::Comma);
                let condition = self.parse_expression();
                self.expect(Token::Semicolon);
                Stmt::DBDelete { table, condition }
            }
            Token::CreateTable => {
                // அட்டை_ஆக்கு table_name, "schema definition";
                let table_token = self.tokens.next().expect("Expected table name");
                let table = self.token_name(&table_token);
                self.expect(Token::Comma);
                let schema = self.parse_expression();
                self.expect(Token::Semicolon);
                Stmt::CreateTable { table, schema }
            }
            Token::Select => {
                // தேர்வெடு column1, column2 இதனில் table_name விதி condition;
                let mut columns = Vec::new();
                loop {
                    let col_token = self.tokens.next().expect("Expected column name");
                    columns.push(self.token_name(&col_token));
                    if !self.matches(Token::Comma) {
                        break;
                    }
                }
                self.expect(Token::From);
                let table_token = self.tokens.next().expect("Expected table name");
                let from_table = self.token_name(&table_token);
                let where_clause = if self.matches(Token::Where) {
                    Some(self.parse_expression())
                } else {
                    None
                };
                self.expect(Token::Semicolon);
                Stmt::Select { columns, from_table, where_clause }
            }
            Token::Route => {
                // வழி GET "/api/users" { handler code };
                let method_token = self.tokens.next().expect("Expected HTTP method");
                let method = self.token_name(&method_token);
                self.expect(Token::Comma);
                let path = self.parse_expression();
                self.expect(Token::LBrace);
                let mut handler = Vec::new();
                while !self.matches(Token::RBrace) {
                    handler.push(self.parse_statement());
                }
                Stmt::DefineRoute { method, path, handler }
            }
            Token::StartServer => {
                // வழங்கி_தொடங்கு "localhost", 8080;
                let host = self.parse_expression();
                self.expect(Token::Comma);
                let port = self.parse_expression();
                self.expect(Token::Semicolon);
                Stmt::StartServer { host, port }
            }
            Token::StopServer => {
                // வழங்கி_நிறுத்து;
                self.expect(Token::Semicolon);
                Stmt::StopServer
            }
            Token::Response => {
                // பதில் 200, "OK", headers;
                let status_code = self.parse_expression();
                self.expect(Token::Comma);
                let body = self.parse_expression();
                let headers = if self.matches(Token::Comma) {
                    // Parse headers (simplified)
                    Some(Vec::new())
                } else {
                    None
                };
                self.expect(Token::Semicolon);
                Stmt::SendResponse { status_code, body, headers }
            }
            Token::JSONBody => {
                // ஜேசான்_உரை data, 200;
                let data = self.parse_expression();
                let status_code = if self.matches(Token::Comma) {
                    Some(self.parse_expression())
                } else {
                    None
                };
                self.expect(Token::Semicolon);
                Stmt::SendJSON { data, status_code }
            }
            Token::LParen => {
                // Handling (condition) enil { ... }
                let condition = self.parse_expression();
                self.expect(Token::RParen);
                
                let next = self.tokens.next().expect("Expected keyword after condition");
                match next {
                    Token::If => self.parse_if_remainder(condition),
                    Token::Loop => self.parse_loop_remainder(condition),
                    _ => panic!("Expected enil or cuRRu after condition"),
                }
            }
            _ => panic!("Unexpected token: {:?}", current_token),
        }
    }

    // Handles the body of 'enil' (if) and 'inREl' (else)
    fn parse_if_remainder(&mut self, condition: Expr) -> Stmt {
        self.expect(Token::LBrace);
        let mut then_branch = Vec::new();
        while !self.matches(Token::RBrace) {
            then_branch.push(self.parse_statement());
        }

        let mut else_branch = None;
        if self.matches(Token::Else) {
            self.expect(Token::LBrace);
            let mut branch = Vec::new();
            while !self.matches(Token::RBrace) {
                branch.push(self.parse_statement());
            }
            else_branch = Some(branch);
        }

        Stmt::If { condition, then_branch, else_branch }
    }

    // Handles the body of 'cuRRu' (loop)
    fn parse_loop_remainder(&mut self, condition: Expr) -> Stmt {
        self.expect(Token::LBrace);
        let mut body = Vec::new();
        while !self.matches(Token::RBrace) {
            body.push(self.parse_statement());
        }
        Stmt::Loop { condition, body }
    }

    // --- Expression Parsing (LLOPR & Precedence) ---

    fn parse_expression(&mut self) -> Expr {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Expr {
        let mut left = self.parse_additive();
        while let Some(t) = self.tokens.peek() {
            match t {
                Token::GreaterThan | Token::LessThan | Token::Equals | 
                Token::GreaterThanOrEqual | Token::LessThanOrEqual | Token::NotEquals => {
                    let op_token = self.tokens.next().unwrap();
                    let op = match op_token {
                        Token::GreaterThanOrEqual => ">=".to_string(),
                        Token::LessThanOrEqual => "<=".to_string(),
                        Token::GreaterThan => ">".to_string(),
                        Token::LessThan => "<".to_string(),
                        Token::Equals => "==".to_string(),
                        Token::NotEquals => "!=".to_string(),
                        _ => unreachable!(),
                    };
                    let right = self.parse_additive();
                    left = Expr::Comparison { left: Box::new(left), op, right: Box::new(right) };
                }
                _ => break,
            }
        }
        left
    }

    fn parse_additive(&mut self) -> Expr {
        let mut left = self.parse_term();
        while let Some(t) = self.tokens.peek() {
            match t {
                Token::Plus | Token::Minus => {
                    let op = if matches!(t, Token::Plus) { "+" } else { "-" }.to_string();
                    self.tokens.next();
                    let right = self.parse_term();
                    left = Expr::BinaryOp { op, left: Box::new(left), right: Box::new(right) };
                }
                Token::Ampersand => {
                    self.tokens.next();
                    let right = self.parse_term();
                    left = Expr::Concat { left: Box::new(left), right: Box::new(right) };
                }
                _ => break,
            }
        }
        left
    }

    fn parse_term(&mut self) -> Expr {
        let mut left = self.parse_factor();
        while let Some(t) = self.tokens.peek() {
            match t {
                Token::Multiply | Token::Divide => {
                    let op = if matches!(t, Token::Multiply) { "*" } else { "/" }.to_string();
                    self.tokens.next();
                    let right = self.parse_factor();
                    left = Expr::BinaryOp { op, left: Box::new(left), right: Box::new(right) };
                }
                _ => break,
            }
        }
        left
    }

    fn parse_factor(&mut self) -> Expr {
        // Base case for numbers/identifiers
        let token = self.tokens.next().unwrap();
        match token {
            Token::Number(n) => Expr::Number(*n),
            Token::Percentage(n) => Expr::Number(*n),
            Token::String(s) => Expr::String(s.clone()),
            Token::Identifier(name) => Expr::Variable(name.clone()),
            Token::LParen => {
                // Parenthesized expression
                let expr = self.parse_expression();
                self.expect(Token::RParen);
                expr
            }
            // Handle financial keywords as variable references
            t if self.is_identifier_like(t) => {
                let name = self.token_name(t);
                Expr::Variable(name)
            }
            _ => panic!("Expected factor, got {:?}", token),
        }
    }

    // Helpers to recognize identifier-like tokens (domain keywords or identifiers)
    fn is_identifier_like(&self, token: &Token) -> bool {
        match token {
            Token::Number(_) | Token::Percentage(_) | Token::String(_) => false,
            Token::If | Token::Else | Token::Loop | Token::Print | Token::Input => false,
            Token::Assign | Token::Plus | Token::Minus | Token::Multiply | Token::Divide | Token::Ampersand => false,
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
            Token::Route | Token::StartServer | Token::StopServer | Token::Response | Token::JSONBody => false,
            Token::Request | Token::Endpoint | Token::API | Token::Header | Token::Body | Token::Param | Token::QueryParam | Token::PathParam => false,
            Token::URL | Token::Host | Token::Port | Token::Method | Token::StatusCode | Token::StatusMessage | Token::Auth | Token::BearerToken | Token::ContentType | Token::Serve => false,
            Token::HttpGet | Token::HttpPost | Token::HttpPut | Token::HttpDelete | Token::HttpPatch | Token::HttpOptions | Token::HttpHead => false,
            // Security
            Token::Encrypt | Token::Decrypt | Token::Password | Token::EncryptionKey => false,
            // Financial & Accounting Keywords
            Token::Credit | Token::Debit | Token::Balance | Token::Rate | Token::Asset => false,
            Token::Liability | Token::Equity | Token::Revenue | Token::Expense | Token::Income => false,
            Token::Profit | Token::Loss | Token::Tax | Token::Net | Token::Gross => false,
            Token::Interest | Token::Ledger | Token::Journal | Token::Loan | Token::Finance => false,
            Token::Statement | Token::Valuation | Token::CreditCard | Token::Cash | Token::Bank => false,
            Token::Receivable | Token::Payable | Token::Vendor | Token::Customer => false,
            Token::Fixed | Token::Current | Token::NonCurrent | Token::Address | Token::Amount => false,
            Token::Currency | Token::Transaction | Token::Depreciation | Token::Amortization => false,
            Token::Appreciation | Token::Capital | Token::TrialBalance | Token::BalanceSheet => false,
            Token::IncomeStatement | Token::CashFlow | Token::IncomeTax | Token::GST | Token::ITR => false,
            _ if self.is_type_token(token) => false,
            Token::Identifier(_) => true,
            _ => true,
        }
    }

    fn is_type_token(&self, token: &Token) -> bool {
        matches!(token, 
            Token::IntegerType | Token::FloatType | Token::StringType | 
            Token::BoolType | Token::TextType | Token::ArrayType | 
            Token::DataType | Token::ObjectType | Token::DateType
        )
    }

    fn token_name(&self, token: &Token) -> String {
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
            Token::Table => "Table".to_string(),
            Token::Collection => "Collection".to_string(),
            Token::Row => "Row".to_string(),
            Token::Column => "Column".to_string(),
            // REST API tokens
            Token::HttpGet => "GET".to_string(),
            Token::HttpPost => "POST".to_string(),
            Token::HttpPut => "PUT".to_string(),
            Token::HttpDelete => "DELETE".to_string(),
            Token::HttpPatch => "PATCH".to_string(),
            Token::HttpOptions => "OPTIONS".to_string(),
            Token::HttpHead => "HEAD".to_string(),
            _ => format!("{:?}", token),
        }
    }

    fn expr_to_string(&self, expr: Expr) -> String {
        match expr {
            Expr::String(s) => s,
            Expr::Variable(name) => name,
            Expr::Number(n) => n.to_string(),
            Expr::BinaryOp { op, .. } => op,
            Expr::Comparison { op, .. } => op,
            Expr::Concat { .. } => "concat".to_string(),
        }
    }

    // Utility helpers
    fn expect(&mut self, expected: Token) {
        if self.tokens.next() != Some(&expected) {
            panic!("Expected {:?}", expected);
        }
    }

    fn matches(&mut self, expected: Token) -> bool {
        if self.tokens.peek() == Some(&&expected) {
            self.tokens.next();
            true
        } else {
            false
        }
    }
}