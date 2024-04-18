# Serverless Protocol Over Serial

Allows the VMM and Agent to communicate over a serial connection.

## How to test it

1. Use socat to create a virtual serial port:

```bash
socat -d -d pty,raw,echo=0 pty,raw,echo=0
```

2. Run the example:

```bash
cargo run --example serverless_protocol_example -- --serial-path-a=<path_to_first_pty> --serial-path-b=<path_to_second_pty>
```

This example will show how processes can communicate over a serial connection.

