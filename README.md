# frametrace

> Debug VP8 RTP streams

`frametrace` is a proof-of-concept library to output debug details of a RTP/VP8 stream. A working example of using `frametrace` with `mediasoup` is located in [src/bin/echoserver.rs](src/bin/echoserver.rs). The `mediasoup` `echoserver` demo was chosen as a simple 1:1 webrtc call that only requires one person/browser tab to try out.

## Build requirements

- recent stable version of [rust](https://rustup.rs/)

- [`mediasoup` requirements](https://mediasoup.org/documentation/v3/mediasoup/installation/) for your operating system. Note that the `mediasoup` crate this project depends on will build `mediasoup` automatically.

- yarn

## How to run the example

> Tested on WSL and MacOS (native windows unsupported).

1. Start the local web server

```bash
cd web
yarn && yarn start
```

2. In a seperate shell window, start the `mediasoup` `echoserver`

```bash
cargo run
```

3. Navigate to [http://localhost:3001](http://localhost:3001) in a web browser.

4. Video frame logs will be streamed to `$PWD/video_log.json`

## Devlog

Notes taken along the way can be found in [the devlog](devlog.md).

## Future Work

- handle out-of-order RTP packets (I'm not sure if MediaSoup is doing any sort of rtx/reordering under the hood for direct transports)
- add more fields/information about vp8 frames (i.e. bitrate, num partitions, probably many other things)

## libvpx tests

The VP8 parser can be tested against libvpx using `cargo test`. Note that libvpx must be present on the machine and findable by the [system_deps](https://crates.io/crates/system-deps) crate.
