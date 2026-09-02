# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
"""A very small Redis, for testing without one installed.

Speaks RESP and implements just enough to exercise a client: PING, SET with
EX, GET, DEL, EXISTS, INCR, LPUSH, LRANGE, and an unknown command so the error
path can be checked. Not a cache — a conversation partner.

    python retis_pOli.py 6399
"""
import socket
import sys
import threading

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 6399
STORE = {}
LISTS = {}


def read_command(f):
    """One RESP array of bulk strings, or None at end of stream."""
    line = f.readline()
    if not line:
        return None
    if not line.startswith(b"*"):
        return []
    count = int(line[1:].strip())
    parts = []
    for _ in range(count):
        header = f.readline()
        length = int(header[1:].strip())
        data = f.read(length)
        f.read(2)  # the CRLF after the payload
        parts.append(data.decode("utf-8", "replace"))
    return parts


def bulk(value):
    if value is None:
        return b"$-1\r\n"
    raw = value.encode()
    return b"$%d\r\n%s\r\n" % (len(raw), raw)


def handle(conn):
    f = conn.makefile("rwb")
    while True:
        parts = read_command(f)
        if parts is None:
            break
        if not parts:
            continue
        name = parts[0].upper()
        args = parts[1:]

        if name == "PING":
            reply = b"+PONG\r\n"
        elif name == "SET":
            STORE[args[0]] = args[1]
            reply = b"+OK\r\n"
        elif name == "GET":
            reply = bulk(STORE.get(args[0]))
        elif name == "DEL":
            gone = sum(1 for k in args if STORE.pop(k, None) is not None)
            reply = b":%d\r\n" % gone
        elif name == "EXISTS":
            reply = b":%d\r\n" % sum(1 for k in args if k in STORE)
        elif name == "INCR":
            STORE[args[0]] = str(int(STORE.get(args[0], "0")) + 1)
            reply = b":%s\r\n" % STORE[args[0]].encode()
        elif name == "LPUSH":
            LISTS.setdefault(args[0], [])
            for item in args[1:]:
                LISTS[args[0]].insert(0, item)
            reply = b":%d\r\n" % len(LISTS[args[0]])
        elif name == "LRANGE":
            items = LISTS.get(args[0], [])
            start, stop = int(args[1]), int(args[2])
            chosen = items[start:] if stop == -1 else items[start:stop + 1]
            reply = b"*%d\r\n" % len(chosen) + b"".join(bulk(i) for i in chosen)
        else:
            reply = b"-ERR unknown command '%s'\r\n" % name.encode()

        f.write(reply)
        f.flush()
    conn.close()


server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
server.bind(("127.0.0.1", PORT))
server.listen(8)
print(f"mock Redis on {PORT}", flush=True)
while True:
    client, _ = server.accept()
    threading.Thread(target=handle, args=(client,), daemon=True).start()
