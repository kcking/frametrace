# Goal

Extract relevant information from VP8 frames in an RTP stream.

# Considered Approaches

## Capture local rtp stream

Run wireshark and capture srtp packet stream.

Pros

- source of truth for sent network packets before loss occurs

Cons

- requires running additional packet capture software on client
- decrypting stream is difficult or inconvenient
  - extract shared secret from server or client and decrypt session manually
  - disable srtp encryption with chrome flag
- logs would have to be uploaded from client

## Custom WebRTC Client

Use a library like pion/libwebrtc/webrtc.rs to connect to SFU . Have this custom client log frame details.

Pros

- not beholden to browser implementation

Cons

- SFU may mask underlying issues in the RTP stream such as frequent retransmits or packet reordering
- introducing another network hop (even if local) could cause additional network issues

## Analyze RTP stream at SFU

Pros

- stream likely going through SFU anyways for privacy + scalability
- client-agnostic
- easy to aggregate logs

Cons

- depedendent on SFU support for intercepting RTP packets (yay mediasoup DirectTransport, pion interceptors)
- one step removed from source of truth of sent RTP packets

# Initially Chosen Approach

After browsing [mediasoup DirectTransport](https://docs.rs/mediasoup/0.9.0/mediasoup/router/struct.Router.html#method.create_direct_transport), this seemed like a very ergonomic way to shim into the decrypted RTP stream. As for depacketizing the vp8 frames, I plan on using [webrtc.rs's implementation](https://docs.rs/rtp/latest/rtp/codecs/vp8/struct.Vp8Packet.html). Then the actual VP8 frames must be parsed to extract the relevant information. An initial search didn't bring up any existing crates to do this, so I may implement the parsing I need using [`nom`](https://crates.io/crates/nom) and the [VP8 RFC](https://datatracker.ietf.org/doc/html/rfc6386#section-19.2).

# Devlog

- Ran into issues building mediasoup on windows. The mediasoup Makefile requires a bash-aware make (i.e. cygwin make). Building with the `scoop`-installed gnuwin32 make fails since it uses `cmd.exe` for shell commands. It looks like this is a [known issue](https://github.com/versatica/mediasoup/issues/701). Generally I prefer WSL to cygwin so I switched to developing inside of a Debian WSL instance as opposed to cygwin. WSL compilation worked after `sudo apt install python3-pip` 🎉

- Error compiling initial example

  ```
  the trait `actix::actor::Actor` is not implemented for `EchoConnection`
  ```

  Caused because `actix-web-actors -> actix` version (0.10) mismatched latest `actix` version (0.12) that was added to our crate by `cargo add`. Solved by downgrading `actix` to 0.10 in our `Cargo.toml`.
