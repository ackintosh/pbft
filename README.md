# PBFT

```bash
# primary replica
$ cargo run 8000

# backup replicas
$ cargo run 8001
$ cargo run 8002
$ cargo run 8003

# client
$ telnet 127.0.0.1 8000
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.

{"operation":"testOperation","timestamp":1}
```
