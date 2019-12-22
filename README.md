# PBFT

A PBFT implementation based on [rust-libp2p](https://github.com/libp2p/rust-libp2p).

```bash
# primary replica
$ cargo run primary

# backup replicas
$ cargo run 
$ cargo run 
$ cargo run 

# client
$ telnet 127.0.0.1 8000
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.

{"ClientRequest": {"operation": "testOperation", "timestamp": 1} }
```
