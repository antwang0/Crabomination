#!/usr/bin/env python3
"""Round 72's untrained warm-start artifact: the pilot's file with
head_policy.{weight,bias} := head_win.{weight,bias}, bit for bit. Pure
Python over the safetensors layout (u64 header length, JSON header, data)."""
import json, struct, sys
src, dst = sys.argv[1], sys.argv[2]
raw = open(src, "rb").read()
n = struct.unpack("<Q", raw[:8])[0]
hdr = json.loads(raw[8:8 + n])
data = raw[8 + n:]
meta = hdr.pop("__metadata__", None)
assert "head_policy.weight" not in hdr, "already headed"
end = max(v["data_offsets"][1] for v in hdr.values())
extra = b""
for suffix in ("weight", "bias"):
    w = hdr[f"head_win.{suffix}"]
    a, b = w["data_offsets"]
    hdr[f"head_policy.{suffix}"] = {"dtype": w["dtype"], "shape": w["shape"],
                                   "data_offsets": [end + len(extra), end + len(extra) + (b - a)]}
    extra += data[a:b]
if meta is not None:
    hdr["__metadata__"] = meta
h = json.dumps(hdr, separators=(",", ":")).encode()
h += b" " * ((8 - len(h) % 8) % 8)
open(dst, "wb").write(struct.pack("<Q", len(h)) + h + data + extra)
print(f"{dst}: head_policy copied from head_win ({hdr['head_policy.weight']['shape']})")
