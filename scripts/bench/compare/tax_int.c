/* Exact, using int64 scaled to paisa — what a C program does when it has to be
 * right about money and the language has no decimal type.
 *
 * N comes from argv. At /O2 a literal bound lets the compiler evaluate the
 * whole loop at compile time and print a constant, which turns the benchmark
 * into a measurement of process startup and nothing else. */
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv) {
    if (argc < 2) { fprintf(stderr, "need N\n"); return 2; }
    long long n = atoll(argv[1]);

    long long total = 0;                          /* paisa */
    for (long long i = 0; i < n; i++) {
        long long income = 30000000 + i * 100;    /* paisa */
        total += ((income - 30000000) * 5) / 100;
    }

    long long whole = total / 100, frac = total % 100;
    if (frac == 0) printf("%lld\n", whole);
    else           printf("%lld.%02lld\n", whole, frac);
    return 0;
}
