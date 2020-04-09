# libtelnet-rs

libtelnet-inspired telnet parser for rust.

[Documentation](https://envis10n.github.io/libtelnet-rs/libtelnet_rs/)

# Usage

Check `src/tests.rs` for an example parser.

Ideally, you would place this parser somewhere directly behind a socket or external source of data.

When data comes in from the socket, immediately send it into the parser with `parser.receive(data)`.

This will append it to the current internal buffer, and then process any events that are in the buffer.

After processing, all telnet events will be pushed out through the event hooks provided to the parser via `parser.add_hooks(hooks_struct)`.

Anything to be sent back over the socket to the remote end should be sent through the parser as well, to ensure any data will be encoded properly for the telnet protocol.

Data to be sent will be pushed to the event hooks through the `on_send` event method.
