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
| `col.qmz` | strings — `துண்டு` `தேடு` `பிரி` `ஒன்றிணை` `ஒழுங்கு` `தொடங்குகிறதா` `முடிகிறதா` `இடமிருந்து_நிரப்பு` |
| `kaNiqam.qmz` | math — `முழுமதிப்பு` `சிறியது` `பெரியது` `கூட்டு` `சராசரி` `சதவீதம்` |
| `aNi.qmz` | arrays — `உள்ளதா` `இடம்_காண்` `தலைகீழ்` `வெட்டு` `புலம்_எடு` `காலியா` |
| `paNam.qmz` | money — `ரூபாய்` `காசு_வடிவம்` `காசாக` `லட்சம்` `கோடி` |

## What the host provides

Only what cannot be expressed in the language itself:

`நீளம்` `இணை` `வகை` · `சரி` `தவறு` `சரியா` `தவறா` `மதிப்பு` `இயல்பு` ·
`வட்டமிடு` `தரை` `மேல்` · `சொல்லாக்கு` `எண்ணாக்கு` · `மேல்_எழுத்து` `கீழ்_எழுத்து`

Everything in this directory is built from those. Each also answers to a
romanized and an `_english` name — see `docs/reference/KEYWORDS.md`.

## Two behaviours worth knowing

**Strings are measured in written letters, not code points.**
`நீளம்("வணக்கம்")` is 5. A Tamil letter is often a consonant plus a vowel
sign or pulli, so counting code points would give 7 and every helper here
would be wrong on Tamil text.

**A keyword used as a record field is stored under its token name.**
`{வரி: 1000}` and `{vari: 1000}` both produce the field `Tax` — which is what
makes the two spellings interchangeable. If you look a field up by string,
use the canonical name, or use a field name that is not a keyword.

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
