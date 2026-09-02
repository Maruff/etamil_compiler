"""Exact decimals — the arithmetic eTamil guarantees."""
import sys
from decimal import Decimal

n = int(sys.argv[1])
RATE = Decimal("0.05")
BASE = Decimal(300000)

total = Decimal(0)
i = 0
while i < n:
    income = BASE + i
    tax = (income - BASE) * RATE
    total = total + tax
    i += 1

print(total)
