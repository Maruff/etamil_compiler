# nAtkAtti — நாட்காட்டி (the calendar)

`kaNakkiyal/kAlam.qmz` answers whether a date falls inside a reporting period,
which needs no arithmetic at all — ISO-8601 text sorts chronologically, so it
is a string comparison. This directory is everything that *does* need
arithmetic.

The host provides exactly three primitives and nothing else:

```
இன்று()                      today, in UTC
நாள்_வேறுபாடு(அ, ஆ)          whole days from அ to ஆ, negative if earlier
நாள்_கூட்டு(நாள், நாட்கள்)    a date shifted by whole days
```

Everything here is built from those three.

## Modules

| File | Contents |
|---|---|
| `nAL.qmz` | dates — `நாள்_வடிவமா` `நாள்_செல்லுபடியா` `நாள்_சரிபார்` `நாள்_ஆக்கு` `ஆண்டைப்_பெறு` `மாதத்தைப்_பெறு` `நாளைப்_பெறு` `நெட்டாண்டா` `மாத_நாட்கள்` `மாத_முதல்` `மாத_இறுதி` `மாத_இறுதியா` `மாதங்களைக்_கூட்டு` `ஆண்டுகளைக்_கூட்டு` `முடிந்த_மாதங்கள்` `முடிந்த_ஆண்டுகள்` `பகுதி_மாதங்கள்` `முந்தையது` `பிந்தையது` `வாரநாள்` `வாரநாள்_பெயர்` `நிதியாண்டின்_தொடக்க_ஆண்டு` `கால்_ஆண்டு_எண்` |
| `vElYnAL.qmz` | working days — `நாட்காட்டி_ஆக்கு` `வார_ஓய்வா` `விடுமுறையா` `வேலை_நாளா` `வேலை_வாரம்_உள்ளதா` `அடுத்த_வேலை_நாள்` `முந்தைய_வேலை_நாள்` `நகர்த்தி_வேலை_நாள்` `வேலை_நாட்களைக்_கூட்டு` `வேலை_நாட்களை_எண்ணு` `முடிவு_நாள்` `பணி_அட்டவணை` `உருட்டு` |
| `nAtkAtti_cOqaZY.qmz` | the tests — 115 assertions |

```bash
etamil --vm nUlakam/nAtkAtti/nAtkAtti_cOqaZY.qmz
```

## Six decisions worth knowing

### The host does not reject an impossible date

`நாள்_கூட்டு("2026-02-31", 0)` answers **`2026-03-03`**. The host's parser
checks only that the month is 1-12 and the day 1-31, then rolls the overflow
forward — so the 31st of February is not an error, it is the 3rd of March.

`நாள்_செல்லுபடியா` catches it by **round trip**: format the date, hand it to
the host, and compare what comes back. A date that returns different from the
one that went in was never a date. One comparison covers 31 February, 30
February in a leap year, 29 February in a common one, and 31 April.

It does its own shape and range checks **first**, before the host sees the
string, because a month of 13 is not a `தவறு` there — it is a runtime error
that stops the program. A library used at the edge of a system, where malformed
input is the only kind that arrives, cannot stop the program.

### Everything else assumes a valid date

Validating inside every function would mean every function returned a Result,
and a calendar whose every call has to be unwrapped is a calendar nobody will
use. Validate at the edge — where the date arrives from a person, a file or a
database — and the arithmetic stays readable. `kAlam.qmz` makes the same
choice.

### Adding a month is not adding thirty days

The 31st of January plus one month is the **28th of February**. There is no
31st, so the day clamps to the end of the target month rather than rolling into
March.

Clamping does not reverse, and that is a property of the calendar rather than a
defect: 31 January + 1 month = 28 February, and 28 February − 1 month = 28
January. So a schedule built by repeatedly adding one month to the *previous*
result drifts — 31 Jan, 28 Feb, 28 Mar — while one built by adding n months to
the *original* does not. The tests assert both, including the drift. **Always
count from the original.**

### Months are counted, not divided

`பகுதி_மாதங்கள்` implements the Income Tax Act's "every month or part of a
month" as calendar months. Days divided by thirty and rounded up is the usual
shortcut and it disagrees on any month of 31 days: 1 January to 1 February is
**one** month here and **two** by the division. Seven months of twelve are
long, so the error accumulates rather than averaging out — a full year is
twelve months by this rule and **thirteen** by the division.

The division always overcharges and never undercharges, which is why it
survives review: nobody disputes an interest computation in their favour.

This is what `muZvari.qmz` and `mUlavari.qmz` were waiting for. Both take a
count of months, and both say in their own comments that a caller holding real
dates should count calendar months instead of dividing. This is that counter.

### There is no default calendar

No default weekend, no built-in holiday list. A factory works Saturdays, a bank
does not, a project spanning two states has two holiday lists, and every one of
those varies by year. `நாட்காட்டி_ஆக்கு` takes the weekly offs and the
holidays, and every function takes a calendar — the tests run the same week
through a five-day office (four working days) and a six-day factory (six).

**A calendar with all seven days off is refused rather than searched.** Every
function that walks forward looking for a working day would otherwise not
terminate. That is a configuration error and it fails loudly.

### A duration includes its start

An activity of one day starting Monday finishes **Monday**. So `முடிவு_நாள்` is
start plus (duration − 1) working days, and the start is rolled onto a working
day first because nothing begins on a Sunday.

Adding the full duration is the commonest error in scheduling code, it makes
every activity one day too long, and along a path of twenty activities it
compounds into most of a month. The tests assert the correct finish and assert
that it differs from the naive one.

`பணி_அட்டவணை` returns start, finish, working days and **elapsed** days
together, because the gap between the last two is exactly the weekends and
holidays — which is what a Gantt chart shows and a duration does not. Ten
working days from Monday 2 March 2026 finish on Monday the 16th, fifteen
elapsed days later.

## Rolling conventions

A due date that lands on a holiday has to move, and which way is a term of the
contract:

| `மரபுப்_பெயர்` | |
|---|---|
| `"அடுத்தது"` | forward to the next working day |
| `"திருத்திய_அடுத்தது"` | forward, unless that leaves the month, then backward |
| `"முந்தையது"` | back to the previous working day |
| `"திருத்திய_முந்தையது"` | back, unless that leaves the month, then forward |

The modified forms keep a monthly obligation inside its own month. A payment
due on Sunday 31 May 2026 rolls forward to 1 June under *following* — into the
next accounting period, the next quarter, and for an Indian financial year
starting in April, a different quarter's return. Under *modified following* it
goes back to Friday the 29th and stays in May. That is why this is not a flag
for "roll forward", and why an unknown convention is refused rather than
guessed.

## Where it stops

- **No times, no time zones.** A date is a date. `இன்று()` is UTC, which is
  already the wrong day for part of every Indian evening — a caller who needs
  local dates should get them from its own clock rather than trust this.
- **No week numbers.** ISO-8601 week numbering has its own year, which disagrees
  with the calendar year at both ends and is a reliable source of off-by-one
  errors. Nothing in costing needs it yet.
- **No recurrence rules.** "The second Tuesday of every month" is a scheduler's
  job, and the primitives here are what one would be built from.
- **No holiday data.** Deliberately, and permanently. Indian public holidays are
  declared by the centre and by each state, they change every year, and several
  are decided on the sighting of the moon. They belong in a table with an
  effective date, exactly like `vari_vikiqam.sql`.
- **Years are 0001-9999.** The four-digit form has no room for anything else.

## What is now unblocked

`qittam/pAqY.qmz` computes the critical path in abstract time units.
`முடிவு_நாள்` and `பணி_அட்டவணை` turn one into real dates, which is what a
Gantt chart and a schedule of payments both need. The interest computations in
`nErativari/` can stop dividing by thirty. Neither change is made here — this
directory only supplies the arithmetic.
