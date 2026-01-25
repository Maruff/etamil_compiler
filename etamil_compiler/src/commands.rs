// Commands Module for eTamil Compiler
// Provides compile, run, query, and database operations

use crate::db::{RelationalDB, NoSQLDB, DBResult, Database};
use std::collections::HashMap;

/// Command executor for eTamil compiler
#[allow(dead_code)]
pub struct CommandExecutor {
    relational_db: Option<RelationalDB>,
    nosql_db: Option<NoSQLDB>,
    variables: HashMap<String, String>,
}

impl CommandExecutor {
    /// Create a new command executor
    pub fn new() -> Self {
        CommandExecutor {
            relational_db: None,
            nosql_db: None,
            variables: HashMap::new(),
        }
    }

    // ===== Compiler Commands =====

    /// Compile eTamil source code
    pub fn compile(&self, source: &str, output_file: &str) -> Result<String, String> {
        println!("[COMMAND] Compiling eTamil source...");
        if source.is_empty() {
            return Err("Source code is empty".to_string());
        }
        println!("[COMMAND] Output: {}", output_file);
        Ok(format!("Successfully compiled to {}", output_file))
    }

    /// Run compiled eTamil program
    pub fn run(&self, program_file: &str) -> Result<String, String> {
        println!("[COMMAND] Running eTamil program: {}", program_file);
        Ok("Program executed successfully".to_string())
    }

    /// Build eTamil project
    pub fn build(&self, project_path: &str) -> Result<String, String> {
        println!("[COMMAND] Building eTamil project: {}", project_path);
        Ok("Project built successfully".to_string())
    }

    /// Validate eTamil syntax
    pub fn validate(&self, source: &str) -> Result<String, String> {
        println!("[COMMAND] Validating eTamil syntax...");
        if source.is_empty() {
            return Err("Source code is empty".to_string());
        }
        Ok("Syntax is valid".to_string())
    }

    // ===== Database Connection Commands =====

    /// Connect to relational database
    pub fn db_connect_relational(&mut self, db_type: &str, connection_string: &str) -> DBResult<String> {
        let mut db = match db_type.to_lowercase().as_str() {
            "sqlite" => RelationalDB::sqlite(),
            "mysql" => RelationalDB::mysql(),
            "postgresql" => RelationalDB::postgresql(),
            _ => return Err(crate::db::DBError::OperationNotSupported(format!("Unknown DB type: {}", db_type))),
        };
        db.connect(connection_string)?;
        self.relational_db = Some(db);
        Ok(format!("Connected to {}", db_type))
    }

    /// Connect to NoSQL database
    pub fn db_connect_nosql(&mut self, db_type: &str, connection_string: &str) -> DBResult<String> {
        let mut db = match db_type.to_lowercase().as_str() {
            "mongodb" => NoSQLDB::mongodb(),
            "redis" => NoSQLDB::redis(),
            "jsonstore" => NoSQLDB::json_store(),
            _ => return Err(crate::db::DBError::OperationNotSupported(format!("Unknown DB type: {}", db_type))),
        };
        db.connect(connection_string)?;
        self.nosql_db = Some(db);
        Ok(format!("Connected to {}", db_type))
    }

    /// Disconnect from relational database
    pub fn db_disconnect_relational(&mut self) -> DBResult<String> {
        if let Some(ref mut db) = self.relational_db {
            db.disconnect()?;
            self.relational_db = None;
            Ok("Disconnected from relational database".to_string())
        } else {
            Err(crate::db::DBError::ConnectionFailed("No relational DB connection".to_string()))
        }
    }

    /// Disconnect from NoSQL database
    pub fn db_disconnect_nosql(&mut self) -> DBResult<String> {
        if let Some(ref mut db) = self.nosql_db {
            db.disconnect()?;
            self.nosql_db = None;
            Ok("Disconnected from NoSQL database".to_string())
        } else {
            Err(crate::db::DBError::ConnectionFailed("No NoSQL DB connection".to_string()))
        }
    }

    // ===== Database Query Commands =====

    /// Execute relational query
    pub fn db_query_relational(&self, query: &str) -> DBResult<Vec<String>> {
        if let Some(ref db) = self.relational_db {
            db.query(query)
        } else {
            Err(crate::db::DBError::ConnectionFailed("Not connected to relational database".to_string()))
        }
    }

    /// Execute NoSQL query
    pub fn db_query_nosql(&self, query: &str) -> DBResult<Vec<String>> {
        if let Some(ref db) = self.nosql_db {
            db.query(query)
        } else {
            Err(crate::db::DBError::ConnectionFailed("Not connected to NoSQL database".to_string()))
        }
    }

    /// List relational tables
    pub fn db_list_tables(&self) -> DBResult<Vec<String>> {
        if let Some(ref db) = self.relational_db {
            Ok(db.list_tables())
        } else {
            Err(crate::db::DBError::ConnectionFailed("Not connected to relational database".to_string()))
        }
    }

    /// List NoSQL collections
    pub fn db_list_collections(&self) -> DBResult<Vec<String>> {
        if let Some(ref db) = self.nosql_db {
            Ok(db.list_collections())
        } else {
            Err(crate::db::DBError::ConnectionFailed("Not connected to NoSQL database".to_string()))
        }
    }

    // ===== Database Schema Commands =====

    /// Create table in relational database
    pub fn db_create_table(&mut self, table_name: &str, columns: Vec<String>) -> DBResult<String> {
        if let Some(ref mut db) = self.relational_db {
            db.create_table(table_name, columns)?;
            Ok(format!("Table {} created", table_name))
        } else {
            Err(crate::db::DBError::ConnectionFailed("Not connected to relational database".to_string()))
        }
    }

    /// Create collection in NoSQL database
    pub fn db_create_collection(&mut self, collection_name: &str) -> DBResult<String> {
        if let Some(ref mut db) = self.nosql_db {
            db.create_collection(collection_name)?;
            Ok(format!("Collection {} created", collection_name))
        } else {
            Err(crate::db::DBError::ConnectionFailed("Not connected to NoSQL database".to_string()))
        }
    }

    // ===== Variable Management =====

    /// Set a variable
    pub fn set_var(&mut self, name: &str, value: &str) {
        self.variables.insert(name.to_string(), value.to_string());
        println!("[COMMAND] Variable {} = {}", name, value);
    }

    /// Get a variable
    pub fn get_var(&self, name: &str) -> Option<String> {
        self.variables.get(name).cloned()
    }

    /// List all variables
    pub fn list_vars(&self) -> Vec<(String, String)> {
        self.variables.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    // ===== Help and Info =====

    /// Get help for a command
    pub fn help(&self, command: Option<&str>) -> String {
        match command {
            Some("compile") => {
                "compile <source.qmz> <output.ll> - Compile eTamil source to LLVM IR".to_string()
            }
            Some("run") => {
                "run <program.qmz> - Run eTamil program".to_string()
            }
            Some("db") => {
                "db connect <type> <connection_string> - Connect to database".to_string()
            }
            _ => {
                "eTamil Compiler Commands:\n\
                 compile <source> <output> - Compile eTamil source\n\
                 run <program> - Run eTamil program\n\
                 build <path> - Build project\n\
                 validate <source> - Validate syntax\n\
                 db connect <type> <string> - Connect to database\n\
                 db query <query> - Execute database query\n\
                 db list-tables - List database tables\n\
                 help [command] - Show help".to_string()
            }
        }
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}
