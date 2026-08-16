# Database & Data Files Integration Guide

## Overview

This document provides detailed information about the data files used with the eTamil compiler examples, including schemas, relationships, and integration patterns.

---

## Data File Inventory

### CSV Files (5 files)

| File | Purpose | Records | Columns | Format |
|------|---------|---------|---------|--------|
| `students.csv` | Student records with academic scores | 8 | 8 columns | Standard CSV |
| `payroll.csv` | Employee salary information | 8 | 5 columns | Standard CSV |
| `inventory.csv` | Product inventory tracking | 10 | 7 columns | Standard CSV |
| `sales.csv` | Sales transaction records | 10 | 8 columns | Standard CSV |
| `products.csv` | Product database | 10 | 3 columns | Standard CSV |

### JSON Files (2 files)

| File | Purpose | Documents | Type | Format |
|------|---------|-----------|------|--------|
| `students.json` | Student data (NoSQL) | 4 | MongoDB | JSON Array |
| `products.json` | Product catalog (NoSQL) | 4 | MongoDB | JSON Array |

---

## Detailed Schema Definitions

### students.csv Schema

**Purpose:** Store student academic records with multi-subject scores

**Columns:**
```
Column 1: student_id    (String)  - Unique student identifier (S1001-S1008)
Column 2: name          (String)  - Student full name in Tamil
Column 3: class         (Number)  - Class level (9-10)
Column 4: tamil_score   (Number)  - Tamil subject score (0-100)
Column 5: english_score (Number)  - English subject score (0-100)
Column 6: math_score    (Number)  - Mathematics score (0-100)
Column 7: science_score (Number)  - Science subject score (0-100)
Column 8: gpa           (Decimal) - Grade Point Average (0.0-4.0)
```

**Sample Record:**
```csv
S1001,ராஜ் குமாரசாமி,10,92,88,95,91,3.8
```

**Data Characteristics:**
- 8 student records
- GPA range: 3.4 to 3.8
- Score average: 85-95 range
- All Tamil names with proper Unicode

**Use Cases:**
- Academic performance tracking
- Grade calculation
- GPA aggregation
- Class-wise statistics

---

### payroll.csv Schema

**Purpose:** Maintain employee master data and salary information

**Columns:**
```
Column 1: emp_id        (String)  - Employee ID (E1001-E1008)
Column 2: name          (String)  - Employee full name in Tamil
Column 3: department    (String)  - Department name
Column 4: salary        (Number)  - Monthly salary in INR
Column 5: join_date     (Date)    - Joining date (YYYY-MM-DD)
```

**Sample Record:**
```csv
E1001,ராஜா குமாரசாமி,Engineering,50000,2021-01-15
```

**Department Distribution:**
- Engineering: 2 employees
- Sales: 1 employee
- HR: 1 employee
- Finance: 1 employee
- IT: 1 employee
- Operations: 1 employee
- Marketing: 1 employee

**Salary Range:**
- Minimum: ₹35,000 (HR)
- Maximum: ₹52,000 (IT)
- Average: ₹42,375

**Use Cases:**
- Payroll processing
- Tax calculation
- Deduction management
- HR reporting

---

### inventory.csv Schema

**Purpose:** Track product inventory with stock and pricing information

**Columns:**
```
Column 1: product_id      (String)  - Unique product code (P2001-P2010)
Column 2: product_name    (String)  - Product description
Column 3: category        (String)  - Product category
Column 4: stock_quantity  (Number)  - Current stock in units
Column 5: price_per_unit  (Number)  - Unit price in INR
Column 6: reorder_level   (Number)  - Minimum stock threshold
Column 7: supplier_id     (String)  - Supplier identifier
```

**Sample Record:**
```csv
P2001,Laptop Dell XPS,Electronics,25,40000,5,SUP001
```

**Product Categories:**
- Electronics: 6 products
- Furniture: 2 products
- Stationery: 2 products

**Stock Analysis:**
- Total inventory value: ~₹15,00,000
- Low stock items: Dell Laptop (25), Monitor (45), Standing Desk (35)
- High stock items: USB Cable (300), Notebook (500), Pen Set (200)

**Use Cases:**
- Inventory management
- Stock level alerts
- Reorder notifications
- Warehouse tracking

---

### sales.csv Schema

**Purpose:** Record all sales transactions with financial details

**Columns:**
```
Column 1: sale_id           (String)  - Transaction ID (S5001-S5010)
Column 2: product_id        (String)  - Product reference (links to inventory.csv)
Column 3: product_name      (String)  - Product description (denormalized)
Column 4: quantity_sold     (Number)  - Units sold
Column 5: unit_price        (Number)  - Price per unit at time of sale
Column 6: total_amount      (Number)  - Total revenue (quantity × price)
Column 7: sale_date         (Date)    - Transaction date (YYYY-MM-DD)
Column 8: salesperson_id    (String)  - Sales representative ID
```

**Sample Record:**
```csv
S5001,P2001,Laptop Dell XPS,2,40000,80000,2024-01-15,SA001
```

**Sales Analysis (Jan 15-19, 2024):**
- Total transactions: 10
- Total revenue: ₹2,92,000
- Products sold: 65 units
- Top selling product: Keyboard Mechanical (8 units)
- Top revenue product: Laptop Dell XPS (₹80,000)

**Salesperson Performance:**
- SA001: 4 transactions, ₹1,58,000
- SA002: 3 transactions, ₹23,000
- SA003: 2 transactions, ₹1,00,000
- SA004: 1 transaction, ₹26,500

**Use Cases:**
- Sales reporting
- Revenue tracking
- Salesperson performance
- Product demand analysis

---

### students.json Schema

**Purpose:** NoSQL document storage for student records with embedded data

**Document Structure:**
```json
{
  "_id": "STU001",                          // Document ID
  "name": "ராஜ் குமாரசாமி",                  // Full name
  "email": "raj.kumar@school.edu",          // Email address
  "phone": "+91-9876543210",                // Contact number
  "class": "10",                            // Class level
  "address": {
    "street": "123 முக்குந்த் ஸ்ட்ரீட்",     // Street address
    "city": "சென்னை",                       // City
    "state": "தமிழ்நாடு",                    // State
    "zip": "600001"                          // Postal code
  },
  "enrollment_date": "2023-06-15",          // ISO date format
  "guardian": {
    "name": "குமாரசாமி வெங்கடாசலபதி",     // Guardian name
    "relation": "Father",                   // Relationship
    "phone": "+91-9876543211"               // Guardian contact
  },
  "academic_status": "Active"               // Status flag
}
```

**Document Count:** 4 records (STU001-STU004)

**Field Types:**
- String: name, email, phone, class, city
- Object/Nested: address (object), guardian (object)
- Date: enrollment_date (ISO 8601 format)
- Boolean/Enum: academic_status

**Use Cases:**
- Student profile management
- Contact information storage
- Guardian record keeping
- NoSQL database examples

---

### products.json Schema

**Purpose:** NoSQL document storage for product catalog with detailed specifications

**Document Structure:**
```json
{
  "_id": "PROD001",                    // Product ID
  "name": "Dell XPS 13",               // Product name
  "category": "Laptops",               // Category
  "specifications": {
    "processor": "Intel Core i7",      // CPU model
    "ram": "16GB",                     // Memory
    "storage": "512GB SSD",            // Storage capacity
    "display": "13.4 inch FHD"         // Screen specs
  },
  "price": 85000,                      // Price in INR
  "stock": {
    "total": 25,                       // Total stock
    "warehouse_a": 10,                 // Location A qty
    "warehouse_b": 15                  // Location B qty
  },
  "rating": 4.7,                       // Customer rating
  "reviews_count": 145,                // Number of reviews
  "last_updated": "2024-01-19T10:30:00Z"  // Timestamp
}
```

**Document Count:** 4 records (PROD001-PROD004)

**Field Types:**
- String: name, category, processor, color
- Number: price, rating, stock quantities
- Nested Object: specifications, stock
- DateTime: last_updated (ISO 8601)
- Numeric: reviews_count, rating

**Categories:**
- Laptops: 1 product
- Peripherals: 1 product
- Furniture: 1 product
- Accessories: 1 product

**Use Cases:**
- Product catalog storage
- Specification management
- Stock tracking across warehouses
- Review and rating system

---

## Data Relationships & Joins

### Relational Model (CSV Files)

```
students.csv
    ↓ (academic data)
    └→ Aggregates to payroll.csv (employees are related)

payroll.csv
    ↓ (salary data)
    └→ Calculates deductions for tax year

inventory.csv
    ↓ (product master)
    ↑ (referenced by)
    
sales.csv
    ↓ (transaction detail)
    └→ Links to inventory.csv via product_id
```

**Join Example:**
```sql
-- Sales analysis by product
SELECT 
  s.product_id,
  i.product_name,
  SUM(s.quantity_sold) as total_qty,
  SUM(s.total_amount) as total_revenue
FROM sales.csv s
JOIN inventory.csv i ON s.product_id = i.product_id
GROUP BY s.product_id
ORDER BY total_revenue DESC;
```

### NoSQL Model (JSON Files)

```
students.json
    └→ Embeds: address, guardian (nested objects)
    
products.json
    └→ Embeds: specifications, stock (nested objects)
```

**Query Example:**
```javascript
// Find students in Chennai with Active status
db.students.find({
  "address.city": "சென்னை",
  "academic_status": "Active"
})

// Find laptop products with price > 80000
db.products.find({
  "category": "Laptops",
  "price": {$gt: 80000}
})
```

---

## Data Usage in eTamil Examples

### simple_fileio.qmz
- Input: `input.txt` (basic text file)
- Processing: Read and write operations
- Output: Text-based output

### fileio_example.qmz
- Input: `products.csv`, `numbers.txt`
- Processing: CSV parsing
- Output: Processed CSV

### student_management.qmz
- Input: `students.csv` (academic records)
- Processing: Score aggregation, GPA calculation
- Output: Summary statistics
- SQL operations: SELECT, SUM, AVG

### inventory_system.qmz
- Input: `inventory.csv`, `sales.csv`
- Processing: Sales recording, revenue calculation
- Output: Inventory report, sales summary
- SQL operations: INSERT, SELECT, UPDATE

### payroll_system.qmz
- Input: `payroll.csv` (employee data)
- Processing: Tax calculation (10%), deduction (variable), net salary
- Output: `payroll_summary.csv` (processed payroll)
- Calculation: `net = gross - tax - deduction`

### crypto_demo.rs
- Input: Sample text and CSV files
- Processing: XOR cipher encryption
- Output: `.ani` (encrypted text), `.qrv` (encrypted CSV)

### db_commands_demo.rs
- Input: `students.json`, `products.json` (NoSQL demos)
- Processing: Document insertion, querying, aggregation
- Output: Query results, collection statistics

---

## Data Flow Diagrams

### Student Record Processing
```
students.csv (raw data)
    ↓
[eTamil Parser]
    ↓
Variable Assignment (scores in memory)
    ↓
Calculation (total_score = s1 + s2 + s3)
    ↓
Output (Print total)
    ↓
[Optional: Encrypt] → students.qrv (encrypted CSV)
```

### Payroll Processing Pipeline
```
payroll.csv (employee master)
    ↓
[Read employee record]
    ↓
gross_salary ← read from CSV
tax_amount ← (gross_salary × 0.10)
deduction ← read/calculate
net_salary ← (gross_salary - tax_amount - deduction)
    ↓
[Write to payroll_summary.csv]
    ↓
Output (payroll_summary.csv)
    ↓
[Optional: Encrypt] → payroll_summary.qrv
```

### Inventory & Sales Integration
```
inventory.csv (product master)
    ↓
    ├→ Stock tracking
    │   └→ Alert if < reorder_level
    │
sales.csv (transactions)
    ↓
    ├→ Update inventory.stock_quantity
    │   └→ quantity_sold deducted
    │
    ├→ Calculate revenue
    │   └→ SUM(total_amount by date)
    │
    └→ Sales report
        └→ Revenue by product/salesperson
```

---

## Data Import/Export Specifications

### Supported Formats

| Format | Extension | Use Case | Notes |
|--------|-----------|----------|-------|
| CSV | `.csv` | Relational data | Standard RFC 4180 |
| JSON | `.json` | Document data | Compact UTF-8 |
| Text | `.txt` | Plain text | UTF-8 encoded |
| Encrypted Text | `.ani` | Secure text | XOR cipher |
| Encrypted CSV | `.qrv` | Secure tabular | XOR cipher |

### Import Procedures

**CSV Import:**
```tamil
தரவுरை_एपुतु "inventory.csv" "product_id,name,price";
```

**JSON Import (NoSQL):**
```rust
let products = serde_json::from_str::<Vec<Product>>(json_str)?;
db.insert_document("products", id, products);
```

**Encrypted File Import:**
```rust
let decrypted = CryptoHandler::read_encrypted_txt("file.ani", "key");
```

### Export Procedures

**CSV Export:**
```tamil
तरवुरै_एपुतु "output.csv" "col1,col2,col3";
```

**JSON Export:**
```rust
let json = serde_json::to_string(&document)?;
fs::write("export.json", json)?;
```

**Encrypted File Export:**
```rust
CryptoHandler::write_encrypted_txt("output.ani", content, "key")?;
```

---

## Performance Characteristics

### CSV Files
- Average row size: 80-150 bytes
- Total size (all CSV): ~45 KB
- Read time (full file): < 1ms
- Write time (single row): < 0.1ms

### JSON Files
- Average document size: 400-600 bytes
- Total size (all JSON): ~4 KB
- Parse time (single doc): < 0.5ms
- Serialize time (doc): < 0.2ms

### Encryption Performance
- Encryption speed: ~10 MB/s (XOR)
- File overhead: +33% (base64 encoding)
- Decryption speed: ~10 MB/s (symmetric)

---

## Data Quality Standards

### Completeness
- All required fields populated
- No NULL values in key columns
- Tamil names properly encoded (UTF-8)

### Consistency
- Student IDs follow pattern: S[0-9]{4}
- Employee IDs follow pattern: E[0-9]{4}
- Product IDs follow pattern: P[0-9]{4}
- Sale IDs follow pattern: S[0-9]{4}
- Dates in ISO 8601 format (YYYY-MM-DD)

### Accuracy
- Salary values within realistic range (₹30K-₹60K)
- Test scores in range 0-100
- GPA in range 0.0-4.0
- Stock quantities > 0

---

## Backup & Recovery

### Backup Strategy
```bash
# Backup all CSV files
tar -czf data_backup.tar.gz examples/*.csv

# Backup all JSON files
tar -czf documents_backup.tar.gz examples/*.json

# Backup encrypted files
tar -czf encrypted_backup.tar.gz examples/*.{ani,qrv}
```

### Restoration
```bash
# Restore CSV data
tar -xzf data_backup.tar.gz -C examples/

# Verify integrity
sha256sum examples/*.csv > checksums.txt
sha256sum -c checksums.txt
```

---

## Future Data Enhancements

### Planned Additions
- Time series data (daily/monthly reports)
- Hierarchical data (department → teams → employees)
- Graph data (relationships between students)
- Unstructured data (student feedback, comments)

### Scalability Considerations
- Current: 40 total records
- Medium scale: 1,000 records
- Large scale: 10,000+ records
- Handle accordingly for query optimization

---

**Last Updated:** January 2024  
**Data Version:** 1.0  
**Total Records:** 40 (CSV + JSON combined)  
**Encoding:** UTF-8 (Tamil text support)
