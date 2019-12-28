# PBFT

A PBFT implementation based on [rust-libp2p](https://github.com/libp2p/rust-libp2p).

```bash
####################################
# Primary Replica
####################################
$ cargo run primary
[main] node_type: Primary
[ClientHandler::new] Listening on V4(127.0.0.1:8000)
...
...

####################################
# Backup Replicas
####################################
$ cargo run 
[main] node_type: Backup
[ClientRequestHandler::new] Listening on V4(127.0.0.1:65450)
...
...

$ cargo run 
...

$ cargo run 
...

####################################
# Client
####################################
# Run tcp listener to receive replies from the replicas
$ nc -kl 9000

# Send a request to the Primary replica
$ telnet 127.0.0.1 8000
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.

{"ClientRequest": {"operation": "testOperation", "timestamp": 1} }
```
