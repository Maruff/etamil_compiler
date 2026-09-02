/* Binary float — the fast, inexact half of the table. N from argv, for the
 * same constant-folding reason as tax_int.c. */
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv) {
    if (argc < 2) { fprintf(stderr, "need N\n"); return 2; }
    long long n = atoll(argv[1]);

    double total = 0.0;
    for (long long i = 0; i < n; i++) {
        double income = 300000.0 + (double)i;
        double tax = (income - 300000.0) * 0.05;
        total += tax;
    }
    printf("%.2f\n", total);
    return 0;
}
