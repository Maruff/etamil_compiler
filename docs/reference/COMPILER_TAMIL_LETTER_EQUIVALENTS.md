# Tamil letter equivalents — the ezuqqu scheme

How each Tamil letter is written in ASCII, and how to read a romanized eTamil
name back as Tamil.

**Status**: descriptive. The scheme itself lives in
`scripts/transliterate.py`, which `--check` audits the lexer against, and is
specified in [EZUQQU_TRANSLITERATION_STANDARD.pdf](EZUQQU_TRANSLITERATION_STANDARD.pdf).
Where this document and that script disagree, the script is right.

This is a guide to the **letters**. For the list of keywords see
[KEYWORDS.md](KEYWORDS.md), which is generated from the lexer and is the
authority on what the compiler accepts.

---

## The idea

One Latin letter stands for one Tamil letter. Case carries a letter
distinction rather than emphasis, which is what lets the scheme keep apart the
letters English writes the same way: Tamil has three nasals where English has
one `n`, and two rhotics and two laterals where English has one of each.

**The three nasals are the ones that get confused, so they come first:**

| Tamil | eTamil | Which nasal |
|---|---|---|
| ண | `N` | retroflex |
| ந | `n` | dental |
| ன | `Z` | alveolar |

`ண` and `ன` are different letters and different sounds, and neither is `n`.
Writing `n` for `ன` merges it with `ந` and the compiler then cannot tell
`பின்னம்` from a word spelled with `ந`.

---

## Consonants (மெய் எழுத்து)

The eighteen native consonants, in the traditional order:

| Tamil | eTamil | Class | Sound |
|---|---|---|---|
| க | `k` | vallinam | velar stop |
| ங | `w` | mellinam | velar nasal |
| ச | `c` | vallinam | palatal stop |
| ஞ | `W` | mellinam | palatal nasal |
| ட | `t` | vallinam | retroflex stop |
| ண | `N` | mellinam | retroflex nasal |
| த | `q` | vallinam | dental stop |
| ந | `n` | mellinam | dental nasal |
| ப | `p` | vallinam | labial stop |
| ம | `m` | mellinam | labial nasal |
| ய | `y` | idaiyinam | palatal approximant |
| ர | `r` | idaiyinam | alveolar tap |
| ல | `l` | idaiyinam | alveolar lateral |
| வ | `v` | idaiyinam | labial approximant |
| ழ | `z` | idaiyinam | retroflex approximant |
| ள | `L` | idaiyinam | retroflex lateral |
| ற | `R` | vallinam | alveolar trill |
| ன | `Z` | mellinam | alveolar nasal |

### Grantha letters, for borrowed sounds

These are in everyday Tamil use and the compiler accepts them:

| Tamil | eTamil |
|---|---|
| ஜ | `j` |
| ஷ | `S` |
| ஸ | `s` |
| ஹ | `H` |
| க்ஷ | `x` |

---

## Vowels (உயிர் எழுத்து)

Twelve vowels, plus the aytham. A vowel has two written forms: the independent
letter that begins a word, and the sign it takes after a consonant.

| Vowel | eTamil | Sign after a consonant | Length |
|---|---|---|---|
| அ | `a` | (none — inherent) | short |
| ஆ | `A` | ா | long |
| இ | `i` | ி | short |
| ஈ | `I` | ீ | long |
| உ | `u` | ு | short |
| ஊ | `U` | ூ | long |
| எ | `e` | ெ | short |
| ஏ | `E` | ே | long |
| ஐ | `Y` | ை | diphthong |
| ஒ | `o` | ொ | short |
| ஓ | `O` | ோ | long |
| ஔ | `V` | ௌ | diphthong |
| ஃ | `h` | — | aytham |

Short vowels are lower case and long vowels are the upper case of the same
letter. `ஐ` and `ஔ` are `Y` and `V`, one character each — **never `ai` or
`au`**. A two-character vowel would be a digraph, and a lexer reading `ai`
could not tell one long vowel from two short ones.

---

## The pulli, and why every vowel is written

A bare consonant symbol carries the inherent `a`. The pulli `்` strips it:

| Tamil | eTamil | Reads as |
|---|---|---|
| க | `ka` | க with its inherent vowel |
| க் | `k` | bare consonant, no vowel |

So the romanized form always writes the vowel. `k` is the bare consonant and
`ka` is the consonant with `a`; they are two different things and never
interchangeable. This is what makes the scheme reversible — if `k` could mean
either one, a romanized name could not be read back to a single Tamil spelling.

---

## Worked examples

Each of these is a real keyword. The breakdown splits the word into the units
the scheme consumes: an independent vowel, or a consonant carrying either the
pulli or a vowel sign.

| Tamil | eTamil | Letter by letter | Meaning | The letter to notice |
|---|---|---|---|---|
| எண் | `eN` | எ + ண் → `e + N` | number | ண் is N, the retroflex nasal |
| பின்னம் | `piZZam` | பி + ன் + ன + ம் → `pi + Z + Za + m` | fraction | ன்ன is ZZ, not nn |
| நிகர | `nikara` | நி + க + ர → `ni + ka + ra` | net | ந is n, the dental nasal |
| வினா | `viZA` | வி + னா → `vi + ZA` | query | ன is Z, the alveolar nasal |
| வங்கி | `vawki` | வ + ங் + கி → `va + w + ki` | bank | வ is v and ங் is w |
| மொத்த | `moqqa` | மொ + த் + த → `mo + q + qa` | gross | த்த is qq |
| உள்ளிடு | `uLLitu` | உ + ள் + ளி + டு → `u + L + Li + tu` | input | ள்ள is LL |
| பற்று | `paRRu` | ப + ற் + று → `pa + R + Ru` | debit | ற்ற is RR |
| தொகை | `qokY` | தொ + கை → `qo + kY` | amount | ை is Y |
| கோப்பு | `kOppu` | கோ + ப் + பு → `kO + p + pu` | file | ோ is O and ப்ப is pp |
| வருமானம் | `varumAZam` | வ + ரு + மா + ன + ம் → `va + ru + mA + Za + m` | income | ன is Z inside a long word |
| எழுது | `ezuqu` | எ + ழு + து → `e + zu + qu` | write | ழ is z and து is qu |

To check any word yourself:

```bash
python scripts/transliterate.py வருமானம்
```

---

## Gemination

A doubled consonant doubles its letter. The two to watch are the nasals whose
single forms are already easy to confuse:

| Tamil | eTamil | | Tamil | eTamil |
|---|---|---|---|---|
| க்க | `kk` | | ப்ப | `pp` |
| ங்ங | `ww` | | ம்ம | `mm` |
| ச்ச | `cc` | | ய்ய | `yy` |
| ஞ்ஞ | `WW` | | ர்ர | `rr` |
| ட்ட | `tt` | | ல்ல | `ll` |
| ண்ண | `NN` | | வ்வ | `vv` |
| த்த | `qq` | | ழ்ழ | `zz` |
| **ந்ந** | **`nn`** | | ள்ள | `LL` |
| | | | ற்ற | `RR` |
| | | | **ன்ன** | **`ZZ`** |

---

## This is not ISO 15919

ISO 15919 is the international transliteration standard, and it is a different
system built for a different purpose. It reaches its letter distinctions with
diacritics — `ṇ`, `ṉ`, `ḷ`, `ḻ`, `ṟ` — which are outside ASCII and have no key
on an ordinary keyboard. The ezuqqu scheme reaches the same distinctions with
case, so every letter is one keystroke.

Mixing the two produces names the compiler rejects or, worse, accepts as a
different name:

| Tamil | ezuqqu | ISO 15919 | A common mistake |
|---|---|---|---|
| ங் | `w` | `ṅ` | writing `ng` |
| ஞ் | `W` | `ñ` | writing `nj` |
| த் | `q` | `t` | writing `th` |
| வ் | `v` | `v` | writing `w` |
| ழ் | `z` | `ḻ` | writing `zh` |
| ற் | `R` | `ṟ` | writing lower-case `r` |
| ன் | `Z` | `ṉ` | writing `n`, which is ந |
| ஐ | `Y` | `ai` | writing `ai` |
| ஔ | `V` | `au` | writing `au` |

---

## Using it in code

A name is stored exactly as you typed it, so the Tamil spelling and the
romanized spelling are **different names**. `வருவாய்` and `varuvAy` are two
variables, not one. Pick one spelling for a program and keep to it.

### Tamil script

```etamil
எண் வருவாய் = 100000;
எண் செலவு = 30000;
எண் பயன் = வருவாய் - செலவு;
அச்சு பயன்;
```

### The same program romanized

```etamil
eN varuvAy = 100000;
eN celavu = 30000;
eN payaZ = varuvAy - celavu;
accu payaZ;
```

### English names carry a leading underscore

An English word written in ASCII is indistinguishable from a romanized Tamil
word, and under the eTamil font it renders as nonsense — `sum` draws as ஸும்.
So an identifier containing English begins with `_`, and an English comment is
wrapped in `__ … __`. Both rules are normative and are enforced by
`scripts/check_script_rules.py`; see [SCRIPT_RULES.md](SCRIPT_RULES.md).

```etamil
// __Mixed Tamil and English names__
எண் _income = 50000;
எண் செலவு = 15000;
எண் _profit = _income - செலவு;
அச்சு "Profit: " & _profit;
```

---

## Related documentation

- [KEYWORDS.md](KEYWORDS.md) — every keyword, generated from the lexer
- [SCRIPT_RULES.md](SCRIPT_RULES.md) — the `_` and `__` marks, normative
- [QUICK_REFERENCE.md](QUICK_REFERENCE.md) — quick syntax guide
- [QUICKSTART.md](../getting-started/QUICKSTART.md) — getting started
- [scripts/transliterate.py](../../scripts/transliterate.py) — the scheme, and
  the checker that audits the lexer against it
- [lexer.rs](../../etamil_compiler/src/lexer.rs) — the token definitions
