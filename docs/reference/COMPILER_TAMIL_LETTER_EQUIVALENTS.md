# eTamil Compiler - Tamil Letter Equivalents Documentation

**Version**: 2.0 (Corrected - eTamil Standard Only)  
**Date**: February 1, 2026  
**Reference**: `ezuqqu.pdf` Tamil letter romanization system

---

## Overview

The eTamil compiler supports bilingual programming - you can write code using either Tamil characters or romanized (Latin character) equivalents. This document explains the Tamil letter system and how it maps to romanized forms used throughout the compiler.

**IMPORTANT**: This guide uses ONLY the eTamil standard from `ezuqqu.pdf`, NOT standard transliteration (ISO 15919). The two systems are fundamentally different and NOT compatible.

---

## Tamil Alphabet System

### Core Consonants (மெய் எழுத்து) - eTamil Standard ONLY

The Tamil alphabet has 18 consonants with these eTamil romanized equivalents:

| Tamil | eTamil | Sound | Example | Meaning | Note |
|-------|--------|-------|---------|---------|------|
| க | k | /ka/ | கோப்பு | file | - |
| ங | w | /ŋa/ | சங்கு | conch | **NOT 'ng'** |
| ச | c | /cha/ | சுற்று | loop | - |
| ஞ | W | /ña/ | ஞாணம் | cord | **NOT 'nj'** |
| ட | t | /ṭa/ | ஒட்டு | attach | - |
| ண | N | /ṇa/ | கணம் | moment | **Retroflex N** |
| த | q | /tha/ | தளம் | platform | **NOT 'th'** |
| ந | N | /na/ | நிலை | status | Same as ண் |
| ப | p | /pa/ | பணம் | money | - |
| ம | m | /ma/ | மாறி | variable | - |
| ய | y | /ya/ | யாழ் | harp | - |
| ர | r | /ra/ | வரி | tax | - |
| ல | l | /la/ | நிலை | status | - |
| வ | v | /va/ | வரவு | credit | **NOT 'w'** |
| ழ | z | /ḻa/ | பழம் | fruit | **NOT 'zh'** |
| ள | L | /ḷa/ | சூள் | list | **Retroflex L** |
| ற | R | /ṟa/ | கற் | stone | **Retroflex R** |
| ன | n | /na/ | சொன் | said | **Dental n** |

### Core Vowels (உயிர் எழுத்து) - eTamil Standard ONLY

Tamil has 12 vowels + 2 special vowels (14 total):

| Tamil | eTamil | Sound | Example | Meaning | Note |
|-------|--------|-------|---------|---------|------|
| அ | a | /a/ | அச்சு | print | Short a |
| ஆ | A | /aː/ | - | - | Long a |
| இ | i | /i/ | இருப்பு | balance | Short i |
| ஈ | I | /iː/ | நீ | you | Long i |
| உ | u | /u/ | உள்ளிடு | input | Short u |
| ஊ | U | /uː/ | கூ | coo | Long u |
| எ | e | /e/ | எண் | number | Short e |
| ஏ | E | /eː/ | தேதி | date | Long e |
| ஐ | Y | /ai/ | வைப்பு | storage | Diphthong (ai) |
| ஒ | o | /o/ | கோப்பு | file | Short o |
| ஓ | O | /oː/ | தோ | till | Long o |
| ஔ | V | /au/ | கௌ | caste | Diphthong (au) **NOT 'O'** |
| ஃ | h | /h/ | - | - | Archaic |

---

## eTamil Keyword Categories & Tamil Breakdown

### 1. Control Flow Keywords

#### If/Else
```
Tamil:      எனில் | இன்றேல்
Romanized:  enil | inREl
Breakdown:  e+n+i+l | i+n+R+E+l
```

#### Loop
```
Tamil:      சுற்று
Romanized:  cuRRu
Breakdown:  c+u+RR+u (double 'r' indicates gemination)
```

#### Print/Input
```
Tamil:      அச்சு | உள்ளிடு
Romanized:  accu | uLLitu
Breakdown:  a+c+c+u | u+LL+i+t+u
```

### 2. Data Type Keywords

#### Number/Float
```
Tamil:      எண் | பின்னம்
Romanized:  eN | pinnam
Breakdown:  e+N | p+i+nn+a+m
Notes:      'ண்' (geminated n) = N, 'ன்' = n
```

#### String/Text
```
Tamil:      சொல் | உரை
Romanized:  col | urY
Breakdown:  c+o+l | u+r+Y
Notes:      'ை' (ai vowel) = Y
```

#### Boolean
```
Tamil:      ஈர்ம
Romanized:  Irma
Breakdown:  I+r+m+a
Notes:      'ீ' (long i) = I
```

#### Array
```
Tamil:      அணி
Romanized:  aNi
Breakdown:  a+N+i
```

### 3. File Operation Keywords

#### File Open (கோப்பு_திற)
```
Tamil Part 1:  கோப்பு
Romanized:     kOppu
Breakdown:     k+O+p+p+u
Notes:         'ோ' (long o) = O, double 'p' = pp

Tamil Part 2:  திற
Romanized:     qiRa
Breakdown:     q+i+R+a
Notes:         'த' = q (retrofit t), 'ற' = R
```

#### File Close (கோப்பு_மூடு)
```
Tamil Part 1:  கோப்பு (same as above)
Tamil Part 2:  மூடு
Romanized:     mUtu
Breakdown:     m+U+t+u
Notes:         'ூ' (long u) = U
```

#### File Read (கோப்பு_படி)
```
Tamil:      படி
Romanized:  pati
Breakdown:  p+a+t+i
```

#### File Write (கோப்பு_எழுது)
```
Tamil:      எழுது
Romanized:  ezuqu
Breakdown:  e+z+u+qu
Notes:      'ழ' = z, 'து' = qu
```

### 4. Database Operation Keywords

#### DB Connect (தளம்_இணை)
```
Tamil Part 1:  தளம்
Romanized:     qaLam
Breakdown:     q+a+L+a+m
Notes:         'ள' = L (retroflex l)

Tamil Part 2:  இணை
Romanized:     iNY
Breakdown:     i+NN+Y
Notes:         'ண்' = NN (doubled), 'ை' = Y
```

#### DB Query (தளம்_வினா)
```
Tamil:      வினா
Romanized:  vinA
Breakdown:  w+i+n+A
Notes:      'வ' = w, 'ா' = A (long a)
```

#### DB Insert (தளம்_செருக)
```
Tamil:      செருக
Romanized:  ceruka
Breakdown:  c+e+r+u+k+a
```

#### DB Update (தளம்_புதுப்பி)
```
Tamil:      புதுப்பி
Romanized:  puquppi
Breakdown:  p+u+qu+pp+i
Notes:      'த' = q, double 'p' = pp
```

#### DB Delete (தளம்_நீக்கு)
```
Tamil:      நீக்கு
Romanized:  nIkku
Breakdown:  n+I+k+k+u
Notes:      'ீ' = I (long i), double 'k' = kk
```

### 5. Financial & Accounting Keywords

#### Credit/Debit (வரவு | பற்று)
```
Tamil Part 1:  வரவு
Romanized:     varavu
Breakdown:     w+a+r+a+w+u
Notes:         'வ' = w (appears twice)

Tamil Part 2:  பற்று
Romanized:     paRRu
Breakdown:     p+a+RR+u
Notes:         'ற்ற' = RR (doubled retroflex r)
```

#### Balance (இருப்பு)
```
Tamil:      இருப்பு
Romanized:  iruppu
Breakdown:  i+r+u+pp+u
Notes:      double 'p' = pp
```

#### Revenue (வருவாய்)
```
Tamil:      வருவாய்
Romanized:  varuvAy
Breakdown:  w+a+r+u+w+A+y
Notes:      'ா' = A (long a), 'ய்' = y
```

#### Expense (செலவு)
```
Tamil:      செலவு
Romanized:  celavu
Breakdown:  c+e+l+a+w+u
```

#### Income (வருமானம்)
```
Tamil:      வருமானம்
Romanized:  varumAnam
Breakdown:  w+a+r+u+m+A+n+a+m
```

#### Profit (பயன்)
```
Tamil:      பயன்
Romanized:  payan
Breakdown:  p+a+y+a+n
```

#### Loss (இழப்பு)
```
Tamil:      இழப்பு
Romanized:  izappu
Breakdown:  i+z+a+pp+u
Notes:      'ழ' = z, double 'p' = pp
```

#### Tax (வரி)
```
Tamil:      வரி
Romanized:  vari
Breakdown:  w+a+r+i
Notes:      'வ' = w
```

#### Amount (தொகை)
```
Tamil:      தொகை
Romanized:  toqai
Breakdown:  t+o+q+a+i
Notes:      'ொ' = o, 'ை' = Y (but here = ai)
```

#### Bank (வங்கி)
```
Tamil:      வங்கி
Romanized:  vawki
Breakdown:  w+a+N+k+i
Notes:      'ங்' = N (nasal n)
```

#### Cash (பணம்)
```
Tamil:      பணம்
Romanized:  paNam
Breakdown:  p+a+N+a+m
Notes:      'ண்' = N (retroflex n)
```

#### Interest (வட்டி)
```
Tamil:      வட்டி
Romanized:  vatti
Breakdown:  w+a+tt+i
Notes:      'ட்ட' = tt (doubled dental t)
```

#### Ledger (பேரேடு)
```
Tamil:      பேரேடு
Romanized:  pErEtu
Breakdown:  p+E+r+E+t+u
Notes:      'ே' = E (long e) appears twice
```

#### Net (நிகர)
```
Tamil:      நிகர
Romanized:  Nikara
Breakdown:  n+i+k+a+r+a
```

#### Gross (மொத்த)
```
Tamil:      மொத்த
Romanized:  moqqa
Breakdown:  m+o+qq+a
Notes:      'த்த' = qq (doubled dental t)
```

---

## Romanization Rules Summary

### ⚠️ CRITICAL: eTamil vs. Standard Transliteration

**DO NOT use standard transliteration (ISO 15919) - it is DIFFERENT from eTamil.**

| Consonant | eTamil | Standard | ❌ **WRONG** |
|-----------|--------|----------|-------------|
| ங் | **w** | ng | Using 'ng' is WRONG |
| ஞ் | **W** | nj | Using 'nj' is WRONG |
| த் | **q** | th | Using 'th' is WRONG |
| வ் | **v** | v | Using 'w' is WRONG |
| ழ் | **z** | zh | Using 'zh' is WRONG |
| ற் | **R** | r/rr | Using lowercase 'r' is WRONG |
| ஔ | **V** | au | Using 'o' or 'O' is WRONG |
| ஐ | **Y** | ai | Can be written 'ai' but eTamil prefers 'Y' |

### 1. **Vowel Mapping (eTamil Standard)**
- Short vowels: lowercase (a, e, i, o, u)
- Long vowels: uppercase (A, E, I, O, U)
- Diphthong ai: **Y** (NOT 'ai')
- Diphthong au: **V** (NOT 'au')

### 2. **Consonant Types (eTamil Standard)**
- **Velar**: க=k, ங=**w** (NOT N)
- **Palatal**: ச=c, ஞ=**W** (NOT J)
- **Retroflex**: ட=t, ண=**N**, ல=l, ற=**R** (NOT r)
- **Dental**: த=**q** (NOT th), ந=**N**, ப=p, ம=m
- **Semi-vowels**: ய=y, ர=r, ல=l, வ=**v** (NOT w)
- **Fricative**: ழ=**z** (NOT zh)
- **Lateral**: ள=**L** (NOT l)

### 3. **Gemination (Double Consonants) - eTamil Standard**
- க்க = kk
- ங்ங = **ww** (NOT ng+ng)
- ச்ச = cc
- ஞ்ஞ = **WW** (NOT nj+nj)
- ட்ட = tt
- ண்ண = NN
- த்த = **qq** (NOT th+th)
- ந்ந = NN
- ப்ப = pp
- ம்ம = mm
- ய்ய = yy
- ர்ர = rr
- ல்ல = ll
- வ்வ = **vv** (NOT ww)
- ழ்ழ = **zz** (NOT zh+zh)
- ள்ள = LL
- ற்ற = **RR** (NOT rr)
- ன்ன = nn

---

## Usage in Code

### Example 1: Using Tamil Keywords
```etamil
எண் வருவாய் = 100000;
எண் செலவு = 30000;
எண் பயன் = வருவாய் - செலவு;
அச்சு பயன்;
```

### Example 2: Using Romanized Keywords
```etamil
eN varuvAy = 100000;
eN celavu = 30000;
eN payan = varuvAy - celavu;
accu payan;
```

### Example 3: Mixed Usage
```etamil
எண் income = 50000;
செலவு = 15000;
eN profit = income - செலவு;
accu "Profit: " & profit;
```

---

## Reference Tables

### Quick Consonant Lookup (eTamil Standard)

| Category | Tamil | eTamil | Note |
|----------|-------|--------|------|
| **Velars** | க ங | k **w** | ங = w NOT 'ng' |
| **Palatals** | ச ஞ | c **W** | ஞ = W NOT 'nj' |
| **Retroflex** | ட ண ல ற | t **N** l **R** | ற = R (capital) |
| **Dental** | த ந ப ம | **q** **N** p m | த = q NOT 'th' |
| **Approximants** | ய ர ல வ | y r l **v** | வ = v NOT 'w' |
| **Fricative** | ழ | **z** | z NOT 'zh' |
| **Lateral** | ள | **L** | L (capital) |

### Quick Vowel Lookup (eTamil Standard)

| Type | Tamil | eTamil | Example | Note |
|------|-------|--------|---------|------|
| **Short a** | அ | a | அச்சு (accu) | - |
| **Long a** | ஆ | A | - | Uppercase |
| **Short i** | இ | i | இருப்பு (iruppu) | - |
| **Long i** | ஈ | I | நீ (nI) | Uppercase |
| **Short u** | உ | u | உள்ளிடு (uLLitu) | - |
| **Long u** | ஊ | U | - | Uppercase |
| **Short e** | எ | e | எண் (eN) | - |
| **Long e** | ஏ | E | தேதி (qEti) | Uppercase |
| **Diphthong ai** | ஐ | Y | வைப்பு (vYppu) | Y (NOT 'ai') |
| **Short o** | ஒ | o | கோப்பு (kOppu) | - |
| **Long o** | ஓ | O | - | Uppercase |
| **Diphthong au** | ஔ | V | - | V (NOT 'au') |

---

## Integration Notes

1. **Parser Integration**: The lexer (`src/lexer.rs`) uses the `logos` crate with regex patterns supporting both Tamil and romanized forms
2. **Standard Used**: ONLY eTamil standard from `ezuqqu.pdf` - NOT ISO 15919 or any other transliteration
3. **Dual Support**: Both Tamil script and eTamil romanized forms are accepted interchangeably in code
4. **Documentation**: All examples and error messages use Tamil forms with eTamil romanized equivalents in parentheses
5. **Consistency**: All romanization must follow the eTamil standard exclusively

---

## Related Documentation

- [ACTUAL_KEYWORDS.md](ACTUAL_KEYWORDS.md) - Complete keyword reference
- [QUICK_REFERENCE.md](QUICK_REFERENCE.md) - Quick syntax guide
- [HOW_TO_USE.md](HOW_TO_USE.md) - Getting started guide
- [src/lexer.rs](src/lexer.rs) - Source code with all token definitions

---

**Last Updated**: February 1, 2026  
**Version**: 2.0 (Corrected to eTamil Standard Only)  
**Maintained by**: eTamil Development Team  
**Reference**: ezuqqu.pdf Tamil Letter Equivalents (eTamil Standard ONLY)
