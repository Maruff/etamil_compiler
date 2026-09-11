# examples/qittam — one project, end to end

```bash
etamil --vm examples/qittam/cawkamam.qmz
```

`cawkamam.qmz` runs a single ERP implementation — திட்டம் சங்கமம் — through
every part of nUlakam that touches a project. Each library has its own tests;
this exists to show that they **compose**, and that one set of facts reaches
control, the ledger and the financial statements without being re-keyed.

| Step | Library |
|---|---|
| the work breakdown, and the 100% rule | `qittam/pakuppu` |
| three-point estimates and a 90% date | `itar/mUZRuppuLLi` |
| the critical path and float | `qittam/pAqY` |
| duration → real dates | `nAtkAtti/vElYnAL` |
| the risk register → contingency | `itar/itar` |
| the budget and its two reserves | `qittam/kattuppAtu` |
| timesheets, costed at the rate in force | `vaLam/vaLam`, `vaLam/nEraqqAL` |
| posted, and actual cost read back | `vaLam/paqivu` |
| earned value | `qittam/Ittu_maqippu` |
| revenue under Ind AS 115 | `varuvAy/muZZERRam` |
| the contract's position, and its loss | `varuvAy/iruppukaL`, `varuvAy/oppanqac_celavu` |

Thirteen modules across five directories, in one program — and further
still through what they import: `paqivu.qmz` reaches `kaNakkiyal` for the
ledger, and almost everything reaches `kaNiqam` and `poruL`.

## What the run shows

**Committing to the expected duration is a coin toss.** The path's expected
duration is 40 days and the probability of meeting it is 50%. The date that
carries 90% confidence is 43.14 days, and the 3.14 between them is schedule
contingency.

**Forty working days is fifty-seven elapsed days.** The difference is weekends
and two holidays, and it is what a Gantt chart shows and a duration does not.

**Actual cost comes out of the ledger.** ₹4,00,000, read back from the general
ledger per control account — not from a spreadsheet. The control against the
timesheets reports a ₹16,000 difference, which is the licence materials: real,
expected, and exactly what that reconciliation is for.

**The two readings of the same project disagree in direction.** Spend of
₹4,00,000 against ₹4,20,000 planned reads **+₹20,000 favourable** on a
conventional report. The cost variance is **−₹1,00,000** and the schedule
variance **−₹1,20,000**. CPI is 0.75.

**Control and recognition report the same percentage — and that is a theorem,
not a coincidence.** When the estimate at completion is derived as BAC ÷ CPI,
it equals BAC × AC ÷ EV, so cost-to-cost progress AC ÷ EAC reduces to EV ÷ BAC
exactly. Both read 30%. They part company only when the estimate at completion
is rebuilt from the bottom up instead of scaled by CPI — which is what a
project in trouble eventually does, and then the difference has to be explained
rather than reconciled away.

**A CPI of 0.75 is not a ratio; it is a provision.** It implies an estimate at
completion of ₹13,33,333 against a contract price of ₹12,50,000, so the
contract is onerous. Ind AS 37 wants the whole remaining loss of ₹58,333 today
— absorbed here entirely by impairing the contract asset, because the standard
requires the dedicated asset to be written down before any provision is raised.

Meanwhile Ind AS 115 goes on recognising revenue as the work is done. Both are
right, and a business that watches only one of them is surprised twice.

## What it deliberately does not do

No journal entry is posted for revenue, the provision or the impairment — the
amounts are computed and the mapping from contract to account belongs to an
application. No tax is computed on the result. Both would make the example
longer without making the point better.
