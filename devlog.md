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

- Echo example failed to receive video. After some debugging it looks like WSL [doesn't support UDP](https://github.com/microsoft/WSL/issues/6082), with some not super great [workarounds](https://github.com/microsoft/WSL/issues/4825). Since WSL can run GUI apps, I also tried running an in-WSL chromium, but WSL doesn't have access to the webcam so I couldn't produce a video stream. I could in theory create a synthetic one using canvas. Perhaps I will bite the bullet and see if Cygwin just works TM.

- The cygwin mediasoup build failed on building the wheel for ninja. Upgrading pip and trying again gave a "cannot find meson" error. It seemed like the build script was not starting from the begining. Running `cargo clean -p mediasoup-sys` then re-running the build gave a seemingly more useful error about pip not supporting the `--system` flag. I then disabled all of the "app execution aliases" for python (a tricky windows feature) as listed in the mediasoup install docs. Finally I realized I was running in msys and not mingw64 and switched environments.

- Re-reading the install directions made me realize that mediasoup just needs the gnu `make.exe` from msys, but should be run in native windows environment. Ok back to the drawing board.

- `mediasoup` recommends adding the `make.exe` from mingw to your path env var, but I don't want it to pollute my path for other projects. `mediasoup-sys` hard-codes the `make` command, so I am going to try and fork it to allow a `MAKE` env var to override the executable. ... tried this but ended up having lots of MSYS python issues. Instead I'm going to go back to native windows and try to get the build working there.

- Making some changes to the mediasoup build.rs and Makefile, I made it work on Windows. Only compiling on cmd.exe worked for me (after setting vcvars as mediasoup instructs). Powershell didn't handle mixed file path slashes (/ \\) correctly. Now the build works with `cmd.exe` and scoop-installed `python` and `make`.

- With the build working, I ended up running into the same connectivity issue as before. I looked at chrome://webrtc-internals and saw that nothing was even being sent. Then I realized ICE was failing. The client was gathering candidates for 192.\* and 172.\*, but the echo server was only listening on localhost. Hard-coding listening on my 192.\* address fixed this issue, but I will probably find a programmatic way to get the local IPs. I was surprised mediasoup's client library didn't log any errors in this case. I tried adding some event listeners to the transport/producer and didn't have any luck surfacing this ICE failure client-side.
