#!/usr/bin/env python3
"""Plugin de exemplo (transform): prefixa o input do utilizador.

Lê `{id, event, payload}` (JSON) do stdin e responde `{verdict}` no stdout.
"""
import json
import sys

request = json.loads(sys.stdin.readline())
payload = request.get("payload", {})
if payload.get("kind") == "user_input":
    text = payload.get("text", "")
    print(json.dumps({"verdict": "replace", "text": f"[exemplo] {text}"}))
else:
    print(json.dumps({"verdict": "pass"}))
