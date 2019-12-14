# PBFT

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

{"type": "ClientRequest", "payload": "{\"operation\":\"testOperation\",\"timestamp\":1}"}
```
