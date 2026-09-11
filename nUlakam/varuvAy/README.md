# varuvAy — வருவாய் (revenue, Ind AS 115)

`qittam/` costs a project and `celavu/` costs a product. Neither says what
either is *worth*, and for a business that builds things to order the revenue
side is most of the profit and loss.

Ind AS 115 replaced a standard that asked when the risks and rewards passed
with one that asks when **control** passes. For a project business the two give
different answers often enough to matter: work built on a customer's own site
transfers control continuously while risk stays with the contractor until
handover.

## Modules

| File | Contents |
|---|---|
| `oppanqam.qmz` | steps 1–4 — `ஒப்பந்தம்_ஆக்கு` `ஒப்பந்தம்_உள்ளதா` `கடமை_ஆக்கு` `தனிக்_கடமையா` `மாறும்_எதிர்பார்ப்பு` `மிக_வாய்ப்பான_தொகை` `கட்டுப்பாட்டுடன்` `பரிமாற்ற_மதிப்பு` `கடமைகளுக்கு_ஒதுக்கு` `ஒதுக்கியதன்_கூட்டல்` `எஞ்சிய_முறை` `தள்ளுபடியை_ஒதுக்கு` |
| `muZZERRam.qmz` | step 5 — `காலப்போக்கிலா` `கட்டுப்பாட்டு_குறியீடுகள்` `செலவு_முன்னேற்றம்` `வெளியீட்டு_முன்னேற்றம்` `நிறுவாத_பொருள்_முன்னேற்றம்` `நிறுவாத_பொருளுடன்_வருவாய்` `இதுவரை_வருவாய்` `இந்தக்_காலத்து_வருவாய்` `மீட்பளவு_வருவாய்` `வருவாய்_நிலவரம்` |
| `iruppukaL.qmz` | the balance sheet — `ஒப்பந்த_நிலை` `உரிமை_வகை` `தேக்கத்_தொகை` `தேக்கத்தை_வகைப்படுத்து` `விலைப்பட்டியல்_நிகரம்` `ஒப்பந்தங்களைத்_தொகு` `தவறாக_நிகரமிட்டால்` |
| `oppanqac_celavu.qmz` | contract costs and losses — `பெறுதல்_செலவின்_வகை` `நிறைவேற்றல்_செலவின்_வகை` `கழிப்புத்_தொகை` `முதலீட்டுக்_குறைவு` `தவிர்க்க_முடியாத_செலவு` `நட்டம்_தருமா` `நட்டக்_கணக்கு` |
| `varuvAy_cOqaZY.qmz` | the tests — 99 assertions |

```bash
etamil --vm nUlakam/varuvAy/varuvAy_cOqaZY.qmz
```

## Seven decisions worth knowing

### Revenue attaches to an obligation, not to a contract or an invoice

The five steps are in order for a reason: a contract whose performance
obligations have not been identified cannot be measured at all. The step that
gets skipped in practice is the second, and skipping it puts revenue in the
wrong periods without anything looking wrong.

`தனிக்_கடமையா` needs **both** conditions — capable of being distinct, *and*
separately identifiable within the contract. Bricks are capable of being
distinct; bricks supplied under a contract to build a wall are not separately
identifiable from the building service. Treating every line on the invoice as
an obligation is the commonest error in applying the standard.

### The constraint is half of variable consideration

An estimate of variable consideration is computed — by expected value or by
most likely amount — and then **held back** to the extent a significant
reversal could still occur. A bonus reckoned at ₹80 lakh may enter the
transaction price at ₹40 lakh or at nothing.

The judgement is about the *reversal*, not about the estimate. Including the
full estimate because it is the best estimate is exactly what the constraint
exists to prevent, because revenue reversed does far more damage than revenue
deferred.

`மாறும்_எதிர்பார்ப்பு` refuses probabilities that do not total 100 rather than
scaling them. Probabilities summing to 90 mean an outcome was forgotten, and
scaling the rest hides it.

### Allocation is on standalone selling prices, and it must add back

The transaction price is split in proportion to what each good or service would
sell for **on its own** — not to the prices written in the contract. A contract
bundling a machine at full price with three years of "free" service has sold
three years of service, and part of the machine's price belongs to it.

The split goes through `kAcu.qmz` in paise, so the allocated amounts sum to the
transaction price **exactly**. ₹10,000 across three equal obligations is
3,333.33 / 3,333.33 / 3,333.34, and the tests assert the total. Rounding each
share independently leaves paise that have to go somewhere, and wherever they
go is arbitrary.

A discount is allocated proportionately to all obligations unless there is
observable evidence it belongs to only some. Proportionate is the default and
the exception must be evidenced — putting a discount where it is convenient
moves revenue between periods without changing a single real term.

### Over time needs one of three criteria, and the third is two conditions

The third criterion is an asset with **no alternative use** *and* an
**enforceable right to payment for performance to date**. It is routinely
applied as one condition. A custom-built asset has no alternative use, and that
alone is not enough — without the right to payment, revenue waits until
delivery.

A great many construction contracts turn on this single point, and the answer
is in the termination clause rather than in the specification.

### Uninstalled materials

A lift delivered to site and not installed is a cost incurred that represents
no progress. Plain cost-to-cost advances the percentage and drags a full margin
with it — the entity books profit on a machine it bought from somebody else and
merely took delivery of.

The standard's answer: exclude the cost from the measure of progress, and
recognise revenue on it **at cost**. In the tests, a ₹12,000 contract with
₹4,500 of ₹9,000 costs incurred — ₹3,000 of it an uninstalled machine — reads
₹6,000 of revenue unadjusted and **₹5,250** adjusted: ₹2,250 with a margin plus
₹3,000 at cost. The ₹750 difference is margin on somebody else's work.

### A change in estimate is a cumulative catch-up, and it can be negative

Revenue for a period is revenue to date less revenue already recognised. When
an estimate of total cost is revised upward, progress falls, revenue to date
falls below what has been recognised, and **this period's revenue is negative**.
Earlier periods are not restated — the estimate was not wrong when it was made.

The tests follow one contract through it: ₹6,000 recognised in the first
period, then a re-estimate of total cost from ₹9,000 to ₹20,000 produces
**−₹3,000** in the second and an expected margin of −₹8,000.

Clamping that at zero would carry the overstatement forward silently, which is
the one thing the cumulative method exists to prevent.

### Contract assets and liabilities net per contract, never across

Revenue ahead of billing is a contract asset; billing ahead of revenue is a
contract liability. The netting is **within a contract**, and contracts are
aggregated **gross**.

A builder with fifty contracts, half ahead and half behind, has two large
balances and not one small one. `தவறாக_நிகரமிட்டால்` exists to price the
difference — in the tests, portfolio netting would hide ₹3,000 from each side
of the balance sheet at once.

A **receivable** is an unconditional right where only time remains. A
**contract asset** is conditional on something else, usually on doing more of
the work. The distinction is not presentational: a contract asset carries the
risk that the rest of the work is not completed, and is impaired differently.
Whether an invoice has been raised is irrelevant to it.

## Onerous contracts are not in Ind AS 115

They are in Ind AS 37, deliberately — the revenue standard was written without
an onerous-contract model. This is missed constantly, and it matters because
the two standards behave in opposite directions. Ind AS 115 spreads cost
forward over the contract; Ind AS 37 takes the **whole expected loss now**.

A contract heading for a loss does not get a reduced margin each period. It
gets a provision for the entire loss the moment the loss becomes probable.

The order is fixed and gets skipped: **impair the assets dedicated to the
contract first, then provide for what remains.** Recognising a provision while
capitalised costs still sit on the balance sheet counts the same loss twice. In
the tests, a ₹6,000 loss against a ₹2,000 contract asset is ₹2,000 of
impairment and ₹4,000 of provision, and the two are asserted to sum to the
loss.

The unavoidable cost is the **lower** of fulfilling and the penalty for
exiting — nobody is worse off than walking away. The cost of fulfilling is the
*directly related* cost, not incremental only; the 2022 amendment to Ind AS 37
settled that, and the narrower reading made fewer contracts onerous.

## Where it stops

- **No journal entries.** Nothing here posts to `kaNakkiyal/pErEtu.qmz`. The
  amounts are computed; turning them into debits and credits needs a mapping
  from contract to account that belongs to an application.
- **No principal-versus-agent.** Whether to report gross or net turns on who
  controls the good before transfer, and it changes revenue by the whole cost
  of sales without changing profit by a rupee. It deserves its own treatment.
- **No licences, no warranties as a category.** A licence of intellectual
  property has its own right-to-use versus right-to-access test; a warranty is
  a performance obligation only if it provides a service beyond assurance.
  Both are classification questions rather than arithmetic.
- **No modifications.** A contract modification is either a separate contract,
  a termination and replacement, or a cumulative catch-up, depending on whether
  the added goods are distinct and priced at standalone value. The three give
  materially different numbers and the test is a judgement.
- **No financing component measurement.** `பரிமாற்ற_மதிப்பு` takes the
  financing component as an amount; computing it needs a discount rate and the
  timing of payments.

## What it builds on

`kAcu.qmz` guarantees the allocation adds back. `nAtkAtti/` supplies the
calendar a payment schedule needs. `qittam/Ittu_maqippu.qmz` measures progress
for **control** purposes and this measures it for **recognition** purposes —
the same cost-to-cost arithmetic serving two different questions, which is why
`வருவாய்_நிலவரம்` here and `நிலவரம்` there are deliberately separate functions.
