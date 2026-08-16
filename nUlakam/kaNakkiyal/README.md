# kaNakkiyal — கணக்கியல், an accounting framework

Double-entry accounting with IFRS-style presentation, GST, and financial
statements. Written in eTamil, modelled on
[ekmungai/eloquent-ifrs](https://github.com/ekmungai/eloquent-ifrs).

```etamil
இறக்கு "nUlakam/kaNakkiyal/kaNakkukaL.qmz";
இறக்கு "nUlakam/kaNakkiyal/pErEtu.qmz";
இறக்கு "nUlakam/kaNakkiyal/aRikkYkaL.qmz";

கணக்குகள் = [
    மதிப்பு(கணக்கு_ஆக்கு("1000", "வங்கி",   வகை_சொத்து(), "நடப்பு")),
    மதிப்பு(கணக்கு_ஆக்கு("3000", "மூலதனம்", வகை_பங்கு(),  "பங்கு"))
];

பேரேடு = [];
பேரேடு = மதிப்பு(பதிவிடு(பேரேடு, பரிவர்த்தனை_ஆக்கு(
    "JV001", "2026-04-01", "தொடக்க மூலதனம்", [
        பற்று_வரிசை("1000", 500000),
        வரவு_வரிசை("3000", 500000)
    ])));

இருப்பாய்வு_அச்சிடு(இருப்பாய்வு(பேரேடு, கணக்குகள்));
```

## Design

**The ledger is a value.** Posting returns a *new* ledger rather than editing
the old one, so an entry cannot be changed after the fact — the property
eloquent-ifrs protects by keeping clients away from the table.

**Double entry is enforced.** `பதிவிடு` refuses a transaction whose sides
disagree and returns `தவறு`; nothing unbalanced reaches the ledger.

**Balances read in the account's own direction.** A liability with more
credits than debits is positive, just as an asset with more debits than
credits is. Every report depends on `பற்று_இயல்பா`.

**Reports derive from the ledger.** Nothing is stored twice, so a statement
cannot disagree with the postings behind it.

## Modules

| File | Contents |
|---|---|
| `kaNakkukaL.qmz` | Chart of accounts, the five IFRS roots, normal sides |
| `kAlam.qmz` | Reporting periods, Indian and calendar years |
| `pErEtu.qmz` | Transactions, line items, posting, balances, period filters |
| `vari.qmz` | GST and every transaction type |
| `oqukkItu.qmz` | Assignment, outstanding amounts, ageing |
| `aRikkYkaL.qmz` | Trial balance, income statement, balance sheet, account statement, cash flow |
| `niRuvaZam.qmz` | Entities, currencies, exchange differences |
| `mutippu.qmz` | Year-end close |

## Account types

`வகை_சொத்து()` `வகை_பொறுப்பு()` `வகை_பங்கு()` `வகை_வருவாய்()` `வகை_செலவு()`

Assets and expenses increase on the debit side; the rest on the credit side.

## Transaction types

| Function | eloquent-ifrs |
|---|---|
| `பரிவர்த்தனை_ஆக்கு` | JournalEntry |
| `விற்பனை_பரிவர்த்தனை` | ClientInvoice |
| `கொள்முதல்_பரிவர்த்தனை` | SupplierBill |
| `ரொக்க_விற்பனை` | CashSale |
| `ரொக்க_கொள்முதல்` | CashPurchase |
| `பணம்_பெறு` | ClientReceipt |
| `பணம்_செலுத்து` | SupplierPayment |
| `வரவு_குறிப்பு` | CreditNote |
| `பற்று_குறிப்பு` | DebitNote |
| `எதிர்_பதிவு` | ContraEntry |
| `தொடக்க_இருப்பு` | opening balance |
| `அன்னிய_வேறுபாடு` | forex difference |

## Reports

| Function | Covers |
|---|---|
| `இருப்பாய்வு` / `கால_இருப்பாய்வு` | Trial balance |
| `வருமான_அறிக்கை` / `கால_வருமான_அறிக்கை` | Income statement — **for a period** |
| `இருப்புநிலை` / `நாள்_இருப்புநிலை` | Balance sheet — **as at a date** |
| `கணக்கு_அறிக்கை` | Account statement with running balance |
| `பணப்புழக்க_அறிக்கை` | Cash flow, direct method |
| `வயது_அட்டவணை` | Ageing schedule |

The period distinction is not cosmetic. An income statement covers a span; a
balance sheet accumulates to a date. The plain forms run over the whole
ledger, which is only correct for a first year with no close.

## Clearing

```etamil
ஒதுக்கீடுகள் = மதிப்பு(ஒதுக்கு(ஒதுக்கீடுகள், பேரேடு,
    "RCT001", "INV001", 118000, "1100", "சொத்து"));

மீதம் = நிலுவைத்_தொகை(பேரேடு, ஒதுக்கீடுகள், "INV001", "1100", "சொத்து");
```

Refuses clearing more than the invoice is worth, or spreading a receipt
further than the receipt itself. Both are accounting errors, not programming
ones.

## Naming

Field names are compound (`கணக்கு_குறி`, `பற்று_தொகை`) rather than bare
words, because **a keyword used as a field name is stored under its token
name** — `{வரி: 1}` becomes the field `Tax`. See
[ARCHITECTURE.md](../../docs/ARCHITECTURE.md#the-known-wart-keyword-names-are-translated).

Follow the same convention when extending this framework.

## Examples

- `examples/finance/kaNakkiyal.qmz` — a full cycle with GST and the three statements
- `examples/finance/niluvY_vayaqu.qmz` — clearing, ageing, periods, account statement

## Not covered

Budgets, fixed-asset depreciation schedules, inventory valuation,
consolidation eliminations between entities, and tax return generation.
Entities partition one ledger but do not yet eliminate intercompany balances.
