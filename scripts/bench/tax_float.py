"""Same computation with binary floats — fast, and wrong at the paisa."""
total = 0.0
i = 0
while i < 100000:
    income = 300000 + i
    tax = (income - 300000) * 0.05
    total = total + tax
    i += 1
print(total)
