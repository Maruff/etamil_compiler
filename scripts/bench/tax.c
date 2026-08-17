#include <stdio.h>
int main(void) {
    double total = 0.0;
    for (int i = 0; i < 100000; i++) {
        double income = 300000.0 + i;
        double tax = (income - 300000.0) * 0.05;
        total += tax;
    }
    printf("%.2f\n", total);
    return 0;
}
