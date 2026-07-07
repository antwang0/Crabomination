#!/usr/bin/env python3
"""Dev server for the browser build: serves web/dist with the headers wasm
wants (correct MIME type, no stale caching of the bundle). Usage:

    python3 web/serve.py [port]     # default 8000

Then open http://localhost:8000 — and run the game server with its
WebSocket listener enabled (default CRAB_WS_BIND=0.0.0.0:7778).
"""
import http.server
import os
import sys

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8000
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "dist")


class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {
        **http.server.SimpleHTTPRequestHandler.extensions_map,
        ".wasm": "application/wasm",
        ".js": "text/javascript",
    }

    def end_headers(self):
        # The bundle changes on every rebuild; card art is immutable.
        if self.path.endswith((".wasm", ".js", ".html")):
            self.send_header("Cache-Control", "no-cache")
        super().end_headers()


if __name__ == "__main__":
    if not os.path.isdir(ROOT):
        sys.exit("web/dist not found — run web/build.sh first")
    os.chdir(ROOT)
    print(f"serving {ROOT} on http://localhost:{PORT}")
    http.server.ThreadingHTTPServer(("", PORT), Handler).serve_forever()
