FROM scratch
ARG TARGETARCH
COPY linux/${TARGETARCH}/just /just
