// Used _ as prefix for ttokens that are English keywords, and capitals letter for abbreviations to avoid conflicts

use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(skip r"//[^\n]*")] 
pub enum Token {
    // --- Core Financial & Accounting ---
    #[regex("வரவு|varavu")] Credit,
    #[regex("பற்று|paRRu")] Debit,
    #[regex("இருப்பு|iruppu")] Balance,
    #[regex("வீதம்|vIqam")] Rate,
    #[regex("சொத்து|soqqu")] Asset,
    #[regex("பொறுப்பு|poRuppu")] Liability,
    #[regex("பங்கு|paNgu")] Equity,
    #[regex("வருவாய்|varuvAy")] Revenue,
    #[regex("செலவு|celavu")] Expense,
    #[regex("வருமானம்|varumAnam")] Income,
    #[regex("பயன்|payan")] Profit,
    #[regex("இழப்பு|izappu")] Loss,
    #[regex("வரி|vari")] Tax,
    #[regex("நிகர|Nikara")] Net,
    #[regex("மொத்த|moqqa")] Gross,
    #[regex("வட்டி|vatti")] Interest,
    #[regex("பேரேடு|pErEtu")] Ledger,
    #[regex("பதிவு|paqivu")] Journal,
    #[regex("கடன்|katan")] Loan,
    #[regex("நிதி|niqi")] Finance,
    #[regex("அறிக்கை|aRikkY")] Statement,
    #[regex("மதிப்பீடு|matippIDu")] Valuation,
    #[regex("கடன்_அட்டை|katan_attY")] CreditCard,
    #[regex("பணம்|paNam")] Cash,
    #[regex("வங்கி|vawki")] Bank,
    #[regex("பெறத்தக்க|peRaqqakka")] Receivable,
    #[regex("கொடுக்காத|kotukkAqa")] Payable,
    #[regex("விற்பனர்|viRpanar")] Vendor,
    #[regex("வாங்குநர்|vAwkuNar")] Customer,
    #[regex("நிலையான|nilYyAna")] Fixed,
    #[regex("நடப்பு|natappu")] Current,
    #[regex("நடப்பிலில்லா|natappilillA")] NonCurrent,
    #[regex("முகவரி|mukavari")] Address,
    #[regex("தொகை|toqai")] Amount,
    #[regex("நாணயம்|nANayam")] Currency,
    #[regex("பரிவர்த்தனை|parivarttanai")] Transaction,
    #[regex("தேய்மானம்|qEymAnam")] Depreciation,
    #[regex("கடன்தீர்ப்பு|katan_qIrppu")] Amortization,
    #[regex("மதிப்புயர்வு|matippuyarvu")] Appreciation,
    #[regex("மூலதனம்|mUlatanam")] Capital,
    #[regex("இருப்பாய்வு|iruppAyvu")] TrialBalance,
    #[regex("இருப்புநிலை|iruppunilY")] BalanceSheet,
    #[regex("வருமான_அறிக்கை|varumAn_aRikkY")] IncomeStatement,
    #[regex("பணப்புழக்கம்|paNappuzakkam")] CashFlow,
    #[regex("வருமான_வரி|varumAn_vari")] IncomeTax,
    #[regex("வணிகவரி|vaNikavari|_GST")] GST,
    #[regex("வரியறிக்கை|variyaRikkY|_ITR")] ITR,

    // --- Variables & Data Types ---
    #[regex("எண்|eN")] IntegerType,
    #[regex("பின்னம்|pinnam")] FloatType,
    #[regex("சொல்|col")] StringType,
    #[regex("ஈர்ம|Irma")] BoolType,
    #[regex("உரை|urY")] TextType,
    #[regex("அணி|aNi")] ArrayType,
    #[regex("தரவு|qaravu")] DataType,
    #[regex("பொருள்|poruL")] ObjectType,
    #[regex("தேதி|qEqi")] DateType,
    #[regex("மெய்|mey")] True,
    #[regex("பொய்|poy")] False,
    #[regex("இன்மை|inmY")] Null,
    #[regex("மாறி|mARi")] Let,
    #[regex("நிலை|nilY")] Const,

    // --- Control Flow (Your Updated Syntax) ---
    #[regex("எனில்|enil")] If,
    #[regex("இன்றேல்|inREl")] Else,
    #[regex("சுற்று|cuRRu")] Loop,
    #[regex("அச்சு|accu")] Print,
    #[regex("உள்ளிடு|uLLitu")] Input,

    // --- File I/O Operations ---
    #[regex("கோப்பு|kOppu|_file")] File,
    #[regex("தரவுரை|qaravurY|_CSV")] CSV,
    #[regex("படி|pati|_read")] Read,
    #[regex("எழுது|ezuqu|_write")] Write,
    #[regex("திற|qiRa|_open")] Open,
    #[regex("மூடு|mUtu|_close")] Close,
    #[regex("கோப்பு_திற|kOppu_qiRa|_fileOpen")] FileOpen,
    #[regex("கோப்பு_மூடு|kOppu_mUtu|_fileClose")] FileClose,
    #[regex("கோப்பு_படி|kOppu_pati|_fileRead")] FileRead,
    #[regex("கோப்பு_எழுது|kOppu_ezuqu|_fileWrite")] FileWrite,
    #[regex("கோப்பு_நிரை|kOppu_NirY|fileLines")] FileLines,
    #[regex("தரவுரை_படி|qaravurY_pati|_readCSV")] ReadCSV,
    #[regex("தரவுரை_எழுது|qaravurY_ezuqu|_writeCSV")] WriteCSV,

    // --- Database Connectivity Operations ---
    #[regex("தரவுதளம்|qaravuqaLam|_database")] Database,
    #[regex("தளம்_இணை|qaLam_iNY|_dbConnect")] DBConnect,
    #[regex("தளம்_பிரி|qaLam_piri|_dbDisconnect")] DBDisconnect,
    #[regex("தளம்_வினா|qaLam_vinA|_dbQuery")] DBQuery,
    #[regex("தளம்_செய்|qaLam_cey|_dbExecute")] DBExecute,
    #[regex("தளம்_தேடு|qaLam_qEtu|_dbSearch")] DBSearch,
    #[regex("தளம்_செருக|qaLam_ceruka|_dbInsert")] DBInsert,
    #[regex("தளம்_புதுப்பி|qaLam_puquppi|_dbUpdate")] DBUpdate,
    #[regex("தளம்_நீக்கு|qaLam_nIkku|_dbDelete")] DBDelete,
    
    // --- Database Types ---
    #[regex("கவி_மொழி|kavi_mozi|_SQL")] SQL,
    #[regex("தேடு_மொழி|qEtu_mozi|_NoSQL")] NoSQL,
    #[regex("சீகுலைட்|cIkulYt|_SQLite")] SQLite,
    #[regex("மைசீகுல்|mYcIkul|_MySQL")] MySQL,
    #[regex("போச்குரசீகுல்|pOckuracIkul|_PostgreSQL")] PostgreSQL,
    #[regex("மாங்கோடிபி|mAwkOṭipi|_MongoDB")] MongoDB,
    #[regex("ரெடிஸ்|retis|_Redis")] Redis,
    #[regex("ஜேசான்|jEcAn|_JSON")] JSONdb,
    
    // --- Database Operations ---
    #[regex("அட்டை|attY|_table")] Table,
    #[regex("தொகுப்பு|toguippu|_collection")] Collection,
    #[regex("நிரை|NirY|_row")] Row,
    #[regex("பத்தி|paqqi|_column")] Column,
    #[regex("விசை|vicY|_key")] Key,
    #[regex("தனிக_விசை|tanik_vicY|_primaryKey")] PrimaryKey,
    #[regex("வெளி_விசை|veLi_vicY|_foreignKey")] ForeignKey,
    #[regex("குறியீடு|kuRiyItu|_index")] Index,
    #[regex("அட்டை_ஆக்கு|attY_Akku|_createTable")] CreateTable,
    #[regex("அட்டை_மாற்று|attY_mARRu|_alterTable")] AlterTable,
    #[regex("அட்டை_நீக்கு|attY_nIkku|_dropTable")] DropTable,
    
    // --- Database Clauses & Keywords ---
    #[regex("தேர்வெடு|qErvetu|_select")] Select,
    #[regex("இதனில்|iqanil|_from")] From,
    #[regex("விதி|viqi|_where")] Where,
    #[regex("வரிசை|varicY|_orderBy")] OrderBy,
    #[regex("குழு|kuzu|_groupBy")] GroupBy,
    #[regex("சேர்|cEr|_join")] Join,
    #[regex("இடம்|itam|_left")] Left,
    #[regex("வலம்|valam|_right")] Right,
    #[regex("உள்|uL|_inner")] Inner,
    #[regex("வெளி|veLi|_outer")] Outer,
    #[regex("தனிக|qanika|_distinct")] Distinct,
    #[regex("வரம்பு|varampu|_limit")] Limit,
    #[regex("ஈடு|Itu|_offset")] Offset,

    // --- REST API & HTTP ---
    #[regex("தள_இடை|qaLa_itY|_api")] API,
    #[regex("இறுதி_புள்ளி|iraqi_pulli|_endpoint")] Endpoint,
    #[regex("வழி|vazhi|_route")] Route,
    #[regex("கோரிக்கை|kOrikkY|_request")] Request,
    #[regex("பதில்|paDil|_response")] Response,
    #[regex("தலைப்பு|talYppu|_header")] Header,
    #[regex("உடல்|uqal|_body")] Body,
    #[regex("அளவுரு|aLavuru|_param")] Param,
    #[regex("வினா_அளவுரு|vinA_aLavuru|_queryParam")] QueryParam,
    #[regex("பாதை_அளவுரு|pAtY_aLavuru|_pathParam")] PathParam,
    #[regex("ஜேசான்_உரை|jEcAn_urY|_jsonBody")] JSONBody,
    #[regex("உரலி|urali|_url")] URL,
    #[regex("புரவலன்|puravalan|_host")] Host,
    #[regex("குதை|kuqY|_port")] Port,
    #[regex("முறை|muRY|_method")] Method,
    #[regex("பெறு|peRu|_get")] HttpGet,
    #[regex("பதி|paqi|_post")] HttpPost,
    #[regex("இடு|itu|_put")] HttpPut,
    #[regex("அழி|azi|_delete")] HttpDelete,
    #[regex("ஒட்டு|ottu|_patch")] HttpPatch,
    #[regex("தெரிவு|qerivu|_options")] HttpOptions,
    #[regex("தலை|talY|_head")] HttpHead,
    #[regex("நிலை_குறி|nilY_kuRi|_statusCode")] StatusCode,
    #[regex("நிலை_செய்தி|nilai_ceyqi|_statusMessage")] StatusMessage,
    #[regex("உறுதி|uRuqi|_auth")] Auth,
    #[regex("குறிதாங்கி|kuRiqAwki|_bearerToken")] BearerToken,
    #[regex("உரை_வகை|urY_vakY|_contentType")] ContentType,
    #[regex("சேவை|cEvY|_serve")] Serve,
    #[regex("வழங்கி_தொடங்கு|vazawki_toqotawku|_startServer")] StartServer,
    #[regex("வழங்கி_நிறுத்து|vazawki_niRuqqu|_stopServer")] StopServer,
    
    // --- Encryption & Security ---
    #[regex("மறை|maRY|_encrypt")] Encrypt,
    #[regex("வெளிப்படு|veLippatu|_decrypt")] Decrypt,
    #[regex("குறிமுறை|kuRimuRY|_password")] Password,
    #[regex("மறை_விசை|maRY_vicY|_encryptionKey")] EncryptionKey,

    // --- Literals & Identifiers ---
    #[regex(r"[0-9]+(\.[0-9]+)?%", |lex| {
        let s = lex.slice();
        let num_str = &s[..s.len()-1];
        num_str.parse::<f64>().ok().map(|n| n / 100.0)
    })] Percentage(f64),
    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())] Number(f64),
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })] String(String),
    #[regex(r"[\u0B80-\u0BFFa-zA-Z_][\u0B80-\u0BFFa-zA-Z0-9_]*", |lex| lex.slice().to_string())] Identifier(String),

    // --- Comparison Operators (New: Required for Conditionals) ---
    #[token(">")] GreaterThan,
    #[token("<")] LessThan,
    #[token("==")] Equals,
    #[token("!=")] NotEquals,
    #[token(">=")] GreaterThanOrEqual,
    #[token("<=")] LessThanOrEqual,

    // --- Operators & Symbols ---
    #[token("=")] Assign,
    #[token("+")] Plus,
    #[token("-")] Minus,
    #[token("*")] Multiply,
    #[token("/")] Divide,
    #[token("&")] Ampersand,
    #[token("(")] LParen,
    #[token(")")] RParen,
    #[token("{")] LBrace,
    #[token("}")] RBrace,
    #[token(",")] Comma,
    #[token(";")] Semicolon,
}


/// Helper function to tokenize a string of eTamil code
pub fn tokenize(source: &str) -> Vec<Token> {
    Token::lexer(source)
        .filter_map(|token| token.ok())
        .collect()
}
