# Clown Proxy

What is this? It's a simple HTTP proxy that intercepts web pages, and makes a 
few changes:

- substitutes all instances of the word "happy" with "silly", preserving the
original case
    - e.g., if word "hApPy" is present, it will be replaced with "sIlLy"
- replaces all .jpg images with one of two randomly chosen images of a happy
clown

# Features

- HTTP 1.1
- Handle responses of arbitrary size
- Runs on arbitrary, user-specified port
- Asynchronous using Tokio's multi-thread scheduler
- Unit tests to verify domain parsing and string substitution
    - run with `cargo test`
- Ignores all non-GET requests
- idk its cool

# Usage

The proxy requires one argument: a port number. Any port number can be chosen,
so long as you have sufficient privileges to use said port.

`cargo run --release <port>`

Note that you'll need to adjust the proxy and HTTP settings on your browser to
use it with the proxy:

- set IP of the proxy
- set port of the proxy
- set browser HTTP to 1.0 or 1.1

# Potential Future Features

- signal handling
