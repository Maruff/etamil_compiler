# vaLam — வளம் (resources, time, and actual cost)

`qittam/Ittu_maqippu.qmz` computes a cost variance from actual cost, and takes
actual cost as a number somebody types in. This directory is where that number
comes from, and where it ends up in the books.

It closes the loop the other directories left open:

```
timesheet  →  costed at the rate in force that day  →  posted to the ledger
                                                            ↓
                    qittam's AC  ←  read back per control account
```

## Modules

| File | Contents |
|---|---|
| `vaLam.qmz` | resources and rate cards — `வளம்_ஆக்கு` `விகிதப்_பதிவு` `விகிதத்தைத்_தேடு` `மேலதிக_விகிதம்` `காலத்_திறன்` `கிடைக்கும்_மணிகள்` `பயன்பாட்டு_விகிதம்` `கட்டண_விகிதம்` `வளத்தைத்_தேடு` |
| `nEraqqAL.qmz` | timesheets — `நேர_உள்ளீடு` `உள்ளீட்டைச்_சரிபார்` `நாள்_மொத்த_மணிகள்` `மிகைப்_பதிவா` `நிலை_மாற்றம்_சரியா` `நிலையை_மாற்று` `ஏற்றவை_மட்டும்` `உள்ளீட்டைக்_கணக்கிடு` `தாளைக்_கணக்கிடு` `பணிக்கு_மொத்தம்` `கட்டண_மணிகள்` `வளத்துக்கு_மொத்தம்` |
| `paqivu.qmz` | into the ledger — `நடப்புக்_கணக்குக்_குறி` `உழைப்புப்_பதிவு` `பொருள்_பதிவு` `துணை_ஒப்பந்தப்_பதிவு` `மேல்நிலைப்_பதிவு` `வருவாய்ப்_பதிவு` `விலைப்பட்டியல்_பதிவு` `முன்பணப்_பதிவு` `நட்ட_ஒதுக்கீட்டுப்_பதிவு` `ஈட்டிய_செலவு` `காலம்_வரை_செலவு` `செலவை_ஒப்பிடு` |
| `vaLam_cOqaZY.qmz` | the tests — 58 assertions |

```bash
etamil --vm nUlakam/vaLam/vaLam_cOqaZY.qmz
```

## Six decisions worth knowing

### A rate is effective-dated, and the date is never defaulted

The same person costs ₹500 an hour in March and ₹600 in April.
`விகிதத்தைத்_தேடு` takes the day it is being asked about and there is no form
of it that omits one — the same refusal `kaNakkiyal/vari_vikiqam.qmz` makes,
for the same reason.

A month's sheet costed at the rate current when it is processed is wrong for
every entry before the last change, and it **reconciles perfectly to the wrong
number**. The tests follow four days at ₹500 and four at ₹600: the answer is
₹17,600, and neither ₹19,200 (all at the new rate) nor ₹16,000 (all at the old)
looks anything but plausible.

A resource with no rate on a day is a `தவறு`, not zero. Charging time at
nothing understates a project by exactly the amount most worth knowing.

### A day is validated as a whole, not entry by entry

Six hours on one task and six on another are each unremarkable. Together they
are twelve, and one person did not work twelve hours on a day that holds eight.

`உள்ளீட்டைச்_சரிபார்` checks what one entry can be checked for — a real date,
positive hours, not more than a day holds. `மிகைப்_பதிவா` sums the day across
entries, and it is the only thing that finds a double booking. Every entry can
pass the first and the day still be impossible, which is why both exist.

This is the error that inflates actual cost on whichever project is easiest to
book to, and it is invisible in any report that groups by project rather than
by person.

The date check earns its place too. A timesheet is the one document in a
business filled in from memory, and a typed month gets the year wrong every
January — an entry dated a year earlier is costed at a rate from a year earlier
and lands in a closed period.

### Capacity is working days, not elapsed days

`காலத்_திறன்` counts working days from a `nAtkAtti/` calendar. A week with one
holiday holds 32 hours, not 56.

A utilisation percentage computed on elapsed days moves every month for reasons
that have nothing to do with anybody's work, and it is always the denominator
that is wrong. February and March differ by three working days in most years.

### Utilisation and chargeability are different questions

A resource can be **100% utilised and 0% chargeable** — every hour booked, all
of it to rework, internal meetings, or a fixed-price job already past its
budget. `பயன்பாட்டு_விகிதம்` and `கட்டண_விகிதம்` are separate functions because
a business that tracks one of them tracks the flattering one.

### Unapproved time is not cost

A sheet moves வரைவு → சமர்ப்பித்தது → ஏற்றது, and a submitted sheet may be
rejected back to draft. **Nothing comes back from approved.** An approved sheet
has been costed and posted, and is corrected by a further entry rather than by
editing the original — otherwise actual cost moves backwards, and a cost
variance computed last week cannot be reproduced this week.

### The ledger is where actual cost lives

Every posting function returns a balanced transaction in `pErEtu.qmz`'s own
shape. The arithmetic is two lines each; the value is that the **direction** is
decided once, with a name on it. A reversed entry balances perfectly.

Three of the directions are worth stating because they are the ones usually
written the other way:

- **Labour credits a payroll clearing account, not the bank.** Hours are cost
  when worked; money leaves when payroll runs, which is a different date and
  often a different month.
- **Overhead credits an absorption account, not the expense accounts.** The
  difference between absorbed and incurred is the over- or under-absorption
  `celavu/mElnilY.qmz` computes, and crediting the expenses directly makes it
  disappear along with the only evidence that the rate is wrong.
- **Invoicing recognises no revenue.** It debits the receivable and credits the
  contract asset, converting a conditional right into an unconditional one. The
  revenue was recognised when the work was done. Writing this entry as debit
  receivable, credit revenue undoes everything Ind AS 115 asks for.

**Work in progress is one account per control account**, built by
`நடப்புக்_கணக்குக்_குறி` rather than looked up in a table somebody has to keep
in step with the work breakdown. That is what makes `ஈட்டிய_செலவு` able to read
AC back out per control account — a single project WIP account collects the
same money and cannot say where it went.

`செலவை_ஒப்பிடு` compares the ledger against the timesheets. They should agree,
and when they do not, either something was posted that no timesheet supports or
approved time was never posted. Neither shows up in a report that looks at one
source alone, and a project whose books and timesheets disagree has an actual
cost that depends on who you ask.

## Where it stops

- **No allocation of a resource to a plan.** Who is meant to work on what, and
  whether they are overcommitted across projects, is resource levelling —
  `qittam/README.md` says why that is deliberately absent.
- **No payroll.** `kaNakkiyal/Uqiyam.qmz` computes a payslip; the credit here
  is a clearing account and reconciling the two is an application's job.
- **No purchase orders or goods receipt.** `பொருள்_பதிவு` posts an issue from
  stores and assumes the material is already there.
- **No currency.** A resource billed in one currency and costed in another
  needs `kaNakkiyal/niRuvaZam.qmz`'s translation and a decision about which
  rate applies to a timesheet.
- **No approval identity.** `நிலையை_மாற்று` enforces which transitions are
  legal, not who may make them. Authorisation belongs to an application.

## A note on `பேரேடு`

It is a keyword, and the ledger functions take it as a parameter anyway —
matching `kaNakkiyal/pErEtu.qmz`, so a reader comparing the two signatures sees
one word. A keyword in *parameter* position either works or fails loudly at
parse time; this one works, and the tests exercise it through a real ledger
round trip. The silent failure is a keyword used as a bare **record key**,
stored under its token name, and no record here has one.
