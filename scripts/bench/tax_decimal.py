"""Same computation, exact decimals — the apples-to-apples comparison."""
from decimal import Decimal

RATE = Decimal("0.05")
BASE = Decimal(300000)

total = Decimal(0)
i = 0
while i < 100000:
    income = BASE + i
    tax = (income - BASE) * RATE
    total = total + tax
    i += 1

print(total)
