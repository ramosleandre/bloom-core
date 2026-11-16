import json
import os
import socket
import sys
import time

path, out = sys.argv[1], sys.argv[2]
if os.path.exists(path):
    os.unlink(path)
srv = socket.socket(socket.AF_UNIX)
srv.bind(path)
os.chmod(path, 0o600)
srv.listen(1)
srv.settimeout(40)
res = {"steps": []}


def ask(f, cmd, **kw):
    t = time.perf_counter()
    f.write(json.dumps({"id": len(res["steps"]), "cmd": cmd, **kw}) + "\n")
    f.flush()
    r = json.loads(f.readline())
    r["cmd"] = cmd
    r["roundtrip_ms"] = round((time.perf_counter() - t) * 1000, 2)
    res["steps"].append(r)
    return r


try:
    conn, _ = srv.accept()
    conn.settimeout(8)
    f = conn.makefile("rw")
    res["hello"] = json.loads(f.readline())
    time.sleep(4.0)  # fixture window starts
    ask(f, "prep")
    time.sleep(1.0)
    ask(f, "focus")
    ask(f, "primary")
    ask(f, "keys", keys=[[107, 1], [107, 0], [42, 1], [102, 1], [102, 0], [42, 0]])
    time.sleep(0.4)
    ask(f, "mimes")
    for _ in range(10):
        ask(f, "primary")
    ask(f, "keys", keys=[[29, 1], [46, 1], [46, 0], [29, 0]])
    time.sleep(0.4)  # Ctrl+C
    ask(f, "mimes")
    ask(f, "clipboard")
    ask(f, "paste", text="pasted-by-spike", delay_ms=50)
    time.sleep(0.5)
    ask(f, "clipboard")
    f.close()
    conn.close()
    t = time.perf_counter()
    srv.settimeout(10)
    conn, _ = srv.accept()
    f = conn.makefile("rw")
    json.loads(f.readline())
    res["reconnect_s"] = round(time.perf_counter() - t, 2)
    conn.close()
except Exception as e:
    res["error"] = repr(e)
with open(out, "w") as fh:
    json.dump(res, fh, indent=1)
