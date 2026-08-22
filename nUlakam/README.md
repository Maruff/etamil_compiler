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
| `poruL.qmz` | records — `புலம்_உள்ளதா` `புலம்_அல்லது` `புலங்கள்` `மதிப்பீடுகள்` `காலியா_பதிவேடு` |
| `cOqaZY.qmz` | tests — `சோதனை_தொடக்கம்` `உறுதிசெய்` `சமம்` `வேறுபடு` `சேர்_ஓட்டம்` `சோதனை_முடிவு` |
| `vawki/vatti.qmz` | interest — `எளிய_வட்டி` `நாளாந்த_வட்டி` `கூட்டு_வட்டி` `முதிர்வுத்_தொகை` `அடுக்கு` `நாட்கள்` |
| `vawki/kadaZ.qmz` | loans — `மாதத்_தவணை` `தவணை_அட்டவணை` `மொத்த_வட்டி` `மொத்தத்_திருப்பி` `முன்கூட்டியே_அடைத்தால்` |
| `vawki/coqqu.qmz` | asset classification — `விதிமுறைகளை_ஏற்று` `வகைப்படுத்து` `ஒதுக்கீடு` `சரிபார்க்கப்படாதவை` |
| `cawkili/fabric.qmz` | Hyperledger Fabric — `நுழைவு` `மதிப்பிடு` `சமர்ப்பி` `மீண்டும்_சமர்ப்பி` `மோதலா` |
| `upi/vilAcam.qmz` | UPI addresses and pay links — `முகவரி_சரியா` `தொகை_சரியா` `தொகை_உரை` `பணம்_இணைப்பு` `இணைப்பைப்_படி` |
| `upi/nilYmY.qmz` | UPI payment states — `பணம்_வந்ததா` `சரிபார்க்கவா` `நகர்வு_சரியா` `நகர்த்து` |
| `kaNakkiyal/qEymAZam.qmz` | depreciation — `நேர்கோட்டு_ஆண்டு` `குறையும்_ஆண்டு` `பகுதி_ஆண்டு` `நேர்கோட்டு_அட்டவணை` `குறையும்_அட்டவணை` `தொகுதி_தேய்வு` |
| `kaNakkiyal/Uqiyam.qmz` | payroll — `மொத்தச்_சம்பளம்` `நாட்களுக்கு_ஏற்ப` `வரம்புடன்_பங்களிப்பு` `தகுதிக்குள்_பங்களிப்பு` `படிநிலை_வரி` `பணிக்கொடை` `சம்பளச்_சீட்டு` |
| `kaNakkiyal/vari_viziqam.qmz` | tax rates — `விகிதம்_தேடு` `படிகளை_ஏற்று` `படி_வரி_கணக்கிடு` `உள்_மாநிலமா` `மாநிலப்_பெயர்` |
| `qaLam/retis.qmz` | Redis — `சேமி` `காலத்துடன்_சேமி` `எடு` `இருக்கிறதா` `நீக்கு` `ஒன்று_கூட்டு` `முன்_சேர்` `வரிசைப்_பகுதி` `இல்லையெனில்_இயல்பு` |
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

## Tests are written here too

A library written in eTamil used to be testable only from Rust, or by running
an example and seeing whether it exited 0. Neither says which assertion failed.
`cOqaZY.qmz` is the alternative, and `kaNakkiyal/vari_cOqaZY.qmz` is a suite
using it — fifteen assertions about GST arithmetic, runnable like any program:

```bash
etamil --vm nUlakam/kaNakkiyal/vari_cOqaZY.qmz
```

It exits non-zero when anything fails, so a suite that reports a failure also
fails whatever ran it. The run is threaded through each assertion rather than
kept in a module variable, because a function cannot change a global: assigning
to a name inside a `செயல்` makes a local, and a counter incremented there is
lost on return.

## Banking: the engine here, the figures in a table

`vawki/` holds interest, loans and asset classification. Not one regulatory
number is in any of them.

Interest and instalments are arithmetic, so those modules take a rate as an
argument and hold none. `அடுக்கு` raises a base by repeated multiplication
rather than borrowing a floating-point `pow`: every number here is a
fixed-point decimal precisely so that money does not drift, and a float at the
step that compounds would put the drift back.

The amortisation schedule closes exactly. An instalment is rounded to the
paisa and paid two hundred and forty times, so the rounded parts do not sum to
the loan; the last row absorbs the difference. A schedule ending four paise
overdrawn is not a rounding detail, it is an account that will not close.

Classification is different, because the day counts and provisioning
percentages are set by circular and amended. They live in
`vawki/coqqu_viqimuRY.sql`, effective-dated, so that a review run against last
year's rules still produces last year's answer — and so that an amendment is a
new row rather than an edit. **The figures shipped there are placeholders and
are marked as such.** `சரிபார்க்கப்படாதவை` lists any still marked that way, so
a program can refuse to report a number nobody has vouched for.

An account no rule covers is refused rather than called standard. Standard
provisions least, and that is the one direction a provisioning error must
never go.

## Fabric, without gRPC

Fabric's peer Gateway speaks gRPC, which eTamil does not. `cawkili/fabric.qmz`
talks to a REST gateway instead — Firefly is one, and Hyperledger's own
`fabric-rest-sample` is a reference for writing one. The alternative was months
of HTTP/2 and protobuf to reach the same ledger.

Gateways disagree about their paths, so paths are configuration rather than
something baked in. Two operations, and the difference between them is the
whole of Fabric: `மதிப்பிடு` asks a peer and changes nothing, `சமர்ப்பி`
proposes, endorses, orders and commits.

`மோதலா` is the part worth having. Two transactions that read the same key and
both write it cannot both commit: the second is rejected at validation with
`MVCC_READ_CONFLICT`, *after* endorsement and ordering have already succeeded.
It means "someone got there first", not "this was wrong", and it is the one
Fabric error worth retrying. `மீண்டும்_சமர்ப்பி` retries that and nothing
else — retrying a chaincode refusal turns one rejection into several.

`gateway.py` beside the module is a mock that answers the way a gateway does,
including failing the first write to a contended key. The suite runs against it
when `ETAMIL_FABRIC` names one and skips cleanly when it does not.

## UPI: the public part, and the part that is not

A VPA and the `upi://` link are specified and checkable without anyone's
permission — a QR sticker in a shop is that link and nothing else.
`upi/vilAcam.qmz` builds and checks them, including the two things that catch
people: an amount with more than two decimal places is not money UPI takes,
and a decimal normalises `249.50` to `249.5` when some PSPs hold you to both
places.

**Moving money is not public.** That needs a payment service provider or bank
to sponsor you and NPCI to certify you, and no library substitutes for either.
What is here is everything up to the moment a request leaves for your PSP —
and the mutual TLS and ECDSA signing it will ask for already exist.

`upi/nilYmY.qmz` exists for one rule:

> **PENDING IS NOT FAILURE.**

A request that has not answered may still succeed — the payer's bank may be
slow, the switch may be retrying, the answer may arrive in an hour. Treating
that as failure is how a merchant refunds a payment that then lands, or charges
a customer twice. It is the most common way a UPI integration loses money,
because the wrong behaviour is the one that feels safe.

So two questions are asked separately. `பணம்_வந்ததா` — may I ship? Only a
settled success says yes. `சரிபார்க்கவா` — must I ask again? Anything unsettled
says yes, *including a state nobody recognises*. And `நகர்வு_சரியா` refuses to
let a late or duplicated callback rewrite a settled payment: a success does not
become a failure because a later poll was confused.

## ULI is not here

The Unified Lending Interface is access-controlled, and its interface is not
published in a form anyone outside can build against. Writing a client for it
from guesswork would produce something that looks finished and works against
nothing. When you have the specification through an authorised channel, the
pieces it needs — mutual TLS, ECDSA signing, JSON, a state machine — are all
in place.

## Depreciation, payroll, and where the rates live

Three things worth knowing before using these.

**A schedule closes exactly.** A depreciation charge rounded to the paisa and
taken five times need not equal what is to be written off; the last year takes
the remainder. An asset left with eight paise on the books never closes, the
same way a loan does not.

**A ceiling and an eligibility limit are not the same thing.** Provident fund
contributes on wages *up to* a ceiling. Employees' state insurance contributes
nothing at all above its limit — not a share of the limit. Treating one like
the other takes a deduction from someone who owes none, and the test says so.

**Slabs are marginal.** `படிநிலை_வரி` applies each band's rate only to the part
of the amount inside it. Applying the top rate to the whole amount is what "I
moved up a bracket and took home less" describes, and it is simply wrong. One
engine serves income tax, professional tax and anything else stated in bands —
including bands that carry a flat amount rather than a rate, which is how
professional tax usually reads.

### The rate tables

`vari.qmz` computes a tax once you know the rate. `vari_viziqam.qmz` is where
the rate comes from, and holds none: GST by HSN, TDS by section, income tax and
professional tax as slabs, VAT where it survives — one effective-dated table,
described in `vari_viziqam.sql`.

**Every function takes the date it is being asked about, and none defaults it
to today.** A return for last quarter is computed on the rates in force last
quarter; a rate looked up "as of now" quietly rewrites what was filed. A state
rule beats an all-India one, because that is what a state rule is for.

**No rate is seeded. Not one.** What ships is the shape, and the 36 states and
union territories with their GST codes — marked for checking, because a wrong
state code files a return in the wrong state. A missing rate answers a `தவறு`
rather than zero: a rate of nothing and no rate at all are different, and
returning zero for the second understates a liability without saying so.

## Redis is a command, not a query

The roadmap said Redis needed a design before an implementation, because it
does not fit a trait shaped as `execute(sql)` / `query(sql)`. It does not, and
forcing it there would have been the wrong answer: Redis is a command and a
reply, not a query language.

So the host gives exactly one thing — `ரெடிஸ்_கட்டளை(command, arguments)` —
and every Redis command works through it, including ones invented after it was
written. `qaLam/retis.qmz` wraps the handful anybody types.

RESP is implemented in the compiler rather than taken from a crate, for the
same reason the HTTP router was: the protocol is small and a dependency
carrying an async runtime to send `*2
$3
GET
$1
k
`
down a socket is a poor trade.

Two things it gets right that are easy to get wrong. **Arguments are
length-prefixed**, so a value containing CRLF is bytes rather than a second
command — joining arguments with spaces is how command injection works, and it
is not reachable through this. And **a missing key is nil, not `""`**: for a
cache, "absent" and "present and empty" are different questions, and
`வகை(x) == "nil"` tells them apart.

A connection is not shared between requests. Redis keeps state on one — MULTI,
WATCH, SUBSCRIBE — so two requests sharing a connection would interleave a
transaction the way two sharing a SQL connection do. The fix is an exclusive
lease, which the SQL side has and this does not yet.

`retis_pOli.py` is a small mock Redis, so the suite runs on a machine with none
installed.
