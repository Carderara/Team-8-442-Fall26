#!/usr/bin/env python3
"""
Stand-in for the challenge chat server, used to test the receiver without
being on the lab network.

Behaves the way lets_chat.pdf describes the real server: it sends a prompt,
waits for a team name and a newline, then streams an overt message one
character at a time. The pause *after* each character carries one bit of the
covert message, so the gap the receiver measures between characters i and
i+1 is bit i. Both messages end with "EOF".

Usage:
    python3 mock_server.py                 # listen on 127.0.0.1:12321
    python3 mock_server.py --port 9999
    python3 mock_server.py --covert "SOME SECRET"
"""

import argparse
import socket
import time

OVERT = (
    "Transmission from relay beacon. All stations hold position and "
    "maintain radio discipline until the next scheduled window. "
    "Acknowledge on receipt. EOF"
)
# The covert payload the real server carries is Vigenere-ciphertext; this
# stand-in uses the same shape so the decode-then-decrypt path gets exercised.
COVERT = "Gur frperg vf va gur ynfg gevc EOF"

SHORT_MS = 20.0
LONG_MS = 60.0


def bits_of(text):
    return "".join(format(ord(c), "08b") for c in text)


def serve(host, port, overt, covert, short_ms, long_ms):
    bits = bits_of(covert)
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    listener.bind((host, port))
    listener.listen(1)
    print(f"listening on {host}:{port}")
    print(f"overt: {len(overt)} chars, covert: {len(bits)} bits "
          f"(needs {len(bits) + 1} chars to carry)")
    if len(bits) + 1 > len(overt):
        print("WARNING: the overt message is too short to carry every bit")

    while True:
        conn, peer = listener.accept()
        conn.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        print(f"\nconnection from {peer}")
        try:
            conn.sendall(b"Identify yourself: ")
            team = conn.recv(256).decode("utf-8", "replace").strip()
            print(f"team name: {team!r}")

            # Character i is followed by the pause that encodes bit i, so the
            # last character needs no pause after it.
            for i, ch in enumerate(overt):
                conn.sendall(ch.encode())
                if i >= len(overt) - 1:
                    break
                bit = bits[i] if i < len(bits) else "0"
                time.sleep((long_ms if bit == "1" else short_ms) / 1000.0)
            print("transmission complete")
        except OSError as exc:
            print(f"connection error: {exc}")
        finally:
            conn.close()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=12321)
    parser.add_argument("--overt", default=OVERT)
    parser.add_argument("--covert", default=COVERT)
    parser.add_argument("--short", type=float, default=SHORT_MS)
    parser.add_argument("--long", type=float, default=LONG_MS)
    args = parser.parse_args()
    serve(args.host, args.port, args.overt, args.covert, args.short, args.long)


if __name__ == "__main__":
    main()
