// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
// Used _ as prefix for ttokens that are English keywords, and capitals letter for abbreviations to avoid conflicts

use logos::Logos;
use rust_decimal::Decimal;
// Needed for Decimal::from_str inside the literal callbacks below; the
// "unused import" warning you may see is emitted before macro expansion.
use std::str::FromStr;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
// A line comment must run to the end of the line, so the greedy repetition
// is intentional here; logos 0.16.1+ requires saying so explicitly.
#[logos(skip(r"//[^\n]*", allow_greedy = true))]
pub enum Token {
    // --- Core Financial & Accounting ---
    #[regex("வரவு|varavu")]
    Credit,
    #[regex("பற்று|paRRu")]
    Debit,
    #[regex("இருப்பு|iruppu")]
    Balance,
    #[regex("வீதம்|vIqam")]
    Rate,
    #[regex("சொத்து|coqqu|soqqu")]
    Asset,
    #[regex("பொறுப்பு|poRuppu")]
    Liability,
    #[regex("பங்கு|pawku")]
    Equity,
    #[regex("வருவாய்|varuvAy")]
    Revenue,
    #[regex("செலவு|celavu")]
    Expense,
    #[regex("வருமானம்|varumAZam")]
    Income,
    #[regex("பயன்|payaZ")]
    Profit,
    #[regex("இழப்பு|izappu")]
    Loss,
    #[regex("வரி|vari")]
    Tax,
    #[regex("நிகர|nikara")]
    Net,
    #[regex("மொத்த|moqqa")]
    Gross,
    #[regex("வட்டி|vatti")]
    Interest,
    #[regex("பேரேடு|pErEtu")]
    Ledger,
    #[regex("பதிவு|paqivu")]
    Journal,
    #[regex("கடன்|kataZ")]
    Loan,
    #[regex("நிதி|niqi")]
    Finance,
    #[regex("அறிக்கை|aRikkY")]
    Statement,
    #[regex("மதிப்பீடு|maqippItu|matippIDu")]
    Valuation,
    #[regex("கடன்_அட்டை|kataZ_attY")]
    CreditCard,
    #[regex("பணம்|paNam")]
    Cash,
    #[regex("வங்கி|vawki")]
    Bank,
    #[regex("பெறத்தக்க|peRaqqakka")]
    Receivable,
    #[regex("கொடுக்காத|kotukkAqa")]
    Payable,
    #[regex("விற்பனர்|viRpaZar")]
    Vendor,
    #[regex("வாங்குநர்|vAwkunar")]
    Customer,
    #[regex("நிலையான|nilYyAZa")]
    Fixed,
    #[regex("நடப்பு|natappu")]
    Current,
    #[regex("நடப்பிலில்லா|natappilillA")]
    NonCurrent,
    #[regex("முகவரி|mukavari")]
    Address,
    #[regex("தொகை|qokY|toqai")]
    Amount,
    #[regex("நாணயம்|nANayam")]
    Currency,
    #[regex("பரிவர்த்தனை|parivarqqaZY|parivarttaZai")]
    Transaction,
    #[regex("தேய்மானம்|qEymAZam")]
    Depreciation,
    #[regex("கடன்தீர்ப்பு|kataZqIrppu|kataZ_qIrppu")]
    Amortization,
    #[regex("மதிப்புயர்வு|maqippuyarvu|matippuyarvu")]
    Appreciation,
    #[regex("மூலதனம்|mUlaqaZam|mUlataZam")]
    Capital,
    #[regex("இருப்பாய்வு|iruppAyvu")]
    TrialBalance,
    #[regex("இருப்புநிலை|iruppunilY")]
    BalanceSheet,
    #[regex("வருமான_அறிக்கை|varumAZa_aRikkY|varumAZ_aRikkY")]
    IncomeStatement,
    #[regex("பணப்புழக்கம்|paNappuzakkam")]
    CashFlow,
    #[regex("வருமான_வரி|varumAZa_vari|varumAZ_vari")]
    IncomeTax,
    #[regex("வணிகவரி|vaNikavari|_GST")]
    GST,
    #[regex("வரியறிக்கை|variyaRikkY|_ITR")]
    ITR,

    // --- Transactions & Documents ---
    #[regex("விற்பனை|viRpaZY")]
    Sales,
    #[regex("கொள்முதல்|koLmuqal")]
    Purchase,
    #[regex("விலைப்பட்டியல்|vilYppattiyal")]
    Invoice,
    #[regex("ரசீது|racIqu")]
    Receipt,
    #[regex("காசோலை|kAcOlY")]
    Cheque,
    #[regex("ஒப்பந்தம்|oppanqam")]
    Contract,
    #[regex("சரக்கு|carakku")]
    Goods,
    #[regex("சரக்கிருப்பு|carakkiruppu")]
    Inventory,

    // --- Money Movement ---
    #[regex("வைப்பு|vYppu")]
    Deposit,
    #[regex("எடுப்பு|etuppu")]
    Withdrawal,
    #[regex("பரிமாற்றம்|parimARRam")]
    Transfer,
    #[regex("முன்பணம்|muZpaNam")]
    Advance,
    #[regex("நிலுவை|niluvY")]
    Outstanding,
    #[regex("தவணை|qavaNY")]
    Installment,
    #[regex("அசல்|acal")]
    Principal,

    // --- Income & Costs ---
    #[regex("ஊதியம்|Uqiyam")]
    Salary,
    #[regex("வாடகை|vAtakY")]
    Rent,
    #[regex("கட்டணம்|kattaNam")]
    Fee,
    #[regex("தள்ளுபடி|qaLLupati")]
    Discount,
    #[regex("அபராதம்|aparAqam")]
    Penalty,
    #[regex("காப்பீடு|kAppItu")]
    Insurance,
    #[regex("ஈவுத்தொகை|IvuqqokY")]
    Dividend,
    #[regex("முதலீடு|muqalItu")]
    Investment,
    #[regex("விலை|vilY")]
    Price,

    // --- Accounts, Reporting & Audit ---
    #[regex("கணக்கு|kaNakku")]
    Account,
    #[regex("தணிக்கை|qaNikkY")]
    Audit,
    #[regex("நிதியாண்டு|niqiyANtu")]
    FiscalYear,
    #[regex("காலாண்டு|kAlANtu")]
    Quarter,
    #[regex("பங்குதாரர்|pawkuqArar")]
    Shareholder,
    #[regex("நிதித்திட்டம்|niqiqqittam")]
    Budget,

    // --- Indian Taxation ---
    #[regex("வரிப்பிடித்தம்|varippitiqqam|_TDS")]
    TDS,
    #[regex("உள்ளீட்டுவரி|uLLIttuvari|_ITC")]
    InputTaxCredit,
    #[regex("விலக்கு|vilakku")]
    Deduction,
    #[regex("விதிவிலக்கு|viqivilakku")]
    Exemption,
    #[regex("அடுக்கு|atukku")]
    Slab,
    #[regex("சுங்கவரி|cuwkavari")]
    CustomsDuty,
    #[regex("கலால்வரி|kalAlvari")]
    ExciseDuty,
    #[regex("முத்திரைத்தீர்வை|muqqirYqqIrvY")]
    StampDuty,
    #[regex("மதிப்பீட்டாண்டு|maqippIttANtu")]
    AssessmentYear,

    // --- Variables & Data Types ---
    #[regex("எண்|eN")]
    IntegerType,
    #[regex("பின்னம்|piZZam")]
    FloatType,
    #[regex("சொல்|col")]
    StringType,
    #[regex("ஈர்ம|Irma")]
    BoolType,
    #[regex("உரை|urY")]
    TextType,
    #[regex("அணி|aNi")]
    ArrayType,
    #[regex("தரவு|qaravu")]
    DataType,
    #[regex("பொருள்|poruL")]
    ObjectType,
    #[regex("தேதி|qEqi")]
    DateType,
    #[regex("மெய்|mey")]
    True,
    #[regex("பொய்|poy")]
    False,
    #[regex("இன்மை|iZmY")]
    Null,
    #[regex("மாறி|mARi")]
    Let,
    #[regex("நிலை|nilY")]
    Const,

    // --- Control Flow (Your Updated Syntax) ---
    #[regex("எனில்|eZil")]
    If,
    #[regex("இன்றேல்|iZREl")]
    Else,
    #[regex("சுற்று|cuRRu")]
    Loop,
    #[regex("அச்சு|accu")]
    Print,
    #[regex("உள்ளிடு|uLLitu")]
    Input,

    // --- Functions ---
    #[regex("செயல்|ceyal|_fn")]
    Function,
    #[regex("திரும்பு|qirumpu|_return")]
    Return,

    // --- Iteration ---
    #[regex("ஒவ்வொரு|ovvoru|_each")]
    ForEach,
    #[regex("இல்|il|_in")]
    In,

    // --- Modules ---
    #[regex("இறக்கு|iRakku|_import")]
    Import,

    // --- File I/O Operations ---
    #[regex("கோப்பு|kOppu|_file")]
    File,
    #[regex("தரவுரை|qaravurY|_CSV")]
    CSV,
    #[regex("படி|pati|_read")]
    Read,
    #[regex("எழுது|ezuqu|_write")]
    Write,
    #[regex("திற|qiRa|_open")]
    Open,
    #[regex("மூடு|mUtu|_close")]
    Close,
    #[regex("கோப்பு_திற|kOppu_qiRa|_fileOpen")]
    FileOpen,
    #[regex("கோப்பு_மூடு|kOppu_mUtu|_fileClose")]
    FileClose,
    #[regex("கோப்பு_படி|kOppu_pati|_fileRead")]
    FileRead,
    #[regex("கோப்பு_எழுது|kOppu_ezuqu|_fileWrite")]
    FileWrite,
    #[regex("கோப்பு_நிரை|kOppu_nirY|_fileLines")]
    FileLines,
    #[regex("தரவுரை_படி|qaravurY_pati|_readCSV")]
    ReadCSV,
    #[regex("தரவுரை_எழுது|qaravurY_ezuqu|_writeCSV")]
    WriteCSV,

    // --- Database Connectivity Operations ---
    #[regex("தரவுதளம்|qaravuqaLam|_database")]
    Database,
    #[regex("தளம்_இணை|qaLam_iNY|_dbConnect")]
    DBConnect,
    #[regex("தளம்_பிரி|qaLam_piri|_dbDisconnect")]
    DBDisconnect,
    #[regex("தளம்_வினா|qaLam_viZA|_dbQuery")]
    DBQuery,
    #[regex("தளம்_செய்|qaLam_cey|_dbExecute")]
    DBExecute,
    #[regex("தளம்_தேடு|qaLam_qEtu|_dbSearch")]
    DBSearch,
    #[regex("தளம்_செருக|qaLam_ceruka|_dbInsert")]
    DBInsert,
    #[regex("தளம்_புதுப்பி|qaLam_puquppi|_dbUpdate")]
    DBUpdate,
    #[regex("தளம்_நீக்கு|qaLam_nIkku|_dbDelete")]
    DBDelete,

    // --- Database Types ---
    #[regex("கவி_மொழி|kavi_mozi|_SQL")]
    SQL,
    #[regex("தேடு_மொழி|qEtu_mozi|_NoSQL")]
    NoSQL,
    #[regex("சீகுலைட்|cIkulYt|_SQLite")]
    SQLite,
    #[regex("மைசீகுல்|mYcIkul|_MySQL")]
    MySQL,
    #[regex("போச்குரசீகுல்|pOckuracIkul|_PostgreSQL")]
    PostgreSQL,
    #[regex("மாங்கோடிபி|mAwkOtipi|_MongoDB")]
    MongoDB,
    #[regex("ரெடிஸ்|retis|_Redis")]
    Redis,
    #[regex("ஜேசான்|jEcAZ|_JSON")]
    JSONdb,

    // --- Database Operations ---
    #[regex("அட்டை|attY|_table")]
    Table,
    #[regex("தொகுப்பு|qokuppu|toguippu|_collection")]
    Collection,
    #[regex("நிரை|nirY|_row")]
    Row,
    #[regex("பத்தி|paqqi|_column")]
    Column,
    #[regex("விசை|vicY|_key")]
    Key,
    #[regex("தனிக_விசை|qaZika_vicY|taZik_vicY|_primaryKey")]
    PrimaryKey,
    #[regex("வெளி_விசை|veLi_vicY|_foreignKey")]
    ForeignKey,
    #[regex("குறியீடு|kuRiyItu|_index")]
    Index,
    #[regex("அட்டை_ஆக்கு|attY_Akku|_createTable")]
    CreateTable,
    #[regex("அட்டை_மாற்று|attY_mARRu|_alterTable")]
    AlterTable,
    #[regex("அட்டை_நீக்கு|attY_nIkku|_dropTable")]
    DropTable,

    // --- Database Clauses & Keywords ---
    #[regex("தேர்வெடு|qErvetu|_select")]
    Select,
    #[regex("இதனில்|iqaZil|_from")]
    From,
    #[regex("விதி|viqi|_where")]
    Where,
    #[regex("வரிசை|varicY|_orderBy")]
    OrderBy,
    #[regex("குழு|kuzu|_groupBy")]
    GroupBy,
    #[regex("சேர்|cEr|_join")]
    Join,
    #[regex("இடம்|itam|_left")]
    Left,
    #[regex("வலம்|valam|_right")]
    Right,
    #[regex("உள்|uL|_inner")]
    Inner,
    #[regex("வெளி|veLi|_outer")]
    Outer,
    #[regex("தனிக|qaZika|_distinct")]
    Distinct,
    #[regex("வரம்பு|varampu|_limit")]
    Limit,
    #[regex("ஈடு|Itu|_offset")]
    Offset,

    // --- REST API & HTTP ---
    #[regex("தள_இடை|qaLa_itY|_api")]
    API,
    #[regex("இறுதி_புள்ளி|iRuqi_puLLi|iraqi_pulli|_endpoint")]
    Endpoint,
    #[regex("வழி|vazi|vazhi|_route")]
    Route,
    // Runs a block on a timer under --server / --async. Romanization from
    // scripts/transliterate.py, never by hand.
    #[regex("இடைவெளி|itYveLi|_every")]
    Every,
    #[regex("கோரிக்கை|kOrikkY|_request")]
    Request,
    #[regex("பதில்|paqil|paDil|_response")]
    Response,
    #[regex("தலைப்பு|qalYppu|talYppu|_header")]
    Header,
    #[regex("உடல்|utal|uqal|_body")]
    Body,
    #[regex("அளவுரு|aLavuru|_param")]
    Param,
    #[regex("வினா_அளவுரு|viZA_aLavuru|_queryParam")]
    QueryParam,
    #[regex("பாதை_அளவுரு|pAqY_aLavuru|pAtY_aLavuru|_pathParam")]
    PathParam,
    #[regex("ஜேசான்_உரை|jEcAZ_urY|_jsonBody")]
    JSONBody,
    #[regex("உரலி|urali|_url")]
    URL,
    #[regex("புரவலன்|puravalaZ|_host")]
    Host,
    #[regex("குதை|kuqY|_port")]
    Port,
    #[regex("முறை|muRY|_method")]
    Method,
    #[regex("பெறு|peRu|_get")]
    HttpGet,
    #[regex("பதி|paqi|_post")]
    HttpPost,
    #[regex("இடு|itu|_put")]
    HttpPut,
    #[regex("அழி|azi|_delete")]
    HttpDelete,
    #[regex("ஒட்டு|ottu|_patch")]
    HttpPatch,
    #[regex("தெரிவு|qerivu|_options")]
    HttpOptions,
    #[regex("தலை|qalY|talY|_head")]
    HttpHead,
    #[regex("நிலை_குறி|nilY_kuRi|_statusCode")]
    StatusCode,
    #[regex("நிலை_செய்தி|nilY_ceyqi|_statusMessage")]
    StatusMessage,
    #[regex("உறுதி|uRuqi|_auth")]
    Auth,
    #[regex("குறிதாங்கி|kuRiqAwki|_bearerToken")]
    BearerToken,
    #[regex("உரை_வகை|urY_vakY|_contentType")]
    ContentType,
    #[regex("சேவை|cEvY|_serve")]
    Serve,
    #[regex("வழங்கி_தொடங்கு|vazawki_qotawku|vazawki_toqotawku|_startServer")]
    StartServer,
    #[regex("வழங்கி_நிறுத்து|vazawki_niRuqqu|_stopServer")]
    StopServer,

    // --- Encryption & Security ---
    #[regex("மறை|maRY|_encrypt")]
    Encrypt,
    #[regex("வெளிப்படு|veLippatu|_decrypt")]
    Decrypt,
    #[regex("குறிமுறை|kuRimuRY|_password")]
    Password,
    #[regex("மறை_விசை|maRY_vicY|_encryptionKey")]
    EncryptionKey,

    // --- Logical Operators ---
    #[regex("மற்றும்|maRRum|_and")]
    And,
    #[regex("அல்லது|allaqu|_or")]
    Or,
    #[regex("இல்லை|illY|_not")]
    Not,

    // --- Literals & Identifiers ---
    // Numbers are fixed-point decimals, so 20% is exactly 0.20 and money
    // arithmetic does not drift the way f64 does.
    #[regex(r"[0-9]+(\.[0-9]+)?%", |lex| {
        let s = lex.slice();
        let num_str = &s[..s.len()-1];
        Decimal::from_str(num_str).ok().map(|n| n / Decimal::from(100))
    })]
    Percentage(Decimal),
    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| Decimal::from_str(lex.slice()).ok())]
    Number(Decimal),
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let raw = lex.slice();
        unescape(&raw[1..raw.len() - 1])
    })]
    String(String),
    #[regex(r"[\u0B80-\u0BFFa-zA-Z_][\u0B80-\u0BFFa-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // --- Comparison Operators (New: Required for Conditionals) ---
    #[token(">")]
    GreaterThan,
    #[token("<")]
    LessThan,
    #[token("==")]
    Equals,
    #[token("!=")]
    NotEquals,
    #[token(">=")]
    GreaterThanOrEqual,
    #[token("<=")]
    LessThanOrEqual,

    // --- Operators & Symbols ---
    #[token("=")]
    Assign,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Multiply,
    #[token("/")]
    Divide,
    #[token("&")]
    Ampersand,
    #[token("?")]
    Question,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(".")]
    Dot,
    #[token(":")]
    Colon,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
}

/// Turn the escape sequences in a string literal into the characters they name.
///
/// The literal text used to be kept exactly as written, so `"a\nb"` was four
/// characters and there was **no way at all** to put a double quote inside a
/// string. That made a JSON serializer impossible to write in eTamil, and it
/// silently broke the three places in the examples that already assumed
/// escapes worked.
///
/// An unrecognized escape keeps both characters rather than dropping the
/// backslash, so a Windows path written `"C:\kaNakku"` still reads back whole
/// instead of quietly becoming `C:aNakku`.
fn unescape(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            // A trailing backslash cannot occur: the regex requires an escape
            // to be followed by a character. Kept literal rather than panicking.
            None => out.push('\\'),
        }
    }

    out
}

/// One token, with where it came from and exactly how it was written.
///
/// The lexer used to hand back a bare `Vec<Token>`, which discarded two things
/// the rest of the compiler needed. Without the text, a keyword used as a name
/// could only be recorded under its token name, so `வங்கி = 5` created a
/// variable called `Bank` — a Tamil author's chosen name silently anglicised.
/// Without the position, the parser had nowhere to point when a statement went
/// wrong, so a missing semicolon was reported as `Expected Semicolon` and
/// nothing else.
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned {
    pub token: Token,
    /// 1-based, counting characters rather than bytes so Tamil text reports
    /// sensible columns.
    pub line: usize,
    pub column: usize,
    /// The source text this token matched, as written.
    pub text: String,
}

/// Walks forward through the source once while positions are handed out.
///
/// logos yields tokens in order, so each position lookup can carry on from
/// the last one. Rescanning from the start every time would make tokenizing
/// quadratic in the length of the file.
struct LineCursor {
    offset: usize,
    line: usize,
    column: usize,
}

impl LineCursor {
    fn new() -> Self {
        LineCursor {
            offset: 0,
            line: 1,
            column: 1,
        }
    }

    fn at(&mut self, source: &str, target: usize) -> (usize, usize) {
        if target <= self.offset {
            return (self.line, self.column);
        }

        for (index, ch) in source[self.offset..].char_indices() {
            if self.offset + index >= target {
                break;
            }
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }

        self.offset = target;
        (self.line, self.column)
    }
}

/// A lexical error, carrying the position of the offending input.
#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub line: usize,
    pub column: usize,
    pub text: String,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "வரி {}, நெடுவரிசை {}: அறியப்படாத உள்ளீடு '{}'  (line {}, column {}: unrecognized input '{}')",
            self.line, self.column, self.text, self.line, self.column, self.text
        )
    }
}

/// Tokenize a string of eTamil code.
///
/// Every slice the lexer cannot recognize is reported with its position
/// rather than silently discarded, so a mistyped character is a visible
/// error instead of a program that quietly means something else.
pub fn tokenize(source: &str) -> Result<Vec<Spanned>, Vec<LexError>> {
    // Windows editors — Notepad, and VS Code in some configurations — save
    // UTF-8 with a byte-order mark. It is invisible, it is the first thing in
    // the file, and treating it as a lexical error made every such program
    // fail on its first line with an error naming a character the author
    // cannot see. Tamil source is exactly the kind of file that gets saved
    // that way, so the mark is skipped rather than reported.
    let source = source.strip_prefix('\u{FEFF}').unwrap_or(source);

    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    let mut lexer = Token::lexer(source);
    let mut cursor = LineCursor::new();

    while let Some(result) = lexer.next() {
        let (line, column) = cursor.at(source, lexer.span().start);
        let text = lexer.slice().to_string();

        match result {
            Ok(token) => tokens.push(Spanned {
                token,
                line,
                column,
                text,
            }),
            Err(_) => errors.push(LexError { line, column, text }),
        }
    }

    if errors.is_empty() {
        Ok(tokens)
    } else {
        Err(errors)
    }
}
