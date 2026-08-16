# Command Reference

```
etamil [OPTIONS] <FILE>
cat program.qmz | etamil [OPTIONS]
```

With no file argument, eTamil reads the program from standard input.

## Options

| Option | Description | Default |
|---|---|---|
| `--vm` | Run on the bytecode VM | this is the default |
| `--server` | Start the synchronous HTTP server | |
| `--async` | Currently an alias for `--server` | |
| `--llvm` | LLVM backend — requires a build with `--features llvm`, Linux/macOS only | |
| `--host <HOST>` | Server bind address | `127.0.0.1` |
| `--port <PORT>` | Server port | `8080` |
| `-h`, `--help` | Show usage | |
| `-V`, `--version` | Show the version | |

Unknown options are rejected with exit code 2.

## Execution modes

### VM (default)

```bash
etamil --vm program.qmz
etamil program.qmz            # same thing
```

Compiles to bytecode and interprets it. This is the mode everything is tested against.

### Synchronous HTTP server

```bash
etamil --server --port 8080 examples/backend/hello_server.qmz
```

Single-threaded, one connection at a time. It registers **your entire program** as the handler for `GET`, `POST`, `PUT` and `DELETE` on `/`, plus a `/health` endpoint that returns 200. Defining routes from eTamil source is not implemented — see the [roadmap](../ROADMAP.md).

### Asynchronous HTTP server

```bash
etamil --async --port 8080 program.qmz
```

Prints a warning and runs the synchronous server. The async modules exist in the source tree but are not compiled into the binary.

### LLVM backend

```bash
etamil --llvm program.qmz
```

Emits LLVM IR to `output.ll`. Requires a build with `--features llvm` and LLVM 18; unavailable on Windows. Without that feature the binary prints an explanatory error and exits 1.

## Input

```bash
etamil --vm program.qmz                  # from a file
echo "950000" | etamil --vm tax.qmz      # program from file, input from stdin
cat program.qmz | etamil --vm            # program from stdin
```

`உள்ளிடு` (`uLLitu`) reads one line from standard input, so piping a value in answers the first prompt.

Both `.etamil` and `.qmz` extensions work — the compiler does not inspect the extension.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | success |
| 1 | lexical, parse, or runtime error |
| 2 | unknown command-line option |

Lexical errors print every problem with its line and column before exiting. Parse errors currently panic without a position — see [roadmap](../ROADMAP.md) item 2.

## Environment variables

| Variable | Used by | Effect |
|---|---|---|
| `ETAMIL_JWT_SECRET` | the auth module | Signing secret for JWTs. If unset, a random secret is generated per process and a warning is printed, so tokens stop working after a restart. Set this in any deployment that issues tokens. |

## Examples

```bash
# Run the tax calculator with piped input
echo "950000" | etamil --vm examples/basic_samples/example.qmz

# File I/O sample
etamil --vm examples/io_samples/simple_fileio.qmz

# Serve on all interfaces
etamil --server --host 0.0.0.0 --port 3000 examples/backend/hello_server.qmz

# Check the version
etamil --version
```

---

See also: [Keyword Reference](KEYWORDS.md) · [Quick Start](../getting-started/QUICKSTART.md) · [Roadmap](../ROADMAP.md)
