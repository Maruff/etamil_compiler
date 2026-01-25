// Database and Commands Example for eTamil Compiler
// Demonstrates relational DB, NoSQL, and command execution

use etamil_compiler::db::{RelationalDB, NoSQLDB, Database};
use etamil_compiler::commands::CommandExecutor;
use std::collections::HashMap;

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║   eTamil Database & Commands Demo                         ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    demo_relational_db();
    demo_nosql_db();
    demo_commands();
}

fn demo_relational_db() {
    println!("\n📊 RELATIONAL DATABASE DEMO");
    println!("════════════════════════════════════════════════════════════\n");

    let mut db = RelationalDB::sqlite();
    
    // Connect
    if let Err(e) = db.connect("sqlite::memory:") {
        eprintln!("Connection failed: {}", e);
        return;
    }

    // Create table
    if let Err(e) = db.create_table("students", vec![
        "id".to_string(),
        "name".to_string(),
        "score".to_string(),
    ]) {
        eprintln!("Create table failed: {}", e);
        return;
    }

    // Insert rows
    let mut row1 = HashMap::new();
    row1.insert("id".to_string(), "1".to_string());
    row1.insert("name".to_string(), "ராஜா".to_string());
    row1.insert("score".to_string(), "95".to_string());

    let mut row2 = HashMap::new();
    row2.insert("id".to_string(), "2".to_string());
    row2.insert("name".to_string(), "தேவி".to_string());
    row2.insert("score".to_string(), "88".to_string());

    db.insert("students", row1).ok();
    db.insert("students", row2).ok();

    // Select
    match db.select("students", None) {
        Ok(rows) => {
            println!("✅ Selected {} rows from students table", rows.len());
            for (i, row) in rows.iter().enumerate() {
                println!("   Row {}: {:?}", i + 1, row);
            }
        }
        Err(e) => eprintln!("Select failed: {}", e),
    }

    // List tables
    let tables = db.list_tables();
    println!("\n📋 Tables: {:?}", tables);

    // Execute SQL
    if let Ok(result) = db.execute_sql("SELECT * FROM students") {
        println!("✅ SQL Result: {}", result);
    }

    // Disconnect
    db.disconnect().ok();
    println!("\n✓ Relational DB demo complete");
}

fn demo_nosql_db() {
    println!("\n\n📂 NOSQL DATABASE DEMO");
    println!("════════════════════════════════════════════════════════════\n");

    let mut db = NoSQLDB::mongodb();

    // Connect
    if let Err(e) = db.connect("mongodb://localhost:27017") {
        eprintln!("Connection failed: {}", e);
        return;
    }

    // Create collection
    db.create_collection("users").ok();
    db.create_collection("products").ok();

    // Insert documents
    let user_doc = serde_json::json!({
        "name": "ராஜா",
        "email": "raja@example.com",
        "city": "சென்னை"
    });

    let product_doc = serde_json::json!({
        "title": "eTamil Compiler",
        "price": 100,
        "status": "active"
    });

    if let Ok(id) = db.insert_document("users", user_doc) {
        println!("✅ Inserted user with ID: {}", id);
    }

    if let Ok(id) = db.insert_document("products", product_doc) {
        println!("✅ Inserted product with ID: {}", id);
    }

    // Find documents
    match db.find("users", None) {
        Ok(docs) => {
            println!("✅ Found {} documents in users collection", docs.len());
            for doc in docs {
                println!("   Document: {}", doc);
            }
        }
        Err(e) => eprintln!("Find failed: {}", e),
    }

    // List collections
    let collections = db.list_collections();
    println!("\n📚 Collections: {:?}", collections);

    // Get stats
    if let Ok(stats) = db.stats("users") {
        println!("📊 Stats: {}", stats);
    }

    // Disconnect
    db.disconnect().ok();
    println!("\n✓ NoSQL DB demo complete");
}

fn demo_commands() {
    println!("\n\n⚙️  COMMAND EXECUTOR DEMO");
    println!("════════════════════════════════════════════════════════════\n");

    let mut executor = CommandExecutor::new();

    // Compiler commands
    println!("🔧 Compiler Commands:");
    match executor.compile("எண் x = 10;", "output.ll") {
        Ok(msg) => println!("   ✅ {}", msg),
        Err(e) => eprintln!("   ❌ {}", e),
    }

    match executor.validate("எண் x = 10;") {
        Ok(msg) => println!("   ✅ {}", msg),
        Err(e) => eprintln!("   ❌ {}", e),
    }

    // Variable management
    println!("\n📝 Variable Management:");
    executor.set_var("database_type", "SQLite");
    executor.set_var("connection_string", "sqlite::memory:");
    executor.set_var("table_name", "students");

    println!("   Variables:");
    for (name, value) in executor.list_vars() {
        println!("     {} = {}", name, value);
    }

    // Database connection
    println!("\n🗄️  Database Commands:");
    match executor.db_connect_relational("sqlite", "sqlite::memory:") {
        Ok(msg) => println!("   ✅ {}", msg),
        Err(e) => eprintln!("   ❌ {}", e),
    }

    match executor.db_list_tables() {
        Ok(tables) => println!("   ✅ Tables: {:?}", tables),
        Err(e) => eprintln!("   ❌ {}", e),
    }

    match executor.db_create_table("products", vec![
        "id".to_string(),
        "name".to_string(),
        "price".to_string(),
    ]) {
        Ok(msg) => println!("   ✅ {}", msg),
        Err(e) => eprintln!("   ❌ {}", e),
    }

    match executor.db_list_tables() {
        Ok(tables) => println!("   ✅ Tables after create: {:?}", tables),
        Err(e) => eprintln!("   ❌ {}", e),
    }

    // Query execution
    println!("\n🔍 Query Execution:");
    match executor.db_query_relational("SELECT * FROM products") {
        Ok(results) => println!("   ✅ Query Results: {:?}", results),
        Err(e) => eprintln!("   ❌ {}", e),
    }

    // Disconnect
    if let Ok(msg) = executor.db_disconnect_relational() {
        println!("\n   ✅ {}", msg);
    }

    // Help
    println!("\n📖 Help System:");
    println!("{}", executor.help(None));

    println!("\n✓ Command executor demo complete");
}
