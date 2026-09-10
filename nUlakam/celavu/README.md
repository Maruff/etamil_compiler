# celavu — செலவு கணக்கியல் (cost accounting)

`kaNakkiyal/` answers the question a financial accountant is asked: what did
the entity spend, and what is it worth. This directory answers the question a
cost accountant is asked, which is a different one: **where did the cost go,
what did it attach to, and what should it have been.**

The two are not layered on each other. A ledger records ₹42,000 of material
against one account; a cost sheet has to say how much of it reached a unit of
product, how much sat in work-in-progress, and how much never belonged to the
product at all. That reallocation is what these five modules do.

## Modules

| File | Contents |
|---|---|
| `nilYyam.qmz` | cost centres, objects and elements — `நிலையம்_ஆக்கு` `உற்பத்தி_நிலையமா` `வகையால்_நிலையங்கள்` `செலவுப்_பொருள்_ஆக்கு` `கூறு_ஆக்கு` `நேரடிக்_கூறுகள்` `மாறும்_பகுதி` `நிலையான_பகுதி` |
| `celavu_qAL.qmz` | the cost sheet — `நுகர்ந்த_பொருட்கள்` `முதன்மைச்_செலவு` `தொழிற்சாலைச்_செலவு` `உற்பத்திச்_செலவு` `விற்ற_பொருட்களின்_செலவு` `விற்பனைச்_செலவு` `அலகுக்குச்_செலவு` `செலவுத்_தாள்_ஆக்கு` `தாளை_அச்சிடு` |
| `mElnilY.qmz` | overhead absorption — `உள்வாங்கல்_விகிதம்` `சதவீத_விகிதம்` `உள்வாங்கியது` `உள்வாங்கல்_வேறுபாடு` `அதிக_உள்வாங்கலா` `நிலையங்களுக்குப்_பகிர்` `சேவையை_மறுபகிர்` |
| `niyamam.qmz` | standard costing and variances — `பொருள்_வீத_வேறுபாடு` `பொருள்_பயன்பாட்டு_வேறுபாடு` `ஊதிய_வீத_வேறுபாடு` `ஊதிய_திறன்_வேறுபாடு` `ஊதிய_செயலிழப்பு_வேறுபாடு` `மாறும்_மேல்நிலை_*` `நிலையான_மேல்நிலை_*` `விற்பனை_*` `சாதகமா` `வேறுபாட்டு_உரை` |
| `pawkaLippu.qmz` | marginal costing and CVP — `அலகு_பங்களிப்பு` `லாபம்` `பங்களிப்பு_விகிதம்` `சமநிலை_அலகுகள்` `சமநிலை_மதிப்பு` `பாதுகாப்பு_வரம்பு` `இலக்கு_அலகுகள்` `அலகு_வளத்திற்கு_பங்களிப்பு` |
| `celavu_cOqaZY.qmz` | the tests — 80 assertions, run it to check the arithmetic |

```bash
etamil --vm nUlakam/celavu/celavu_cOqaZY.qmz
```

## Four decisions worth knowing before you use it

### Compound names, because finance is the vocabulary

`செலவு`, `விலை`, `தொகை`, `அளவு`, `நிலை` and `மதிப்பு` are all **keywords** in
this language — that is what makes it a finance DSL rather than a general one.
A keyword used as a field name is stored under its token name, so the record
read back is not the one written. Every parameter and field here is therefore
compound: `மாறும்_செலவு`, `நியம_வீதம்`, `கூறு_தொகை`. The same convention runs
through `kaNakkiyal/`, and `kaNakkukaL.qmz` states it.

### A positive variance is favourable

Used without exception in `niyamam.qmz` and `mElnilY.qmz`:

```
variance = standard − actual
```

so a positive number is favourable and a negative one adverse. Texts differ —
many report the absolute figure with an F or A label — and a mixed convention
inside one system produces reports that add to nothing. `சாதகமா()` and
`வேறுபாட்டு_உரை()` exist so callers never have to remember which way it runs.

The one exception is `விற்பனை_வீத_வேறுபாடு`, where selling *above* standard
price is favourable, so it reads actual less standard. It is commented as the
exception at the point of definition.

### Percentages are whole numbers

`பங்களிப்பு_விகிதம்(250, 150)` answers `40`, meaning forty per cent. The
language has a `%` literal and `40%` is `0.40`, so passing `40%` to any of
these gives an answer a hundred times too small and **nothing complains** — the
type checker cannot tell one number from another. `kaNiqam/சதவீதம்` divides by
100 itself. `vari_cOqaZY.qmz` pins the same trap for GST.

### Division by nothing is `தவறு`, not zero

A product contributing nothing has no break-even point; a period with no units
has no unit cost; an overhead base of zero has no rate. These return `தவறு`
with a message rather than an infinity or a silently wrong zero, because a
costing report that quietly prints `0` for a unit cost is worse than one that
refuses.

## Where it stops

Deliberately not attempted, and each for a reason:

- **Reciprocal service apportionment.** Two service centres serving each other
  needs simultaneous equations. Use the step-down order, or solve it and pass
  the result to `சேவையை_மறுபகிர்`.
- **Process costing and equivalent units.** Needs a normal-loss and
  abnormal-loss model to be worth having, which is its own module.
- **Activity-based costing.** `mElnilY.qmz` records `அடிப்படை_வகை` with every
  rate precisely so that a driver-based successor can be added without
  changing what already exists.

Money that must add back exactly — apportionment, splitting a pool across
centres — is delegated to `kAcu.qmz`, which works in paise and guarantees the
parts sum to the whole. Naive division does not.
