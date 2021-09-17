[![Rust](https://github.com/envis10n/libtelnet-rs/workflows/Rust/badge.svg?branch=master)](https://github.com/envis10n/libtelnet-rs)
[![Crates.io](https://img.shields.io/crates/v/libtelnet-rs)](https://crates.io/crates/libtelnet-rs)
[![Docs.rs](https://docs.rs/libtelnet-rs/badge.svg)](https://docs.rs/libtelnet-rs)
# libtelnet-rs

libtelnet-inspired telnet parser for rust.

[Documentation](https://docs.rs/libtelnet-rs)

# BREAKING CHANGES

As of version 2.0, I have switched over to using the `bytes` [crate](https://crates.io/crates/bytes).

With this change, the method signatures for most methods that return `Vec<u8>` now return `Bytes` instead.

For most situations where a `Vec<u8>` would be required, the returned value can simply be changed to utilize `Bytes::to_vec()`.

To make usage of the new dependency a little bit easier, it is also re-exported as `libtelnet_rs::bytes`.

# Usage

Check `src/tests.rs` for an example parser.

Ideally, you would place this parser somewhere directly behind a socket or external source of data.

When data comes in from the socket, immediately send it into the parser with `parser.receive(data)`.

This will append it to the current internal buffer, and then process any events that are in the buffer.

After processing, all telnet events will be returned by `parser.receive()` and can be looped over and handled as needed.

Anything to be sent back over the socket to the remote end should be sent through the parser as well, to ensure any data will be encoded properly for the telnet protocol.

Data to be sent will be provided either by a `events::TelnetEvents::DataSend` event after processing, or as a return from any method used for sending data.
