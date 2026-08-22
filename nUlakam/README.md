# nUlakam — the eTamil standard library

Written in eTamil, not Rust. That is the point: if the standard library
needed a systems language, the DSL would not be sufficient for the
frameworks built on top of it.

```etamil
இறக்கு "nUlakam/paNam.qmz";

அச்சு ரூபாய்(12345678.5);        // ₹1,23,45,678.50
```

## Modules

| File | Contents |
|---|---|
| `col.qmz` | strings — `துண்டு` `தேடு` `ஒழுங்கு` `தொடங்குகிறதா` `முடிகிறதா` `திரும்பச்செய்` `இடமிருந்து_நிரப்பு` |
| `kaNiqam.qmz` | math — `முழுமதிப்பு` `சிறியது` `பெரியது` `கூட்டு` `சராசரி` `சதவீதம்` |
| `aNi.qmz` | arrays — `உள்ளதா` `இடம்_காண்` `தலைகீழ்` `வெட்டு` `புலம்_எடு` `காலியா` |
| `paNam.qmz` | money — `ரூபாய்` `காசு_வடிவம்` `காசாக` `லட்சம்` `கோடி` |
| `jEcAZ.qmz` | JSON — `ஜேசான்_ஆக்கு` `ஜேசான்_படி` |
| `kuRiyAkkam.qmz` | encoding — `அறுபத்துநான்கு_ஆக்கு` `அறுபத்துநான்கு_படி` `பதினாறு_ஆக்கு` `பதினாறு_படி` |
| `AvaNam.qmz` | documents — `ஆவணம்_நிரப்பு` `பொதியை_நிரப்பு` `pdf_ஆக்கு`, and the `ODT_வடிவம்` / `ODS_வடிவம்` / `DOCX_வடிவம்` / `XLSX_வடிவம்` shapes |

## JSON is written here, not in the host

A parser needs to build a record whose field names come from the data, and the
VM already allows that: `பொருள்[சாவி] = மதிப்பு` computes the key at runtime.
So `jEcAZ.qmz` is ordinary eTamil, and Layer 0 gains nothing.

```etamil
இறக்கு "nUlakam/jEcAZ.qmz";

ப = மதிப்பு(ஜேசான்_படி(request_body));
அச்சு ப["qokY"] + 1;                       // a number, not text
பதில் 200, ஜேசான்_ஆக்கு({நிலுவை: 1500});
```

`ஜேசான்_படி` returns `சரி`/`தவறு`, so malformed input is handled rather than
guessed at. Record fields serialize in sorted order, which makes a response
body stable enough to assert on. `\uXXXX` escapes are not decoded.

## What the host provides

Only what cannot be expressed in the language itself:

`நீளம்` `இணை` `வகை` · `சரி` `தவறு` `சரியா` `தவறா` `மதிப்பு` `இயல்பு` ·
`வட்டமிடு` `தரை` `மேல்` · `சொல்லாக்கு` `எண்ணாக்கு` · `மேல்_எழுத்து` `கீழ்_எழுத்து` ·
`இன்று` `நாள்_வேறுபாடு` `நாள்_கூட்டு` ·
`கடவுச்சொல்_மறை` `கடவுச்சொல்_சரியா` `சீட்டு_ஆக்கு` `சீட்டு_சரிபார்` ·
`கையொப்பம்` `கையொப்பம்_சரியா` · `வலை_பெறு` `வலை_பதி` `வலை_அனுப்பு` ·
`பைட்டுகள்` `பைட்டுச்_சரம்`

The last nine are bcrypt, JWT, HMAC and HTTP: hashing and signing need bytes,
randomness and a constant-time comparison the language cannot reach, and opening
a socket is a syscall. The last two are the only thing base64 needed: a byte is
not something the language can reach on its own, but once it has an array of
them the encoding is arithmetic, so `kuRiyAkkam.qmz` is ordinary eTamil. Everything above them — who a user is,
which route needs which role — stays in eTamil, and a token's payload crosses
the boundary as JSON text that `jEcAZ.qmz` reads and writes.

Everything in this directory is built from those. Each also answers to a
romanized and an `_english` name — see `docs/reference/KEYWORDS.md`.

## Two behaviours worth knowing

**Strings are measured in written letters, not code points.**
`நீளம்("வணக்கம்")` is 5. A Tamil letter is often a consonant plus a vowel
sign or pulli, so counting code points would give 7 and every helper here
would be wrong on Tamil text.

**A field name is stored exactly as written**, keyword or not: `{வரி: 1000}`
produces the field `வரி`. It used to be filed under the English token name
`Tax`, which anglicised the author's own words; that changed with roadmap item
2. The consequence is that `{வரி: 1}` and `{vari: 1}` are now *different*
fields, so a program should pick one spelling and keep to it.

## Import paths

`இறக்கு` looks beside the importing file, then along `ETAMIL_PATH`, then next
to the compiler binary. To use the library from anywhere:

```bash
export ETAMIL_PATH=/path/to/etamil_compiler
etamil --vm my_program.qmz
```

## Missing, and why

No `map` or `filter`: the language has no first-class functions yet. When
function values arrive, several loops here collapse to one line.

## பிரி and ஒன்றிணை moved to the host

They were written here, like everything else. Both walked the string one
letter at a time, and every read re-segmented the whole string into written
letters, so splitting a document cost O(n²) segmentations — measured at 14
seconds over 8 KB, and 400 KB never finished. They are host builtins now,
along with `மாற்று`, doing one segmentation pass and then a byte search.

A separator still only matches on a letter boundary, so `பிரி("கா", "ா")`
does not cut a letter in half. The pieces are the same pieces; only the cost
differs. A function defined here would shadow a builtin, so they are gone
from `col.qmz` rather than left to delegate.

## Documents are rendered here, not in the host

`.odt`, `.ods`, `.docx` and `.xlsx` are all zip archives of XML. The host
opens and rewrites the archive — `பொதி_படி` and `பொதி_மாற்று` — because a
picture inside one is not text and could not survive being a `சரம்`. What a
template *means* is decided in `AvaNam.qmz`: which placeholder gets which
value, which rows repeat, what has to be escaped.

```etamil
இறக்கு "nUlakam/AvaNam.qmz";

மதிப்புகள் = [{"குறி": "project.name", "மதிப்பு": "Beak PMO"}];
தொகுதிகள் = [{"பெயர்": "o",
              "புலங்கள்": ["no", "objective"],
              "வரிசைகள்": [{"no": "1", "objective": "One source of truth"}]}];

பொதியை_நிரப்பு("charter.odt", "out.odt", ODT_வடிவம், மதிப்புகள், தொகுதிகள்);
```

A `வடிவம்` is the whole of the difference between the formats: which entry
holds the text, and what a table row is called there. `{{ name }}` is a value
and `{%tr for x in list %}` … `{%tr endfor %}` repeats the rows between them,
which is the convention the templates already used.

An `.xlsx` keeps its text in a shared table and its rows in the sheet, so a
repeating row there would have to renumber shared-string indexes. Scalars
work; row groups do not, and `XLSX_வடிவம்` says so rather than half-doing it.
