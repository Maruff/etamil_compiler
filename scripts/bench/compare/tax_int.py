"""Exact, using int scaled to paisa. What you write when Decimal is too slow."""
import sys

n = int(sys.argv[1])
total = 0          # paisa
i = 0
while i < n:
    income = 30000000 + i * 100      # paisa
    total += (income - 30000000) * 5 // 100
    i += 1

print(f"{total // 100}.{total % 100:02d}".rstrip("0").rstrip("."))
