# eTamil Compiler - Actual Keywords Reference

**Verified**: January 31, 2026 | Source: `src/lexer.rs` and `src/parser.rs`

---

## ⚠️ IMPORTANT: Actual vs. False Keywords

### What DOES NOT Work ❌
```etamil
print "Hello";          // WRONG - use அச்சு instead
input x;                // WRONG - use உள்ளிடு instead
if (x > 5) { }          // WRONG - use எனில் instead
loop { }                // WRONG - use சுற்று instead
fun myFunction() { }    // NO FUNCTION KEYWORD - not supported
file_open("test.txt");  // WRONG - use கோப்பு_திற instead
```

### What DOES Work ✅
```etamil
அச்சு "Hello";          // Correct - Print
உள்ளிடு x;              // Correct - Input
(x > 5) எனில् { }       // Correct - If
சுற்று i = 1; i <= 10; { }  // Correct - Loop
கோப்பு_திற "test.txt";  // Correct - File open
```

---

## Complete Keyword List

### Control Flow (5 keywords)
| Tamil | English | Token | Usage |
|-------|---------|-------|-------|
| `எனில்` | `enil` | If | `(condition) எனில् { ... }` |
| `இன்றேல்` | `inREl` | Else | `} இன்றேல् { ... }` |
| `சுற்று` | `cuRRu` | Loop | `சுற்று i = 1; i <= 10; { ... }` |
| `அச்சு` | `accu` | Print | `அச்சு "text";` |
| `உள்ளிடு` | `uLLitu` | Input | `உள்ளிடு variable;` |

### Data Types (7 keywords)
| Tamil | English | Token | Usage |
|-------|---------|-------|-------|
| `எண்` | `eN` | IntegerType | `எண் age = 25;` |
| `பின்னம்` | `pinnam` | FloatType | `பின்னம் price = 99.99;` |
| `சொல்` | `col` | StringType | `சொல் name = "text";` |
| `பொது` | `poqu` | BoolType | `பொது flag = True;` |
| `உரை` | `urY` | TextType | `உரை text;` |
| `அணி` | `aNi` | ArrayType | `அணி arr;` |
| `தேதி` | `qEqi` | DateType | `தேதி today;` |

### Boolean Values (3 keywords)
| Tamil | English | Token | Usage |
|-------|---------|-------|-------|
| `மெய்` | `mey` | True | `பொது x = மெய்;` |
| `பொய்` | `poi` | False | `பொது x = பொய்;` |
| `இன்மை` | `inmY` | Null | `சொல் x = இன்மை;` |

### Variable Declaration (2 keywords)
| Tamil | English | Token | Usage |
|-------|---------|-------|-------|
| `மாறி` | `mARi` | Let | `மாறி x = 10;` |
| `நிலை` | `nilY` | Const | `நிலை PI = 3.14;` |

### File Operations (8 keywords)
| Tamil | English | Token | Usage |
|-------|---------|-------|-------|
| `கோப்பு_திற` | `kOppu_qiRa` | FileOpen | `கோப்பு_திற "file.txt";` |
| `கோப்பு_மூடு` | `kOppu_mUtu` | FileClose | `கோப்பு_மூடு "file.txt";` |
| `கோப்பு_படி` | `kOppu_pati` | FileRead | `கோப்பு_படி "file.txt", var;` |
| `கோப்பு_எழுது` | `kOppu_ezuqu` | FileWrite | `கோப்பு_எழுது "file.txt", "data";` |
| `கோப்பு_நிரை` | `kOppu_NirY` | FileLines | `கோப்பு_நிரை "file.txt";` |
| `தரவுரை_படி` | `qaravurY_pati` | ReadCSV | `தரவுரை_படி "data.csv", var;` |
| `தரவுரை_எழுது` | `qaravurY_ezuqu` | WriteCSV | `தரவுரை_எழுது "data.csv", "row";` |
| `கோப்பு` | `kOppu` | File | Base keyword for file operations |

### Database Connectivity (7 keywords)
| Tamil | English | Token | Usage |
|-------|---------|-------|-------|
| `தளம்_இணை` | `qaLam_iNY` | DBConnect | `தளம்_இணை "sqlite", "db.db";` |
| `தளம்_பிரி` | `qaLam_piri` | DBDisconnect | `தளம்_பிரி "sqlite";` |
| `தளம்_வினா` | `qaLam_vinA` | DBQuery | `தளம்_வினா "SELECT * FROM users";` |
| `தளம்_செய்` | `qaLam_cey` | DBExecute | `தளம்_செய் "CREATE TABLE ...";` |
| `தளம்_தேடு` | `qaLam_qEtu` | DBSearch | `தளம்_தேடு "WHERE id = 1";` |
| `தளம்_செருக` | `qaLam_ceruka` | DBInsert | `தளம்_செருக "users", data;` |
| `தளம்_புதுப்பி` | `qaLam_puquppi` | DBUpdate | `தளம்_புதுப்பி "users", data, condition;` |
| `தளம்_நீக்கு` | `qaLam_nIkku` | DBDelete | `தளம்_நீக்கு "users", condition;` |

### Database Structure (8 keywords)
| Tamil | English | Token | Usage |
|-------|---------|-------|-------|
| `அட்டை` | `attY` | Table | Database table |
| `தொகுப்பு` | `toguippu` | Collection | MongoDB collection |
| `நிரை` | `NirY` | Row | Table row |
| `பத்தி` | `paqqi` | Column | Table column |
| `விசை` | `vicY` | Key | Database key |
| `தனிக_விசை` | `tanik_vicY` | PrimaryKey | Primary key |
| `வெளி_விசை` | `veLi_vicY` | ForeignKey | Foreign key |
| `குறியீடு` | `kuRiyItu` | Index | Database index |

### Database Types (8 keywords)
| Tamil | English | Token | Usage |
|-------|---------|-------|-------|
| `கவி_மொழி` | `kavi_mozi` | SQL | SQL databases |
| `தேடு_மொழி` | `qEtu_mozi` | NoSQL | NoSQL databases |
| `சீகுலைட்` | `cIkulYt` | SQLite | SQLite database |
| `மைசீகுல்` | `mYcIkul` | MySQL | MySQL database |
| `போச்குரசீகுல்` | `pOckuracIkul` | PostgreSQL | PostgreSQL database |
| `மாங்கோடிபி` | `mAwkOṭipi` | MongoDB | MongoDB database |
| `ரெடிஸ்` | `retis` | Redis | Redis cache |
| `ஜேசான்` | `jEcAn` | JSONdb | JSON database |

### SQL Keywords (12 keywords)
| Tamil | English | Token | Usage |
|-------|---------|-------|-------|
| `தேர்வெடு` | `qErvetu` | Select | SELECT clause |
| `இதனில்` | `iqanil` | From | FROM clause |
| `விதி` | `viqi` | Where | WHERE clause |
| `வரிசை` | `varicY` | OrderBy | ORDER BY clause |
| `குழு` | `kuzu` | GroupBy | GROUP BY clause |
| `சேர்` | `cEr` | Join | JOIN clause |
| `இடம்` | `itam` | Left | LEFT join |
| `வலம்` | `valam` | Right | RIGHT join |
| `உள்` | `uL` | Inner | INNER join |
| `வெளி` | `veLi` | Outer | OUTER join |
| `தனிக` | `qanika` | Distinct | DISTINCT keyword |
| `வரம்பு` | `varampu` | Limit | LIMIT clause |

### Accounting/Finance Keywords (40+ keywords) ✅ NEW
*These exist in the lexer for financial applications*

| Tamil | English | Token | Usage |
|-------|---------|-------|-------|
| `வரவு` | `varavu` | Credit | வரவு 5000; |
| `பற்று` | `paRRu` | Debit | பற்று 3000; |
| `இருப்பு` | `iruppu` | Balance | இருப்பு = 10000; |
| `வீதம்` | `vIqam` | Rate | வீதம் 5.5; |
| `சொத்து` | `soqqu` | Asset | சொத்து = 50000; |
| `பொறுப்பு` | `poRuppu` | Liability | பொறுப்பு = 25000; |
| `பங்கு` | `paNgu` | Equity | பங்கு = 25000; |
| `வருவாய்` | `varuvAy` | Revenue | வருவாய் = 100000; |
| `செலவு` | `celavu` | Expense | செலவு = 40000; |
| `வருமானம்` | `varumAnam` | Income | வருமானம் = 60000; |
| `பயன்` | `payan` | Profit | பயன் = 20000; |
| `இழப்பு` | `izappu` | Loss | இழப்பு = 5000; |
| `வரி` | `vari` | Tax | வரி = 6000; |
| `நிகர` | `Nikara` | Net | நிகர = 14000; |
| `மொத்த` | `moqqa` | Gross | மொத்த = 20000; |
| `வட்டி` | `vatti` | Interest | வட்டி = 2.5; |
| `பேரேடு` | `pErEtu` | Ledger | பேரேடு_நোக்कम்; |
| `பதிவு` | `paqivu` | Journal | பதிவு_நுக्रिष; |
| `கடன்` | `katan` | Loan | கடன் = 50000; |
| `நிதி` | `niqi` | Finance | நிதி_விशिष; |
| `அறிக்கை` | `aRikkY` | Statement | அறிக்கை_रिपोर्ट; |
| `மதிப்பீடு` | `matippIDu` | Valuation | மதिप्पीடु = 75000; |
| `கடன்_அட்டை` | `katan_attY` | CreditCard | கடன்_அட्टை_नम्बर; |
| `பணம்` | `paNam` | Cash | பணம் = 5000; |
| `வங்கி` | `vawki` | Bank | வங்கி_खाता; |
| `பெறத்தக்க` | `peRaqqakka` | Receivable | பெறத्तक्क = 8000; |
| `கொடுக்காத` | `kotukkAqa` | Payable | கொடுக्काtat = 12000; |
| `விற்பனர்` | `viRpanar` | Vendor | விற்पनर्_सूची; |
| `வாங்குநர்` | `vAwkuNar` | Customer | வாங्குनर्_खाता; |
| `நிலையான` | `nilYyAna` | Fixed | நिलYyAna_सम्पत्ति; |
| `நடப்பு` | `natappu` | Current | नतप्पु_सम्पत्ति; |
| `நடப்பிலில்லா` | `natappilillA` | NonCurrent | नतप्पिलिल्ला_सम्पत्ति; |
| `முகவரி` | `mukavari` | Address | मुकवरि = "..."; |
| `தொகை` | `toqai` | Amount | तोqai = 25000; |
| `நாணயம்` | `nANayam` | Currency | नाणयम् = "INR"; |
| `பரிவர्த्தனை` | `parivarttanai` | Transaction | परिवर्त्तनाई_ID; |
| `தேய்மானம்` | `qEymAnam` | Depreciation | तेयमानम् = 5000; |
| `கடன்தீர्ப்பு` | `katan_qIrppu` | Amortization | कटन्तीर्प्पु = 3000; |
| `மதிப்புயர்வு` | `matippuyarvu` | Appreciation | मतिप्पुयर्वु = 2000; |
| `மூலதனம்` | `mUlatanam` | Capital | मूलतनम् = 100000; |
| `இருப்பாய்வு` | `iruppAyvu` | TrialBalance | इरुप्पाYवु_रिपोर्ट; |
| `இருப்புநिலை` | `iruppunilY` | BalanceSheet | इरुप्पुनिलY_डेटा; |
| `வருமான_அறிக்கை` | `varumAn_aRikkY` | IncomeStatement | वरुमान्_अरिक्कY_रिपोर्ट; |
| `பணப்புழக்கம्` | `paNappuzakkam` | CashFlow | पणप्पुझक्कम्_विश्लेषण; |
| `வருமான_வரி` | `varumAn_vari` | IncomeTax | वरुमान्_वरि = 6000; |
| `வணிகவரி` | `vaNikavari` | GST | वणिकवरि = 18; |
| `வரியறிக்கை` | `variyaRikkY` | ITR | वरियारिक्कY_फाइल; |

---

## Complete Keyword Count

**Total Keywords Now Supported**: 130+

- Control Flow: 5
- Data Types: 7
- Boolean Values: 3
- Variable Declaration: 2
- File I/O: 8
- Database Connectivity: 7
- Database Structure: 8
- Database Types: 8
- SQL Keywords: 12
- REST API Keywords: 30+
- Security Keywords: 4
- **Financial & Accounting Keywords: 40+ ✅ NEW**

---

## Correct Syntax Examples

### Example 1: Print and Input
```etamil
// Print output
அச்சு "Welcome to eTamil";

// Get input from user
எண் age;
அச்சு "Enter your age: ";
உள்ளிடு age;

// Print result
அச்சு "Your age is: ";
அச்சு age;
```

### Example 2: Conditional Logic
```etamil
எண் score = 85;

// Check if score is passing
(score >= 60) எனில् {
    அச்சு "Passed!";
}
இன்றேல் {
    அச்சு "Failed!";
}
```

### Example 3: Loop
```etamil
// Print numbers 1 to 5
சுற்று i = 1; i <= 5; i = i + 1; {
    அச்சு i;
}
```

### Example 4: File Operations
```etamil
// Write to file
கோப்பு_திற "output.txt";
கோப்பு_எழுது "output.txt", "Hello from eTamil!";
கோப்பு_மூடு "output.txt";

// Read from file
கோப்பு_திற "output.txt";
சொல் data;
கோப்பு_படி "output.txt", data;
அச்சு data;
கோப்பு_மூடு "output.txt";
```

### Example 5: Database Operations
```etamil
// Connect to database
தளம்_இணை "sqlite", "myapp.db";

// Execute query
தளம்_வினா "SELECT * FROM users";

// Disconnect
தளம்_பிரி "sqlite";
```

---

## Important Notes

⚠️ **These Keywords are NOT Supported**:
- `fun` / `முறை` - Function definition (no parameter)
- `print`, `input`, `if`, `loop`, `else` - English-only keywords (must use Tamil)
- `%` as percentage operator - Lexer has it but execution may not support it
- Package manager installations - Must build from source

✅ **These Keywords ARE Supported**:
- All 5 control flow keywords (Tamil only)
- All 7 data type keywords
- All file operation keywords
- All database keywords
- All CSV operation keywords

---

## How to Use This Reference

1. **Want to print?** Use `அச்சு` keyword
2. **Want to read input?** Use `உள்ளிடு` keyword
3. **Want to check a condition?** Use `(expr) எனில् { }` syntax
4. **Want to loop?** Use `சுற்று` keyword
5. **Want to open a file?** Use `கோப்பு_திற` keyword

---

## Running eTamil Programs

```bash
# Build the compiler first (requires Rust installed)
cd etamil_compiler
cargo build --release

# Run a program with the VM executor
./target/release/etamil_compiler --vm yourprogram.etamil

# With input piped in
echo "data" | ./target/release/etamil_compiler --vm yourprogram.etamil
```

---

**Last Updated**: January 31, 2026  
**Verified Against**: src/lexer.rs (lines 1-226) and src/parser.rs (lines 1-625)
