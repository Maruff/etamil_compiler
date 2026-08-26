# கடை — an online shop

An eCommerce backend in eTamil: a catalogue in SQLite, carts priced with
per-line GST, orders placed atomically against stock, and the same orders
posted into a double-entry ledger.

The point of the example is the last part. Most eCommerce stacks bolt
accounting on afterwards; here the ledger, GST and the three statements were
already in `nUlakam/kaNakkiyal/`, so an order becomes a journal entry rather
than a row to reconcile later.

## Run it

```bash
etamil --vm examples/katY/katY_kAttu.qmz
```

Creates `katY.db` in the working directory, seeds three products at three GST
rates, prices a cart, prints a GST invoice, places one order and refuses
another for want of stock, then posts everything to the ledger and prints the
trial balance, income statement and balance sheet.

Then serve it:

```bash
etamil --server --port 8090 examples/katY/katY_cEvY.qmz
```

| Route | What it does |
|---|---|
| `GET /carakku` | the catalogue, in-stock only |
| `GET /carakku/:kuRi` | one product, 404 if unknown |
| `POST /vaNti/kaNakku` | price a cart: base, CGST/SGST or IGST, total |
| `POST /paNam/vaLYkkokki` | the payment gateway's callback, signature checked |
| `GET /ANY/:kuRi` | an order and its lines |

```bash
curl -X POST -H 'Content-Type: application/json' \
  -d '{"uL_mAnilam": true, "varicYkaL": [{"kuRi":"P1","aLavu":2}]}' \
  http://127.0.0.1:8090/vaNti/kaNakku
```

## Three things worth reading the source for

**GST is computed per line, not on the total.** An order can mix rates — 5% on
a saree, 18% on a phone case — and taxing the sum at one rate would be wrong on
both. Each line's tax is split into CGST/SGST or IGST and the splits are
accumulated. `மாநில_பிரிப்பு` gives the odd paisa to the second half, so the two
always add back to the whole.

**Order placement is a transaction.** Stock coming down and the order going in
have to happen together, so `ஆணை_இடு` wraps them in `BEGIN` / `COMMIT` and
rolls back if any line is short. The demo places one order and has a second
refused, then checks that the refused one left the stock untouched and no order
behind.

**The webhook checks its signature.** Without that, `POST /paNam/vaLYkkokki` is
an unauthenticated endpoint that marks orders paid — anyone who learns the URL
gets the shop's stock for nothing. `கையொப்பம்_சரியா` compares in constant time,
which is why it is a host primitive and not `==` in eTamil. The reply says only
that the signature was wrong, never why.

```bash
BODY='{"ANY_kuRi":"ORD-001","status":"captured"}'
SIG=$(python3 -c "import hmac,hashlib,sys; print(hmac.new(b'wh_test_secret', sys.argv[1].encode(), hashlib.sha256).hexdigest())" "$BODY")
curl -X POST -H "X-eTamil-Signature: $SIG" -d "$BODY" \
  http://127.0.0.1:8090/paNam/vaLYkkokki
```

## What it is not

Prices come from the database, never from the request — a client that could
send its own price could send zero. But this is an MVP, and it is missing
things a shop needs: no login on the routes (the pieces exist —
`கடவுச்சொல்_மறை` and `சீட்டு_ஆக்கு`), no refunds, no shipping, and the
webhook secret is in the source rather than the environment.
