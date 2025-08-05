# Multi-stage build to download the just binary and create a minimal container
FROM alpine:latest as downloader

# Version will be passed from GitHub Actions during release builds
ARG JUST_VERSION
# Docker automatically provides this for multi-platform builds
ARG TARGETARCH

# Download the appropriate just binary for the target architecture
# Map Docker's platform names to just's release naming convention
RUN set -eux; \
    case "$TARGETARCH" in \
        "amd64") ARCH="x86_64-unknown-linux-musl" ;; \
        "arm64") ARCH="aarch64-unknown-linux-musl" ;; \
        *) echo "Unsupported architecture: $TARGETARCH" && exit 1 ;; \
    esac; \
    wget -O - "https://github.com/casey/just/releases/download/${JUST_VERSION}/just-${JUST_VERSION}-${ARCH}.tar.gz" \
        | tar --directory /usr/local/bin --extract --gzip --file - --no-same-owner just

# Use scratch for minimal final image - no OS, just the binary
# This results in a ~10MB image vs ~50MB+ with a full OS
FROM scratch
COPY --from=downloader /usr/local/bin/just /usr/local/bin/just

# Set working directory for when just is run
WORKDIR /work

# Default to running just with help if no arguments provided
ENTRYPOINT ["just"]
CMD ["--help"]