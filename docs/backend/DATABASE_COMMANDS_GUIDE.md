# eTamil Database Commands

> **Status:** Database statements execute in the VM through feature-gated,
> parameterised drivers. SQLite is enabled by default; PostgreSQL and
> MySQL/MariaDB require `--features postgres` or `--features mysql`.
> MongoDB, Redis, and JSON document stores are not implemented.

## Architecture

Database statements compile into VM instructions and dispatch through the
`Database` trait in `etamil_compiler/src/db/mod.rs`. Each database type has one
blocking connection per VM. SQL values are passed in a separate parameter
array and bound by the driver; eTamil never interpolates them into SQL.

Supported backends:

- **SQLite**: enabled by default; use a file path or `:memory:`.
- **PostgreSQL**: enable with `--features postgres`.
- **MySQL / MariaDB**: enable with `--features mysql`; use a `mysql://` URL.

## Syntax

```etamil
தளம்_இணை மைசீகுல், "mysql://user:password@127.0.0.1:3306/kaNakku";
தளம்_செய் "CREATE TABLE students (id INT, name VARCHAR(64), score INT)", [];
தளம்_செய் "INSERT INTO students VALUES (?, ?, ?)", [1, "ராஜா", 95];
தளம்_வினா "SELECT * FROM students WHERE id = ?", [1], results;
அச்சு results;
தளம்_பிரி மைசீகுல்;
```

Use `சீகுலைட்` for SQLite, `போச்குரசீகுல்` for PostgreSQL, and `மைசீகுல்`
for MySQL or MariaDB. The SQL text is sent to the driver as written; values
in the parameter array are bound separately.

| Statement | Behavior |
|---|---|
| `தளம்_இணை <type>, <connection>` | Open a connection |
| `தளம்_செய் <sql>, <params>` | Execute SQL and discard affected-row count |
| `தளம்_வினா <sql>, <params>, <name>` | Bind returned rows as an array of records |
| `தளம்_பிரி <type>` | Close and forget the connection |

Parameters must be an array containing numbers, strings, booleans, or nil.
Arrays, records, and result values cannot be bound as SQL parameters.

## MySQL / MariaDB Connectivity Check

The complete live test is [mYcIkul_qaLam.qmz](../../examples/db_samples/mYcIkul_qaLam.qmz).
It checks connection, DDL, parameterised inserts and queries, exact
`DECIMAL` arithmetic, integer-key coercion, typed `VARCHAR` and `DECIMAL`
results, ISO date conversion, NULL, disconnect, and SQL-injection resistance.

```bash
cd etamil_compiler
cargo build --release --features mysql
cd ..
ETAMIL_TEST_MYSQL=1 ./scripts/run_examples.sh
```

The sample URL must match the local account. MariaDB uses the same
`mysql://` URL format. See [TESTING.md](../../TESTING.md#7c-mysql--mariadb-connectivity)
for package installation and account setup commands.

The example runner skips this external-service check unless
`ETAMIL_TEST_MYSQL=1` is set. A skipped check is not a pass.

## Tests Without A Server

The language tests use a stand-in `Database` implementation to verify VM
wiring, result conversion, parameter ordering, and injection-safe binding:

```bash
cd etamil_compiler
cargo test db
```

This does not prove that a server is reachable. Use the live sample above for
that check.

## Errors and Security

Errors are bilingual and include the failing operation. A build without the
requested driver reports the feature needed to enable it. Unsupported database
types fail loudly rather than being silently ignored.

- Keep connection URLs and passwords out of committed source when possible.
- Use the parameter array for every user-controlled value.
- Grant test accounts access only to the test database.
- Use server TLS and least-privilege accounts in production.

## Future Work

- Connection pooling for multiple simultaneous connections.
- Explicit transaction statements.
- Schema validation before execution.
- Backup, restore, and replication helpers.