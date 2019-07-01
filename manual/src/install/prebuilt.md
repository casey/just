# Pre-built Binaries

Pre-built binaries for Linux, MacOS, and Windows can be found on the [releases page](https://github.com/casey/just/releases).

You can use the following command to download the latest binary for MacOS or Windows, just replace `DESTINATION_DIRECTORY` with the directory where you’d like to put `just`:

```sh
curl -LSfs https://japaric.github.io/trust/install.sh | \
  sh -s -- --git casey/just --to DESTINATION_DIRECTORY
```

On Linux, use:

```sh
curl -LSfs https://japaric.github.io/trust/install.sh | \
  sh -s -- --git casey/just --target x86_64-unknown-linux-musl --to DESTINATION_DIRECTORY
```