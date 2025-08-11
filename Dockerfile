# syntax=docker/dockerfile:1

# NOTE: ARGs `BUILDPLATFORM` + `TARGETARCH` are implicitly defined by BuildKit:
# https://docs.docker.com/reference/dockerfile/#automatic-platform-args-in-the-global-scope
# NOTE: BuildKit supplied ARGs use convention amd64 / arm64 instead of the desired x86_64 / aarch64
# https://itsfoss.com/arm-aarch64-x86_64
#
# Map arch naming conventions from BuildKit to Rust (BUILDARCH => BUILDARCH_RUST):
FROM --platform=linux/amd64 alpine AS downloader-amd64
ARG BUILDARCH_RUST=x86_64
FROM --platform=linux/arm64 alpine AS downloader-arm64
ARG BUILDARCH_RUST=aarch64

# Fetch the expected version of `just` via GH Releases:
FROM downloader-${BUILDARCH} AS downloader
SHELL ["/bin/ash", "-eux", "-o", "pipefail", "-c"]
# This ARG will be set via GitHub Actions during release builds
ARG JUST_VERSION
ARG RELEASE_URL="https://github.com/casey/just/releases/download/${JUST_VERSION}/just-${JUST_VERSION}-${BUILDARCH_RUST}-unknown-linux-musl.tar.gz"
RUN wget -O - "${RELEASE_URL}" \
    | tar --directory /usr/local/bin --extract --gzip --no-same-owner just

# Use scratch for minimal final image - no OS, just the binary
# This results in a ~10MB image vs ~50MB+ with a full OS
FROM scratch
COPY --from=downloader /usr/local/bin/just /usr/local/bin/just

# Default to running just with help if no arguments provided
ENTRYPOINT ["just"]
CMD ["--help"]