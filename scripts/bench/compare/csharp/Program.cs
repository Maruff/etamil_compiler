// C# has a native 128-bit `decimal`, which makes it the closest mainstream
// comparison to what eTamil guarantees: exact base-10 arithmetic in the
// language itself rather than through a library.
//
//   bench decimal <N>
//   bench double  <N>
//   bench empty
using System;
using System.Globalization;

internal static class Program
{
    private static int Main(string[] args)
    {
        if (args.Length == 0)
        {
            Console.Error.WriteLine("need a mode");
            return 2;
        }

        if (args[0] == "empty")
        {
            Console.WriteLine(0);
            return 0;
        }

        if (args.Length < 2)
        {
            Console.Error.WriteLine("need N");
            return 2;
        }
        long n = long.Parse(args[1], CultureInfo.InvariantCulture);

        switch (args[0])
        {
            case "decimal":
            {
                decimal rate = 0.05m;
                decimal baseIncome = 300000m;
                decimal total = 0m;
                for (long i = 0; i < n; i++)
                {
                    decimal income = baseIncome + i;
                    decimal tax = (income - baseIncome) * rate;
                    total += tax;
                }
                // Trim trailing zeros so the digits match every other program.
                Console.WriteLine(total.ToString("0.##########", CultureInfo.InvariantCulture));
                return 0;
            }
            case "double":
            {
                double total = 0.0;
                for (long i = 0; i < n; i++)
                {
                    double income = 300000.0 + i;
                    double tax = (income - 300000.0) * 0.05;
                    total += tax;
                }
                Console.WriteLine(total.ToString("F2", CultureInfo.InvariantCulture));
                return 0;
            }
            default:
                Console.Error.WriteLine($"unknown mode {args[0]}");
                return 2;
        }
    }
}
