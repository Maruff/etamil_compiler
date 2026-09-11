# qittam — திட்டச் செலவு (project costing)

`celavu/` costs a product: a thing made repeatedly, where the question is how
much of each period's spending reached a unit. This directory costs a
**project**: a thing made once, where the question is different and harder —
how much of the work is actually done, and what will the rest cost.

The two share their arithmetic. Earned value is standard costing applied to a
project, and `Ittu_maqippu.qmz` and `celavu/niyamam.qmz` compute the same kind
of variance from the same kind of flexed budget. What changes is the measure of
output: a production order counts units, a project counts budget earned.

## Modules

| File | Contents |
|---|---|
| `pakuppu.qmz` | the WBS — `கணு_ஆக்கு` `சேய்கள்` `இலைக்_கணுவா` `வேலைத்_தொகுப்புகள்` `உள்_கூட்டல்` `ஆழம்` `நூறு_விதி_சரியா` `விதியை_சரிபார்` |
| `kattuppAtu.qmz` | control accounts, time-phased budget, reserves — `கட்டுப்பாட்டுக்_கணக்கு_ஆக்கு` `பாதையில்_கணக்குகள்` `கட்டமைப்பை_சரிபார்` `காலப்_பகிர்வு` `திரட்டிய_நிதி` `நிதித்_தொகுதி_ஆக்கு` `அளவீட்டு_அடிப்படை` `அனுமதித்த_நிதி` `இருப்பை_விடுவி` |
| `Ittu_maqippu.qmz` | earned value — `முறை_0_100` `முறை_50_50` `முறை_மைல்கற்கள்` `ஈட்டிய_மதிப்பு` `செலவு_வேறுபாடு` `கால_வேறுபாடு` `செலவுச்_செயல்திறன்` `காலச்_செயல்திறன்` `முடிவில்_மதிப்பீடு_*` `முடிக்கத்_தேவையான_திறன்` `நிலவரம்` |
| `pAqY.qmz` | the critical path — `செயல்_ஆக்கு` `வலையைக்_கணக்கிடு` `கடுமையானவை` `பெறு_புலம்` `சுழற்சி_உள்ளதா` `உணர்திறன்_வலையா` |
| `curukkal.qmz` | crashing — `செயல்_நேர_செலவு` `செலவுச்_சரிவு` `அதிகபட்ச_சுருக்கம்` `மலிவான_வேட்பாளர்` `கூட்டுச்_சரிவு` `மொத்தச்_செலவு` `தாமதச்_செலவுடன்` |
| `qittam_cOqaZY.qmz` | the tests — 95 assertions |

```bash
etamil --vm nUlakam/qittam/qittam_cOqaZY.qmz
```

## Five decisions worth knowing

### Spend against budget answers nothing

A project has a budget of 1,550 and has spent 775. Is it in trouble?

The question has no answer, and that is why `Ittu_maqippu.qmz` exists. Two
numbers cannot describe three variables: spend says how much money left, budget
says how much was meant to, and neither says anything about **work
accomplished**. The test file pins the case exactly — a conventional report
reads **+25 favourable** while the cost variance is **−155** and the schedule
variance **−180**. The two readings disagree in direction, not degree.

### Schedule variance is money, not time

`கால_வேறுபாடு` of −180 does not mean the project is 180 behind. It means the
work not yet done was *budgeted* at 180. Converting that to time needs the
schedule — `pAqY.qmz` — not this module. Everyone meets this once.

### Percent complete is a rule, not an opinion

The hard part of earned value is not the arithmetic, it is deciding how much of
a package is done, and "about 80%" has been the status of many packages for
many months. Three conventions remove the judgement — `முறை_0_100`,
`முறை_50_50`, `முறை_மைல்கற்கள்` — and the module deliberately offers **no
function that accepts a typed-in percentage**.

### The WBS is stored flat

A list of nodes each naming its parent, not nested records. A nested record
cannot be indexed by code without walking it; re-parenting is a field
assignment rather than a rebuild; and the roll-up is a fold either way. The
root's parent code is the empty string.

`நூறு_விதி_சரியா` compares a parent's entered budget against its children's,
because a parent with more has work nobody has been assigned and one with less
has work counted twice. Both are invisible in a tree view. `விதியை_சரிபார்`
returns the offending node and both figures rather than a boolean — "the WBS
does not balance" is not actionable.

### Only critical activities are worth crashing

`மலிவான_வேட்பாளர்` takes the list of critical activity codes as an argument and
will not return anything outside it. Shortening an activity with float buys
nothing at all and is the most expensive mistake available, because it looks
like progress. The tests assert it: C is cheaper per day than D and is never
offered, because C has five days of float.

## Where it stops

- **Finish-to-start only.** Lags and the other three dependency types belong to
  a richer scheduler and would change every signature in `pAqY.qmz`.
- **No resource levelling.** It is an optimisation problem with no efficient
  exact solution, and a heuristic that cannot explain what it did will not be
  trusted.
- **Crashing is not automated.** The module gives the slope, the candidate and
  the totals; the caller loops, because the critical path changes after every
  step and where several paths are critical one activity per path must be
  crashed. `கூட்டுச்_சரிவு` prices that comparison.
- **No Monte Carlo.** Three-point estimates and PERT belong with it, and both
  need a random source the standard library does not yet have.

The passes in `pAqY.qmz` iterate to a fixed point rather than running over a
topological sort. For hand-built networks the cost is irrelevant, and the
benefit is that a cycle produces a `தவறு` from `சுழற்சி_உள்ளதா` rather than a
stack overflow. A cycle is the commonest structural error in a network and it
must fail loudly.
