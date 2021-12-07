# frametrace

> Debug VP8 RTP streams

## Build requirements

- recent stable version of [rust](https://rustup.rs/)

- [`mediasoup` requirements](https://mediasoup.org/documentation/v3/mediasoup/installation/)

## Future Work

- handle out-of-order RTP packets (I'm not sure if MediaSoup is doing any sort of rtx/reordering under the hood for direct transports)
