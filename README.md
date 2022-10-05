# NETLIB

---

Writing network library which keeps C style in Rust.

The main goal is learning Linux and TCP/IP stack.

It should be obviously heuristic for production.

## Source layout

```yaml

- bin: Example executable files
- res: Example resource files
- src:
    - datalink: Datalink layer, L2
    - network: Network layer, L3
    - transport: Transport layer, L4
    - application: Application layer, L5-L7
    - data: Releated data structures
    - dev: Device files
    - c_error: C oriented error (low-level error)
    - rs_error: Rust oriented error (high-level error)
    - view: Data wrapper for readable debug output
```
