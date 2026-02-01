# eTamil Tamil Letter Equivalents Guide

**Reference**: Based on `ezuqqu.pdf` Tamil letter romanization system  
**Version**: 2.0 (Corrected - eTamil Standard Only)  
**Updated**: February 1, 2026

---

## 📖 Introduction

This document provides the Tamil letter equivalents and their **eTamil-specific** romanized (Latin character) representations used throughout the eTamil programming language. This helps developers understand how Tamil keywords are converted to their phonetic eTamil equivalents for use in code and documentation.

**IMPORTANT**: This guide uses ONLY the eTamil standard from `ezuqqu.pdf`, NOT standard transliteration (ISO 15919). The two systems are different.

---

## 🔤 Tamil Letter System (eTamil Standard)

### Vowels (உயிர் எழுத்து)

| Tamil Letter | eTamil Romanized | Example Tamil | eTamil Example | Meaning |
|--------------|------------------|----------------|-----------------|---------|
| அ | a | அச்சு | accu | print |
| ஆ | A | ஆ | A | (ah) |
| இ | i | இருப்பு | iruppu | balance |
| ஈ | I | நீ | nI | you |
| உ | u | உள்ளிடு | uLLitu | input |
| ஊ | U | கூ | kU | (coo) |
| எ | e | எண் | eN | number |
| ஏ | E | தேதி | tEti | date |
| ஐ | Y | வைப்பு | vYppu | storage |
| ஒ | o | கோப்பு | kOppu | file |
| ஓ | O | தோ | tO | (till) |
| ஔ | V | கௌ | kV | (caste) |
| ஃ | h | - | - | (archaic) |

### Consonants (மெய் எழுத்து) - With Gemination Marker ்

| Tamil Letter | eTamil Romanized | Example Tamil | eTamil Example | Meaning |
|--------------|------------------|----------------|-----------------|---------|
| க் | k | கோப்பு | kOppu | file |
| ங் | w | சங்கு | cawku | conch |
| ச· | c | சுற்று | cuRRu | loop |
| ஞ் | W | ஞாணம் | WANam | cord |
| ட் | t | ஒட்டு | ottu | attach |
| ண் | N | கணம் | kaNam | moment |
| த் | q | தளம் | qaLam | platform |
| ந· | N | நிலை | nilY | status |
| ப் | p | பணம் | paNam | money |
| ம் | m | மாறி | mARi | variable |
| ய் | y | யாழ் | yAL | harp |
| ர் | r | வரி | vari | tax |
| ல் | l | நிலை | nilY | status |
| வ் | v | வரவு | varavu | credit |
| ழ் | z | பழம் | pazam | fruit |
| ள் | L | சூள் | sUL | list |
| ற· | R | கற் | kaR | stone |
| ன् | n | சொன் | con | said |

### Additional Consonants

| Tamil Letter | eTamil Romanized | Example Tamil | eTamil Example | Meaning |
|--------------|------------------|----------------|-----------------|---------|
| ஹ | H | - | - | (h sound) |
| ஜ | j | - | - | (j sound) |
| ஷ | S | - | - | (sh sound) |
| ஸ | s | - | - | (s sound) |
| க்ஷ | x | - | - | (ksha sound) |

### Gemination Rules (Double Consonants)

When consonants are doubled (marked with ்), use the eTamil romanized form appropriately:

| Tamil Cluster | eTamil Romanized | Example Tamil | eTamil Example |
|---------------|------------------|----------------|-----------------|
| க்க | kk | செக்கு | cekku |
| ங்ங | ww | சாங்ங | sawwu |
| ச்ச | cc | அச்சு | accu |
| ஞ்ஞ | WW | - | - |
| ட்ட | tt | உட்டு | uttu |
| ண்ண | NN | - | - |
| த்த | qq | மொத்த | moqqa |
| ந்ந | NN | - | - |
| ப்ப | pp | இருப்பு | iruppu |
| ம்ம | mm | - | - |
| ய்ய | yy | - | - |
| ர்ர | rr | - | - |
| ல்ல | LL | உள்ளிடு | uLLitu |
| வ்வ | vv | - | - |
| ழ்ழ | zz | - | - |
| ள்ள | LL | சூள் | sUL |
| ற்ற | RR | பற்று | paRRu |
| ன்ன | nn | - | - |

---

## 🗂️ eTamil Keyword Categories & Letter Equivalents

### Control Flow Keywords

| Tamil | eTamil Romanized | Letter Breakdown | Meaning | Usage |
|-------|------------------|------------------|---------|-------|
| எனில் | enil | e+n+i+l | If | `(condition) எனில் { ... }` |
| இன்றேல் | inREl | i+n+R+E+l | Else | `} இன்றேல் { ... }` |
| சுற்று | cuRRu | c+u+RR+u | Loop | `சுற்று i=1; i<=10; { ... }` |
| அச்சு | accu | a+c+c+u | Print | `அச்சு "text";` |
| உள்ளிடு | uLLitu | u+LL+i+t+u | Input | `உள்ளிடு variable;` |

### Data Type Keywords

| Tamil | eTamil Romanized | Letter Breakdown | Meaning | Usage |
|-------|------------------|------------------|---------|-------|
| எண் | eN | e+N | Integer/Number | `எண் age = 25;` |
| பின்னம் | pinnam | p+i+nn+a+m | Float | `பின்னம் price = 99.99;` |
| சொல் | col | c+o+l | String | `சொல் text = "hello";` |
| ஈர்ம | Irm | I+r+m | Boolean | `ஈர்ம flag = mey;` |
| உரை | urY | u+r+Y | Text | `உரை content;` |
| அணி | aNi | a+N+i | Array | `அணி items;` |
| தேதி | tEti | t+E+t+i | Date | `தேதி today;` |

### File Operation Keywords

| Tamil | eTamil Romanized | Letter Breakdown | Meaning | Usage |
|-------|------------------|------------------|---------|-------|
| கோப்பு_திற | kOppu_tiRa | k+O+pp+u / t+i+Ra | File Open | `கோப்பு_திற "file.txt";` |
| கோப்பு_மூடு | kOppu_mUtu | k+O+pp+u / m+U+t+u | File Close | `கோப்பு_மூடு "file.txt";` |
| கோப்பு_பாتி | kOppu_pati | k+O+pp+u / p+a+t+i | File Read | `கோப்பு_பாதி "file.txt", var;` |
| கோப்பு_எழுतु | kOppu_ezutu | k+O+pp+u / e+z+u+t+u | File Write | `கோப்பு_எழுตு "file.txt", data;` |
| தरवुरै_पাതி | taruvurY_pati | t+a+r+u+v+u+r+Y / p+a+t+i | Read CSV | `தरवुरै_पाति "data.csv", var;` |
| தरवुरै_एजुतु | taruvurY_ezutu | t+a+r+u+v+u+r+Y / e+z+u+t+u | Write CSV | `தरवुरै_एजुตு "data.csv", row;` |

### Database Operation Keywords

| Tamil | eTamil Romanized | Letter Breakdown | Meaning | Usage |
|-------|------------------|------------------|---------|-------|
| தளம்_இணை | taLam_iNY | t+a+L+a+m / i+NN+Y | DB Connect | `தளம்_இணை "sqlite", "db.db";` |
| தளம்_பிরி | taLam_piri | t+a+L+a+m / p+i+r+i | DB Disconnect | `தளம்_பிரி "sqlite";` |
| தளம்_விना | taLam_vinA | t+a+L+a+m / v+i+n+A | DB Query | `தளம்_விना "SELECT ...";` |
| தளம்_செய் | taLam_cey | t+a+L+a+m / c+e+y | DB Execute | `தளம்_செய் "CREATE TABLE ...";` |
| தளம்_செருக | taLam_ceruka | t+a+L+a+m / c+e+r+u+k+a | DB Insert | `தளம்_செருக users, data;` |
| தளம்_புசுपि | taLam_puqupi | t+a+L+a+m / p+u+qu+p+i | DB Update | `தளம்_புசுपि users, data;` |
| தளம்_நீக்கு | taLam_nIkku | t+a+L+a+m / n+I+kk+u | DB Delete | `தளம்_நீக்கு users, condition;` |

### Boolean & Null Values

| Tamil | eTamil Romanized | Letter Breakdown | Meaning | Usage |
|-------|------------------|------------------|---------|-------|
| மேய் | mey | m+e+y | True | `flag = mey;` |
| பொய் | poy | p+o+y | False | `flag = poy;` |
| இன்மய் | inmY | i+n+m+Y | Null | `value = inmY;` |

---

## 📋 eTamil Romanization Rules

### 1. **Vowel Representations (eTamil Standard)**
- Short vowels use lowercase: a, e, i, o, u
- Long vowels use uppercase: A, E, I, O, U
- Diphthong ai: Y
- Diphthong au: V
- Examples:
  - அ = a
  - ஆ = A
  - ஐ = Y (NOT standard 'ai')
  - ஔ = V (NOT standard 'au')

### 2. **Consonant Representations (eTamil Standard - Key Differences)**
| Consonant | eTamil | Standard | Note |
|-----------|--------|----------|------|
| ங் | w | ng | eTamil uses 'w' |
| ஞ் | W | nj | eTamil uses 'W' |
| த் | q | th | eTamil uses 'q' |
| வ் | v | v | Same as standard |
| ழ் | z | zh | eTamil uses 'z' |
| ற் | R | r/tr | eTamil uses 'R' |
| ள் | L | l | eTamil uses 'L' |
| ண் | N | n | eTamil uses 'N' |
| ந் | N | n | eTamil uses 'N' |

### 3. **Consonant Clusters (Gemination)**
- Double consonants: use eTamil form + eTamil form
- ற்ற = RR (example: பற்று = paRRu)
- ப்ப = pp (example: இருப்பு = iruppu)
- க்க = kk (example: செக்கு = cekku)
- ல்ல = LL (example: உள்ளிடு = uLLitu)
- ங்ங = ww (example: சாங்ங = sawwu)
- த்த = qq (example: மொத்த = moqqa)

### 4. **Combined Forms & Conjuncts**
When consonants combine:
- த + ர = qr
- ப + ற = pR  
- ஞ் + ச = Wc

---

## 💡 Usage Examples

### Example 1: Simple Variable Declaration
```etamil
// Tamil form
எண் வருவாய் = 50000;

// eTamil Romanized breakdown:
// எ(e) + ண்(N) = eN
// வ(v) + ர(r) + ு(u) + வ(v) + ா(A) + ய்(y) = varuvAy
```

### Example 2: File Operation
```etamil
// Tamil form
கோப்பு_திற "data.txt";

// eTamil Romanized breakdown:
// கோ(kO) + ப்ப(pp) + ு(u) = kOppu (File)
// தி(ti) + ற(R) + ா(a) = tiRa (Open)
// Combined: kOppu_tiRa
```

### Example 3: Financial Calculation
```etamil
வருவாய் = 100000;
செலவு = 30000;
பயன் = வருவாய் - செலவு;

// eTamil Romanized:
// varuvAy (Revenue) = 100000
// celavu (Expense) = 30000
// payan (Profit) = Revenue - Expense
```

---

## 🔗 References

- **Source**: `ezuqqu.pdf` - Tamil Letter Equivalents Guide (eTamil Standard ONLY)
- **eTamil Lexer**: `src/lexer.rs` in compiler source

---

## 📝 Key Notes

1. **Standard Used**: eTamil-specific romanization from `ezuqqu.pdf` ONLY
2. **NOT Standard Transliteration**: ISO 15919 is different and should NOT be used
3. **Consistency**: All romanized forms must use eTamil standard exclusively
4. **Case Sensitivity**: Romanized keywords are case-sensitive in code
5. **Mixed Usage**: You can use either Tamil or romanized form interchangeably

---

**Last Updated**: February 1, 2026  
**Version**: 2.0 (Corrected to eTamil Standard Only)  
**Maintained by**: eTamil Development Team  
**Reference**: ezuqqu.pdf Tamil Letter Equivalents (eTamil Standard)
