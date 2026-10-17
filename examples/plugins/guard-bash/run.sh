#!/usr/bin/env python3
"""Plugin de exemplo (gate): nega `bash` destrutivo.

Lê `{id, event, payload}` (JSON) do stdin e responde `{verdict}` no stdout.
"""
import json
import sys

request = json.loads(sys.stdin.readline())
payload = request.get("payload", {})
if payload.get("kind") != "action_before" or payload.get("tool") != "bash":
    print(json.dumps({"verdict": "pass"}))
    sys.exit(0)

arguments = payload.get("arguments", "")
patterns = ("rm -rf", "mkfs", "push --force", "> /dev/")
if any(pattern in arguments for pattern in patterns):
    print(json.dumps({"verdict": "deny", "reason": "bash_destrutivo"}))
else:
    print(json.dumps({"verdict": "pass"}))
