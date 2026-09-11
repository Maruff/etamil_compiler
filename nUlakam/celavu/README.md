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
| `mILpakirvu.qmz` | reciprocal service apportionment — `சேவை_நிலையம்_ஆக்கு` `இரு_சேவை_ஒரேசமயம்` `மீண்டும்_பகிர்` `படிநிலைப்_பகிர்வு` `படிநிலை_வரிசை_வேறுபாடு` `உற்பத்திக்கு_மொத்தம்` |
| `ceyalmuRY.qmz` | process costing — `இயல்பு_இழப்பு_அலகுகள்` `அசாதாரண_வேறுபாடு` `அலகுக்கான_செலவு` `அசாதாரண_இழப்பின்_மதிப்பு` `அசாதாரண_ஆதாயத்தின்_மதிப்பு` `செயல்முறையைக்_கணக்கிடு` `சராசரி_சமமான_அலகுகள்` `fifo_சமமான_அலகுகள்` `முறைகளின்_வேறுபாடு` |
| `ceyalpAtu.qmz` | activity-based costing — `செயல்பாடு_ஆக்கு` `இயக்கி_விகிதம்` `தயாரிப்பின்_மேல்நிலை` `பாரம்பரிய_மேல்நிலை` `குறுக்கு_மானியம்` `பயனளிக்குமா` `மொத்தம்_மாறவில்லையா` |
| `celavu_cOqaZY.qmz` | the tests — 128 assertions, run it to check the arithmetic |

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

## Three answers to the reciprocal service problem, and only two agree

The maintenance department services the canteen and the canteen feeds the
maintenance staff, so each total depends on the other and there is no order in
which to do the arithmetic once.

**Simultaneous equations** solve it exactly. **Repeated distribution** hands
each balance out over and over until what is left is immaterial, and converges
to the same answer — the tests assert the two agree to the paisa, not merely
that they are close. **Step-down** takes the centres in some order and refuses
to send anything backwards; in the tests it charges P1 702 where the exact
answer is 765.

Step-down's real defect is not its size but its arbitrariness: **the answer
depends on the order**. Taking S1 first charges P1 702 and taking S2 first
charges 793 — both defensible, both reconciling to the same total, and nothing
in the accounts records which was used.

Repeated distribution must not round inside the iteration. Rounding each share
to the paisa on every round injects money — the first version distributed 1,350
and landed 1,350.06 on the production centres. The conservation assertion is
what found it, and the shares are now rounded once, where the answer leaves.

## Normal loss bears no cost; abnormal loss does

Expected loss is a cost of making the good units, so the good units absorb it —
achieved by leaving the normal loss **out of the denominator**, not by a
separate charge:

    (total cost − scrap value of normal loss) ÷ (input − normal loss units)

In the tests, 1,000 units costing 9,500 with 10% normal loss and 5 a unit of
scrap gives exactly **10** a unit. Dividing by good output instead — the
natural-looking mistake — gives **10.59**, burying the abnormal loss in every
surviving unit.

Abnormal loss is valued at that same rate and **written off**, so it never
touches product cost. Good output plus abnormal loss accounts for the whole
cost to be absorbed, and the tests assert the process account closes.

An abnormal gain is credited the same way, less the scrap the un-lost units
would have fetched — a gain of 20 units at 10 is worth 100, not 200, because
the process gave up 100 of scrap revenue to produce them. That second half is
routinely forgotten.

Weighted average and FIFO differ by exactly the opening stock's prior-period
work: 1,080 against 1,000 on the test figures, a gap of 80, which is
200 units × 40%. Nothing else. FIFO's unit cost is this period's performance
and can be compared with last period's; the weighted average's cannot, because
it contains last period's.

## Activity-based costing redistributes; it never reduces

Setting up a machine costs the same whether the run is ten units or ten
thousand. Absorbing that on a volume base charges it to whoever made the most
volume, which is precisely the wrong party.

The tests carry the canonical case: a high-volume simple product and a
low-volume complex one, 1,60,000 of overhead in two pools. The blanket rate
charges A 1,50,000 and B 10,000; the activities justify 32,000 and 1,28,000.
Per unit, A falls from **15 to 3.20** and B rises from **10 to 128**.

The shift is ±1,18,000 — **equal and opposite**, and the tests assert it nets
to zero. It is a transfer, not a saving, and the pools are fully assigned
either way.

It is also not automatically worth doing. `பயனளிக்குமா` answers no for a
product that consumes activities in proportion to the volume base, because
there the method arrives at the same answer at more expense.

## Where it stops

Deliberately not attempted, and each for a reason:

- **Joint and by-product costing.** Splitting a common cost at a split-off
  point needs a basis — sales value, physical units, net realisable value —
  and the choice changes every product's margin while changing nothing real.
  It belongs with `mILpakirvu.qmz`'s treatment of arbitrary bases.
- **Standard costing of a process.** `niyamam.qmz` computes variances and
  `ceyalmuRY.qmz` computes equivalent units; running the two together needs a
  standard stated per equivalent unit, which is a modelling decision rather
  than arithmetic.
- **Backflush costing.** It exists because recording every movement costs more
  than it is worth in a short-cycle plant, and the argument for it is about
  the cost of bookkeeping rather than about cost accounting.

Money that must add back exactly — apportionment, splitting a pool across
centres — is delegated to `kAcu.qmz`, which works in paise and guarantees the
parts sum to the whole. Naive division does not.
