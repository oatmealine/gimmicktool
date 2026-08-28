# sometsuki

**Sometsuki** is a simple two-way communication protocol for NotITG using the
GetExternal/SetExternal method pair. The host (the NotITG process) may send
messages by writing with SetExternal which up to 1 client (external program)
read with syscalls to read the NotITG process's memory, and the client may send
messages to the host by writing to the NotITG process's memory.

## Structure

Sometsuki enforces a simple structure upon the external memory space: 1 byte is
used as the header, while the rest is the data buffer. This byte indicates the
current state of the protocol:

| Value | Direction | Shorthand | Meaning |
|-|-|-|-|
| `0x01` | H→C | ACK     | The message has been acknowledged by the host |
| `0x02` | H→C | WRITING | The host is currently writing a message, the client should hold on reading |
| `0x03` | H→C | READY   | The host has written a message ready for consumption |
| `0x04` | C→H | ACK     | The message has been acknowledged by the client |
| `0x05` | C→H | WRITING | The client is currently writing a message, the host should hold on reading |
| `0x06` | C→H | READY   | The client has written a message ready for consumption |

The host should only write values `0x01` - `0x03` to the header, while the
client should only write values `0x04` - `0x06` to the header.

Any values outside of this range should be considered invalid and written over
with an ACK (`0x01`, `0x04`).

## Communication

If a party wants to send a message to another party, it must store that message
in the **write buffer**. Then, every single frame (or whichever frequency is
more convenient), it should check the header to see what it should do:

- If the header is an ACK from either party, it should proceed with writing the
  message from the write buffer.
- If the header is a READY message header from the other party, it should first
  consume this message by writing it to a read buffer, then either write an ACK
  if the write buffer is empty or proceed with writing the message from the
  write buffer if it is not.
- If the header is a READY message header from itself, it should do nothing and
  wait until the other party consumes the message.
- If the header is a WRITING message header from either party, it should do
  nothing and wait until the message is fully written to avoid reading a
  malformed unfinished message.
  - Note that under single-threaded environments (such as NotITG), a WRITING
    message from itself should be considered invalid.

### Timeouts

The host and client should both keep track of how long ago their last message
write to the external memory buffer was. If, when holding for the other party
(when the header is a READY from itself or a WRITING from the other party), it
has been more than 5 seconds since the last message was sent, the connection
should be considered dropped.

## Messages

A message is a string of characters, specifically UTF-8 JSON data written in
indices 1 to the end of the external memory space. Data is packed such that each
i32 external memory "slot" holds 4 u8 characters. `0x00` is used as a stream end
marker, while `0x01` is used as a message end marker.

For example, when sending 3 messages, each one should have a `0x01` marker after
itself, except for the last, which has a `0x00` marker to terminate the stream.

Data after a `0x00` marker should be entirely disregarded.

Messages and message streams should be split into multiple writes to the memory
space if they cannot fit within it. Therefore, when reading a message, if it
reaches the end of the memory space without a `0x00`, you should consider this
message unfinished, emit an ACK, and wait for the other party to complete it in
a later write.

### Format

Messages have only 1 required field, which is `t` (short for type). This
discerns what kind of message it is.

Sometsuki only defines a few message types necessary for basic connection
tracking, however implementing them is not strictly necessary depending on your
context.

### Establishing a connection

The `hello` message type is used to attempt to establish a connection. The
client should be the first to send this message, after which the host should
respond with another `hello`, after which the connection is established.

The client should always send this message upon connecting, disregarding the
previous value of the buffer. This also means that the host, upon seeing it,
should consider the previous connection closed if there is one currently open.

Any other messages sent before this one with the exception of `error` and
`goodbye` should be ignored. Duplicate `hello` messages should be considered
invalid. Errors encountered with this message should count as fatal and
terminate the connection.

If the connection context requires it, this is when connection state should be
first established.

The `hello` message type has two required fields:
- `n` (short for name), representing the name of the client/host (for the host,
  this should be the theme name)
- `v` (short for version), representing the name of the client/host (for the
  host, this should be the theme version)

### Heartbeats

If too much time has passed since the last message or ACK from the other party,
a `heartbeat` message type should be sent to ensure the connection has not been
dropped. To prevent both parties attempting a heartbeat at the exact same time,
the host should wait 5 seconds, while the client should wait 10 seconds (because
the host is most likely to be the one to freeze up for long durations of time,
being single-threaded). This message type has no other special behavior or
required fields, as ensuring the heartbeat succeeds is already handled by
[Timeouts](#timeouts).

### Closing a connection

The `goodbye` message type is used to close connections. These messages do not
require an ACK or any response, as immediately after being sent both the sending
and receiving party should consider the connection closed.

### Errors

When given an invalid message from the other party (such as an invalid header,
malformed JSON or UTF-8, or message format issues), the special `error` message
type should be sent, and the connection should not be dropped.

The `error` message type has one required field, which is `m` (short for
message): an arbitrary string with a message representing the error.