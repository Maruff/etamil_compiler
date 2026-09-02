# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
"""A stand-in for a Fabric REST gateway.

Answers the way one does: a query returns state, an invoke writes it, a
chaincode refusal comes back as a 500 with the reason, and a write to a
contended key fails the first time with MVCC_READ_CONFLICT — which is what a
real network does when two transactions read and write the same key, and the
reason the retry in fabric.qmz exists.
"""
import http.server
import json
import sys

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8977

STATE = {"asset1": {"ID": "asset1", "Owner": "ராஜா", "Value": 300}}
# The first write to this key conflicts, the second succeeds.
CONTENDED = {"asset_busy": 1}


class Handler(http.server.BaseHTTPRequestHandler):
    def _send(self, code, payload):
        body = json.dumps(payload).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0))
        request = json.loads(self.rfile.read(length) or b"{}")
        function = request.get("function", "")
        args = request.get("args", [])

        if self.path.endswith("/query"):
            if function == "ReadAsset":
                key = args[0] if args else ""
                if key in STATE:
                    return self._send(200, STATE[key])
                # A chaincode saying no is a 500 with a reason, not a 404.
                return self._send(500, {"message": f"the asset {key} does not exist"})
            return self._send(500, {"message": f"no such function: {function}"})

        if self.path.endswith("/invoke"):
            if function == "CreateAsset":
                key = args[0]
                if key in CONTENDED and CONTENDED[key] > 0:
                    CONTENDED[key] -= 1
                    return self._send(
                        500,
                        {"message": "transaction invalidated with status "
                                    "MVCC_READ_CONFLICT"},
                    )
                STATE[key] = {"ID": key, "Owner": args[1], "Value": int(args[2])}
                return self._send(200, {"txid": "mock-" + key, "status": "VALID"})
            if function == "RejectMe":
                return self._send(
                    500, {"message": "chaincode response 400, caller is not authorised"}
                )
            return self._send(500, {"message": f"no such function: {function}"})

        self._send(404, {"message": "no such path"})

    def log_message(self, *_args):
        pass


print(f"mock Fabric gateway on {PORT}", flush=True)
http.server.HTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
