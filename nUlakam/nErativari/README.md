# nErativari — நேரடிவரி (direct tax)

`kaNakkiyal/vari.qmz` is indirect tax: GST on a transaction, computed when the
transaction happens, borne by somebody other than the person who remits it.
This directory is **direct** tax — charged on a person's income, computed once
a year, borne by the person who pays it.

The two are different disciplines wearing the same word. Indirect tax is an
attribute of an invoice. Direct tax is a computation over a whole year that
cannot be finished until the year is over, must nevertheless be paid while it
is still running, and is settled a year later against a figure nobody knew at
the time.

## Modules

| File | Contents |
|---|---|
| `varumAZam.qmz` | the five heads and loss set-off — `சம்பள_வருமானம்` `நிகர_ஆண்டு_மதிப்பீடு` `வாடகைச்_சொத்து_வருமானம்` `சொந்த_வீட்டு_வருமானம்` `வீட்டு_இழப்பை_ஈடுசெய்` `குறுகிய_காலமா` `குறியீட்டுச்_செலவு` `மூலதன_ஆதாயம்` `விலக்கிய_ஆதாயம்` `ஈட்டு_விதிகள்` `ஈடுசெய்ய_முடியுமா` `ஈட்டைச்_செய்` `மொத்த_வருமானம்` |
| `kazivukaL.qmz` | Chapter VI-A — `கழிவு_ஆக்கு` `தனி_வரம்பு` `கூட்டுக்_கழிவுகள்` `மீதமுள்ள_இடம்` `நன்கொடைக்_கழிவு` `கழிவுக்குரிய_வருமானம்` `அனுமதிக்கத்தக்க_கழிவு` `இழந்த_கழிவு` `வருமானத்தை_வட்டமிடு` `மொத்த_வரிக்குரிய_வருமானம்` |
| `varikkaNakku.qmz` | total income to tax payable — `அடிப்படை_வரி` `தள்ளுபடியைக்_கணக்கிடு` `மேல்வரி_படி` `மேல்வரி_தொகை` `விளிம்பு_நிவாரணம்` `கழிவு_வரி` `வரியை_வட்டமிடு` `வரியைக்_கணக்கிடு` `முறைகளை_ஒப்பிடு` |
| `mUlavari.qmz` | tax deducted at source — `பிரிவு_ஆக்கு` `பிடிக்க_வேண்டுமா` `பிடித்த_அடிப்படை` `பயன்படும்_வீதம்` `வரி_நீக்கிய_அடிப்படை` `பிடித்தத்_தொகை` `ஆண்டுப்_பிடித்தம்` `பிடிக்காத_வட்டி` `செலுத்தாத_வட்டி` `சம்பள_பிடித்தம்` |
| `muZvari.qmz` | advance tax and interest — `மதிப்பிட்ட_வரி` `கடமை_உள்ளதா` `மூத்தவர்_விலக்கு` `முன்வரித்_தவணைகள்` `தவணை_அட்டவணையை_ஆக்கு` `வட்டிக்குரிய_அடிப்படை` `தவணை_வட்டி` `குறைபாட்டு_வட்டி` `தாமதத்_தாக்கல்_வட்டி` `செலுத்த_வேண்டியது` |
| `oqqivari.qmz` | deferred tax, Ind AS 12 — `வரி_அடிப்படை_சொத்து` `வரி_அடிப்படை_பொறுப்பு` `தற்காலிக_வேறுபாடு` `வேறுபாட்டு_வகை` `ஒத்திவரிக்_கணக்கு` `அங்கீகரிக்கத்தக்க_சொத்து` `இழப்பின்_ஒத்திவரி` `ஈடுசெய்யலாமா` `ஒத்திவரி_மாற்றம்` `வீத_ஒப்புரவு` |
| `nErativari_cOqaZY.qmz` | the tests — 198 assertions |

```bash
etamil --vm nUlakam/nErativari/nErativari_cOqaZY.qmz
```

## Six decisions worth knowing

### No statutory figure is written here

Not one slab, threshold, ceiling, percentage or rate. They are all parameters,
and the real ones are effective-dated in the table `vari_vikiqam.sql`
describes, read as they stood on the day being asked about.

This is not fastidiousness. A rate written into a library is a rate that will
be wrong one April, and wrong silently — the computation keeps working and
keeps producing last year's answer. `vari_vikiqam.qmz` already refuses to
default a date to today for the same reason.

The tests use **specimen** figures chosen so the arithmetic divides cleanly.
They are not defaults and they are not the law.

### The order of operations is the whole of the tax computation

Slab tax, less rebate, plus surcharge, less marginal relief, plus cess,
rounded. Every step is a subtraction or a percentage, and each wrong order is
somebody's published tax calculator:

- Cess before surcharge understates — cess is charged on the surcharge too.
- Rebate after cess overstates — the rebate is a rebate of income-tax.
- Surcharge on income rather than on tax overstates enormously — surcharge is
  a tax on a tax.
- Marginal relief omitted makes the liability jump by ₹1,24,550 for ₹10,000 of
  extra income.

`வரியைக்_கணக்கிடு` returns every intermediate figure, not just a total,
because a computation that disagrees with a departmental notice by four hundred
rupees tells you nothing about which of six steps to look at.

### Marginal relief is computed, not formulated

There is no closed form. `விளிம்பு_நிவாரணம்` runs the whole tax twice — once
at the actual income and once at the surcharge threshold — and caps the total
at *tax on the threshold plus the excess income*. Nobody pays more than a rupee
of extra tax for a rupee of extra income.

The tests pin it exactly: at ₹50,10,000 the surcharge of ₹1,31,550 is relieved
to ₹7,000, and income net of tax and surcharge is **identical** on both sides
of the threshold. Relief applies at every threshold, not only the first, and at
exactly ₹1 crore the 10% band still applies — which is why the function
re-derives the rate at the threshold instead of assuming nil.

### A threshold is not an exemption, and a band is not a reduction

Three provisions in this directory share a shape that behaves backwards from
the intuition, and each is a test of its own:

**TDS thresholds.** Once the year's aggregate is crossed, tax is deducted on
the *whole aggregate*, earlier sub-threshold payments included. In the tests,
eight payments of ₹20,000 produce nothing for five months and then ₹12,000 on
a ₹20,000 invoice. Deducting only on the excess is always an under-deduction.

**234C's relief band.** Pay 12% by the first date and no interest is charged
even though 15% was due. Pay ₹11,999 instead of ₹12,000 and interest runs on
the shortfall against **15%**, not against 12%. The band decides *whether*
interest applies; it never reduces it.

**234B's 90% test.** Pay 89% and interest runs on the whole 11% shortfall. Pay
90% and it is nothing. There is no proportioning anywhere in the section.

Against all three, `விலக்கிய_ஆதாயம்` is a genuine threshold exemption — the
part above the line is charged and the part below is not. The four look alike
in a rate table and three of them are cliffs.

### Deferred tax is a balance sheet computation

Ind AS 12 compares **carrying amount with tax base**. It is not a difference
between accounting and taxable income for the year — that was AS 22's timing
differences, and the two methods agree on most items and disagree on exactly
the ones that matter: revaluations, business combinations, anything recognised
in other comprehensive income. An income-statement method finds no difference
in those at all.

The sign flips between assets and liabilities, and half of all deferred tax
errors are that one flip. `வேறுபாட்டு_வகை(150000, மெய்)` is taxable and
`வேறுபாட்டு_வகை(150000, பொய்)` is deductible — the same signed number, opposite
answers.

**There is no discounting function in this module, and its absence is the
rule.** Ind AS 12 prohibits discounting deferred tax outright.

### Records use quoted keys

Every record in this directory is written `{"வரி": ...}` and read
`பதிவு["வரி"]`, never with bare keys and dot access.

Almost every natural field name in tax is a keyword in this language —
வருமானம், வரி, விலக்கு, இழப்பு, தொகை, நிகர, வாடகை, தள்ளுபடி — and a keyword
used as a bare key is stored under its *token* name, so the field read back is
not the field written. That failure is silent. Quoted keys cannot go wrong, and
the consistency is worth more than the few characters.

The same problem hits variables and parameters, where it is louder but not
always: `குறியீடு` is a hard parse error, while `வீதம்` and `தள்ளுபடி` parse
and misbehave. Everything bound in this directory is a compound.

## Where it stops

- **No presumptive taxation.** Sections 44AD, 44ADA and 44AE are a different
  computation of business income, not a different tax, and they belong with
  the business head rather than here. `ஒற்றைத்_தவணை_அட்டவணை` is the one place
  the regime shows up, because its advance tax schedule genuinely differs.
- **No clubbing, and no transfer pricing.** Both decide *whose* income an
  amount is, which is a question about facts and relationships rather than
  about arithmetic.
- **No carry-forward across years.** `ஈட்டைச்_செய்` reports what could not be
  set off and stops there. Carrying it forward needs a store of prior-year
  losses by kind and year, which is an application's schema and not a
  library's function.
- **No calendar.** Both interest computations take a count of months.
  `முழு_மாதங்கள்` and `மாதங்களாக` derive one from days by dividing by thirty
  and rounding up, which is right for a day count and disagrees with the
  statute's **calendar** months in February. A caller holding real dates should
  count the months itself.
- **No surcharge on income taxed at special rates.** The statute caps
  surcharge on certain capital gains and dividend income at a lower band, which
  needs the tax split by kind of income before surcharge is applied —
  a different shape from the single taxable figure `வரியைக்_கணக்கிடு` takes.

## What it builds on

`படிநிலை_வரி` in `kaNakkiyal/Uqiyam.qmz` already computes a banded tax
correctly, so `அடிப்படை_வரி` is a thin name over it rather than a second copy.
Income tax and professional tax are the same arithmetic on different tables.

`kaNakkiyal/qEymAZam.qmz` holds `தொகுதி_தேய்வு`, the block-of-assets
depreciation the Income Tax Act computes on, including the half-rate for assets
used for fewer than the threshold days. Depreciation for tax purposes lives
there and is not duplicated here; what this directory adds is the deferred tax
that arises because the books depreciate the same asset differently.
