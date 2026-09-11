# itar — இடர் (risk, and estimating under it)

`qittam/kattuppAtu.qmz` takes a contingency reserve as a number somebody passes
in, and `qittam/pAqY.qmz` computes a critical path from single-point durations.
This directory supplies the first and puts a spread around the second.

A single-point duration is a guess wearing a suit. Three-point estimation asks
for the optimistic, most likely and pessimistic cases and gets from them both
an expected duration and a **measure of how uncertain it is** — which is the
part a single number cannot carry, and the part that decides what can actually
be committed to.

## Modules

| File | Contents |
|---|---|
| `itar.qmz` | the register — `இடர்_ஆக்கு` `இடர்_செல்லுபடியா` `எதிர்பார்ப்பு` `மொத்த_எதிர்பார்ப்பு` `வகையால்_எதிர்பார்ப்பு` `இடர்_மதிப்பெண்` `மதிப்பெண்_நிலைமை` `பதிலளிப்பு_சரியா` `எஞ்சிய_இடர்` `பதிலளிப்பு_பயனுள்ளதா` `தற்செயல்_இருப்பைக்_கணக்கிடு` `மேலாண்மை_இருப்பைக்_கணக்கிடு` `கிளை_ஆக்கு` `கிளைகளின்_எதிர்பார்ப்பு` `சிறந்த_முடிவு` |
| `mUZRuppuLLi.qmz` | three-point estimation — `முப்புள்ளி_ஆக்கு` `பீட்டா_சராசரி` `முக்கோண_சராசரி` `நியம_விலகல்` `பரவல்` `பாதை_சராசரி` `பாதை_பரவல்` `பாதை_விலகல்` `தவறாக_விலகல்களைக்_கூட்டினால்` `நிகழ்தகவு_Z` `நம்பிக்கைக்கான_Z` `இலக்கு_நிகழ்தகவு` `நம்பிக்கை_காலம்` |
| `itar_cOqaZY.qmz` | the tests — 83 assertions |

```bash
etamil --vm nUlakam/itar/itar_cOqaZY.qmz
```

`வர்க்கமூலம்` was added to `kaNiqam.qmz` for this work. The host has no square
root, no power and no exponential, so it is Newton's method — general
arithmetic, and it belongs in the maths module rather than hidden in a risk
one, for the same reason `மீதி` is written there.

## Seven decisions worth knowing

### An opportunity is a risk

The register holds both, an opportunity's expected value is positive, and a
register of nothing but threats is one somebody has half filled in. Every
function is signed for that reason.

### Impact is a magnitude; the kind carries the direction

`இடர்_செல்லுபடியா` refuses a negative impact. A threat entered with a negative
impact would be *added* to the reserve rather than subtracted from it, and the
register would still total something plausible.

### The expected value is an average this project will not experience

A risk with a 20% chance of costing ₹10 lakh has an expected value of ₹2 lakh,
and the outcome is either ₹10 lakh or nothing — never ₹2 lakh. That is what a
reserve is for, and it is why a reserve sized from expected value is right for
a portfolio and merely reasonable for one project.

### A probability-and-impact score is not a quantity

`இடர்_மதிப்பெண்(5, 1)` and `இடர்_மதிப்பெண்(1, 5)` both score **5**. One
happens constantly and costs nothing; the other would end the project and
almost certainly will not. Ranking by the product puts them side by side.

It is kept because it is what a risk workshop can produce in an afternoon, and
ordering risks roughly beats not ordering them. The components come back
alongside the score so a caller can break the tie the score cannot, and
`எதிர்பார்ப்பு` is the answer wherever real money is available.

### A response must cost less than the exposure it removes

Halving the probability of a ₹5 lakh risk from 30% to 10% is worth ₹1 lakh.
Spending ₹1.5 lakh on it makes the project worse by ₹50,000 — and it is done
constantly, because the spending is visible, deliberate and feels responsible
while the exposure is a probability nobody experiences.

The comparison is against the **reduction in exposure**, not against the
impact. A 5% chance of costing a crore is worth ₹5 lakh of prevention, not a
crore of it.

Responses are also typed. The four to a threat are avoid, transfer, mitigate,
accept; the four to an opportunity are exploit, share, enhance, accept. Only
acceptance is the same word for both, and "mitigating an opportunity" is a
contradiction rather than a clumsy phrase.

### Contingency comes from the register; management reserve cannot

`தற்செயல்_இருப்பைக்_கணக்கிடு` is the expected value of the **threats** — the
number `qittam/kattuppAtu.qmz` wants for its `தற்செயல்_இருப்பு`. Opportunities
are deliberately excluded: netting one against a threat funds a cost that will
certainly arise if the threat occurs with a benefit that may not, and the two
are different events that will not oblige by happening together.

The management reserve covers what nobody identified, so a figure derived from
identified risks is exactly as complete as the risk workshop was — which is the
one thing it must not depend on. It is a policy percentage, and
`மேலாண்மை_இருப்பைக்_கணக்கிடு` takes that percentage rather than inventing one.

### Variances add along a path; standard deviations do not

The spread of a sequence is the root of the sum of the variances. In the tests,
a three-activity path has a variance of exactly 5 and a deviation of **2.24**;
adding the deviations gives **3.67**.

The wrong answer is always larger — a schedule made to look less certain than
it is, which nobody queries. `தவறாக_விலகல்களைக்_கூட்டினால்` exists to price the
difference, not as an alternative.

## The most useful thing here

**Committing to the expected duration is committing to a coin toss.**
`இலக்கு_நிகழ்தகவு` at the mean answers 50%, every time, and it surprises people
every time. `நம்பிக்கை_காலம்(90, …)` gives the duration that carries 90%
confidence — in the tests, 15.87 against an expected 13 — and the gap is
schedule contingency, the same idea as a contingency reserve applied to time
instead of money.

## The normal curve is a table

The host has no exponential, so Φ is tabulated from 0 to 3.5 at intervals of a
tenth with linear interpolation between points. That is accurate to about four
decimal places.

Stating the method matters more than the accuracy. A confidence figure carried
to two decimals looks like a measurement, and it rests on a beta assumption, a
six-sigma range assumption and a normal approximation — any of which is a
larger source of error than this table.

## Where it stops

- **No Monte Carlo.** It needs a random source the standard library does not
  have. Everything here is closed-form.
- **Nothing picks the riskiest path.** The critical path is not always the
  riskiest — a path a day shorter with twice the variance will overrun it often.
  `பாதை_பரவல்` lets every path be measured; comparing them is the caller's, and
  is the only way that comparison ever gets made.
- **No correlation between risks.** Variances add only for independent
  activities. Two delays caused by the same supplier are not independent, and
  adding their variances understates the joint spread. Modelling that needs a
  covariance the register does not carry.
- **No risk over time.** A risk's probability usually falls as a project
  proceeds and its impact often rises. The register is a snapshot.
- **No qualitative-to-quantitative bridge.** A score of 20 does not convert to
  a rupee figure, and any function offering to do so would be inventing one.
