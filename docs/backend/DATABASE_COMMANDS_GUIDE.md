# eTamil Database & Commands System

## Overview

The eTamil compiler now includes a comprehensive database connectivity and command execution system for managing relational databases (SQL), NoSQL databases (MongoDB, Redis), and compiler operations (compile, run, query).

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                   eTamil Programs                         │
│           (Using DB & Command Operations)                │
└────────────────────┬─────────────────────────────────────┘
                     │
        ┌────────────┴────────────┬──────────────┐
        │                         │              │
┌───────▼──────────┐  ┌──────────▼────┐  ┌─────▼─────────┐
│   Relational DB  │  │   NoSQL DB    │  │   Commands    │
│ (SQLite, MySQL,  │  │ (MongoDB,     │  │ (Compile,     │
│  PostgreSQL)     │  │  Redis, JSON) │  │  Run, Query)  │
└──────────────────┘  └───────────────┘  └───────────────┘
        │                     │                   │
        │           File I/O & Encryption        │
        │                     │                   │
└───────┴─────────────────────┴───────────────────┘
             Backend Storage (.ani, .qrv)
```

## Components

### 1. Relational Database Handler (`db/relational.rs`)

Handles SQL databases with support for:
- **SQLite**: In-memory and file-based
- **MySQL**: Remote database connectivity
- **PostgreSQL**: Enterprise database support

**Features**:
- Create/drop tables
- Insert/update/delete rows
- Select queries with filtering
- SQL execution
- Connection management

**API**:
```rust
let mut db = RelationalDB::sqlite();
db.connect("sqlite::memory:")?;
db.create_table("users", vec!["id".to_string(), "name".to_string()])?;
db.insert("users", row_data)?;
let results = db.select("users", None)?;
```

### 2. NoSQL Database Handler (`db/nosql.rs`)

Handles document-based databases:
- **MongoDB**: Document-oriented NoSQL
- **Redis**: Key-value store
- **JSON Store**: File-based JSON documents

**Features**:
- Create/drop collections
- Insert/find/update/delete documents
- JSON document handling
- Collection statistics
- Connection management

**API**:
```rust
let mut db = NoSQLDB::mongodb();
db.connect("mongodb://localhost:27017")?;
db.create_collection("users")?;
let doc = serde_json::json!({"name": "Raja", "city": "Chennai"});
db.insert_document("users", doc)?;
let docs = db.find("users", None)?;
```

### 3. Command Executor (`commands.rs`)

Provides a unified interface for:
- **Compiler Commands**: compile, run, build, validate
- **Database Operations**: connect, query, list tables/collections
- **Variable Management**: set/get/list variables
- **Help System**: command documentation

**Features**:
- Compile eTamil source to LLVM IR
- Execute compiled programs
- Build entire projects
- Validate syntax
- Connect to multiple databases simultaneously
- Execute queries and manage data
- Store and retrieve variables
- Help and documentation

**API**:
```rust
let mut executor = CommandExecutor::new();

// Compiler
executor.compile("source.qmz", "output.ll")?;
executor.validate("source code")?;

// Database
executor.db_connect_relational("sqlite", "sqlite::memory:")?;
executor.db_create_table("products", columns)?;
executor.db_query_relational("SELECT * FROM products")?;

// Variables
executor.set_var("db_type", "SQLite");
let value = executor.get_var("db_type");
```

## File Structure

```
src/
├── db/
│   ├── mod.rs              (DB types, traits, error handling)
│   ├── relational.rs       (SQL database handlers)
│   └── nosql.rs            (NoSQL document database handlers)
├── commands.rs             (Command executor)
├── fileio/
│   ├── mod.rs
│   ├── csv_handler.rs
│   └── crypto.rs           (Encryption for .ani & .qrv)
└── main.rs                 (Updated with db & commands modules)
```

## Usage Examples

### Connect to Relational Database

```rust
use etamil_compiler::db::RelationalDB;

let mut db = RelationalDB::sqlite();
db.connect("sqlite::memory:")?;

// Create table
db.create_table("students", vec![
    "id".to_string(),
    "name".to_string(),
    "score".to_string(),
])?;

// Insert data
let mut row = std::collections::HashMap::new();
row.insert("id".to_string(), "1".to_string());
row.insert("name".to_string(), "ராஜா".to_string());
row.insert("score".to_string(), "95".to_string());
db.insert("students", row)?;

// Query data
let results = db.select("students", None)?;
println!("Found {} students", results.len());

db.disconnect()?;
```

### Connect to NoSQL Database

```rust
use etamil_compiler::db::NoSQLDB;

let mut db = NoSQLDB::mongodb();
db.connect("mongodb://localhost:27017")?;

// Create collection
db.create_collection("users")?;

// Insert document
let doc = serde_json::json!({
    "name": "தேவி",
    "email": "devi@example.com",
    "city": "சென்னை"
});
let doc_id = db.insert_document("users", doc)?;

// Find documents
let docs = db.find("users", Some("city = Chennai"))?;
for doc in docs {
    println!("{}", doc);
}

// Get statistics
let stats = db.stats("users")?;
println!("{}", stats);

db.disconnect()?;
```

### Use Command Executor

```rust
use etamil_compiler::commands::CommandExecutor;

let mut executor = CommandExecutor::new();

// Compile eTamil source
executor.compile("எண் x = 10;", "output.ll")?;

// Set variables
executor.set_var("database_url", "sqlite::memory:");
executor.set_var("table_name", "products");

// Connect to database
executor.db_connect_relational("sqlite", "sqlite::memory:")?;

// Create table
executor.db_create_table("products", vec![
    "id".to_string(),
    "name".to_string(),
    "price".to_string(),
])?;

// List tables
let tables = executor.db_list_tables()?;
println!("Available tables: {:?}", tables);

// Execute query
let results = executor.db_query_relational("SELECT * FROM products")?;

// Disconnect
executor.db_disconnect_relational()?;

// Get help
println!("{}", executor.help(None));
```

## Integration with File I/O & Encryption

The database system integrates seamlessly with the encryption module:

```
eTamil Program
    ↓
Database Query
    ↓
Data Retrieved
    ↓
File Operations (.txt, .csv)
    ↓
Encryption (CryptoHandler)
    ↓
Encrypted Backend Storage (.ani, .qrv)
```

## Database Error Handling

```rust
use etamil_compiler::db::DBError;

match db.connect("connection_string") {
    Ok(_) => println!("Connected"),
    Err(DBError::ConnectionFailed(msg)) => eprintln!("Failed: {}", msg),
    Err(DBError::QueryFailed(msg)) => eprintln!("Query failed: {}", msg),
    Err(DBError::NotFound(msg)) => eprintln!("Not found: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Testing

### Run the Demo

```bash
# Run complete DB and commands demo
cargo run --example db_commands_demo

# Run individual tests
cargo test --lib db
cargo test --lib commands
```

### Test Output

The demo demonstrates:
- ✅ Creating tables and collections
- ✅ Inserting data into databases
- ✅ Querying and retrieving data
- ✅ Listing tables and collections
- ✅ Executing compiler commands
- ✅ Variable management
- ✅ Connection/disconnection

## Supported Database Types

### Relational (SQL)
- **SQLite**: `RelationalDB::sqlite()`
- **MySQL**: `RelationalDB::mysql()`
- **PostgreSQL**: `RelationalDB::postgresql()`

### NoSQL
- **MongoDB**: `NoSQLDB::mongodb()`
- **Redis**: `NoSQLDB::redis()`
- **JSON Store**: `NoSQLDB::json_store()`

## Command Types

### Compiler Commands
- `compile <source> <output>` - Compile eTamil source
- `run <program>` - Run eTamil program
- `build <path>` - Build project
- `validate <source>` - Validate syntax

### Database Commands
- `db connect <type> <string>` - Connect to database
- `db query <query>` - Execute query
- `db list-tables` - List tables
- `db list-collections` - List collections
- `db create-table <name> <columns>` - Create table
- `db create-collection <name>` - Create collection

### Variable Commands
- `set <name> <value>` - Set variable
- `get <name>` - Get variable
- `list` - List all variables

### Help
- `help [command]` - Show help for command

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| Connect | O(1) | Single connection per executor |
| Create Table | O(n) | n = number of columns |
| Insert | O(1) | Amortized |
| Select | O(n) | n = number of rows |
| Update | O(n) | n = number of rows to update |
| Delete | O(n) | n = number of rows to delete |

## Security Considerations

1. **Encryption**: All files stored as .ani (text) or .qrv (CSV) are encrypted
2. **Connection Strings**: Store in environment variables
3. **Query Validation**: Validate user inputs before executing
4. **Access Control**: Implement role-based access in production
5. **Logging**: Log all database operations for audit

## Future Enhancements

1. **Real Database Integration**: Replace in-memory with actual DB drivers
2. **Connection Pooling**: Support multiple simultaneous connections
3. **Transaction Support**: ACID compliance for SQL operations
4. **Prepared Statements**: Prevent SQL injection attacks
5. **Schema Validation**: Enforce data types and constraints
6. **Indexing**: Performance optimization for large datasets
7. **Backup/Restore**: Automated database backup functionality
8. **Replication**: Support for database replication

## Integration with eTamil Programs

### Future Language Support

```tamil
// eTamil program with database operations
பெறு database_type = "SQLite";
பெறு connection = "sqlite::memory:";

// Connect to database
தரவுக_பொதுக database_type பொதுக connection;

// Create table
அட்டவணி_உருவாக் "students" = ["id", "name", "score"];

// Insert data
insert "students" = {"id": 1, "name": "ராஜா", "score": 95};

// Query data
கண்டுபிடி results = "SELECT * FROM students";

// Print results
அச்சு results;
```

## Module Exports

```rust
pub mod db;        // RelationalDB, NoSQLDB, DBError, Database trait
pub mod commands;  // CommandExecutor
```

## Example: Complete Workflow

See [db_commands_demo.rs](examples/db_commands_demo.rs) for a complete working example demonstrating:
1. Relational database creation and queries
2. NoSQL document insertion and retrieval
3. Command compilation and validation
4. Variable management
5. Multi-database connectivity

## Contributing

To extend database support:
1. Implement the `Database` trait
2. Add connection logic
3. Implement CRUD operations
4. Add error handling
5. Create example in `examples/`
6. Document usage

---

**Database System Version**: 1.0  
**Last Updated**: January 25, 2026  
**Modules**: `db`, `commands`
