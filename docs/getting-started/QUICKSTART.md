# Quick Start

Assumes eTamil is installed — see the [Installation Guide](INSTALLATION.md).

## Hello world

```bash
echo 'அச்சு "வணக்கம் உலகம்!";' > hello.qmz
etamil --vm hello.qmz
```

On Windows, write the file as UTF-8:

```powershell
'அச்சு "வணக்கம் உலகம்!";' | Out-File hello.qmz -Encoding UTF8
etamil --vm hello.qmz
```

Output:

```
✓ Lexical analysis complete (3 tokens)
✓ Parsing complete (1 statements)

=== eTamil VM Executor ===

✓ Bytecode generated (3 instructions)
=== Execution Output ===

வணக்கம் உலகம்!

✓ Execution completed successfully
```

`அச்சு` (`accu`) prints. Statements end with `;`.

## Variables and input

```etamil
எண் வருவாய்;
அச்சு "Enter income: ";
உள்ளிடு வருவாய்;
அச்சு "You earn " & வருவாய்;
```

`எண்` declares a number, `உள்ளிடு` reads a line from standard input, and `&` joins strings.

Every keyword also has a romanized spelling, so this is the same program:

```etamil
eN varuvAy;
accu "Enter income: ";
uLLitu varuvAy;
accu "You earn " & varuvAy;
```

## Conditions

```etamil
(வருவாய் > 800000) எனில் {
    அச்சு "High Tax Bracket";
}
இன்றேல் {
    அச்சு "Low Tax Bracket";
}
```

Conditions go in parentheses *before* `எனில்` (if). `இன்றேல்` (else) is optional.

Conditions can be combined:

```etamil
(வருவாய் > 800000 மற்றும் வயது < 60) எனில் {
    அச்சு "Taxable";
}
```

## Loops

```etamil
எண் i = 0;
(i < 3) சுற்று {
    அச்சு i;
    i = i + 1;
}
```

## A complete program

```etamil
// Income tax calculator
எண் வருவாய்;
அச்சு "Enter income: ";
உள்ளிடு வருவாய்;
வரி = 20%;

(வருவாய் > 800000) எனில் {
    அச்சு "High Tax Bracket";
    அச்சு (வருவாய் - 800000) * வரி;
}
இன்றேல் {
    அச்சு "Low Tax Bracket (No Tax)";
}
```

```bash
echo "950000" | etamil --vm tax.qmz
```

A `20%` literal becomes `0.2` at lex time.

## Files

```etamil
கோப்பு_திற "output.txt", "write";
கோப்பு_எழுது "output.txt", "வணக்கம்";
கோப்பு_மூடு "output.txt";

கோப்பு_படி "output.txt", data;
அச்சு data;
```

Opening for `"write"` truncates the file; each write after that appends a line.

## An HTTP server

```bash
etamil --server --port 8080 examples/backend/hello_server.qmz
curl http://localhost:8080/
```

The server is minimal: it runs your whole program as the handler for every route, single-threaded. `--async` is currently an alias for `--server`. Routes as language statements are not implemented yet — see the [roadmap](../ROADMAP.md).

## Run the examples

```bash
etamil --vm examples/basic_samples/example.qmz
etamil --vm examples/io_samples/simple_fileio.qmz
```

The `examples/db_samples/` programs deliberately **fail** — database statements parse but are not executable yet.

## When something goes wrong

Errors report where they happened:

```
✗ Lexical analysis failed (1 error(s)):
  வரி 3, நெடுவரிசை 12: அறியப்படாத உள்ளீடு '@'  (line 3, column 12: unrecognized input '@')
```

Common causes:

- **Missing `;`** — every statement needs one.
- **`undefined variable`** — the name was never assigned. Declare it first (`எண் x;` sets it to 0).
- **`not implemented in the VM yet`** — a database or server statement; see the [roadmap](../ROADMAP.md).

---

Next: [Keyword Reference](../reference/KEYWORDS.md) · [Command Reference](../reference/COMMANDS.md) · [Roadmap](../ROADMAP.md)
