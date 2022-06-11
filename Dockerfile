FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:edge

RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install --assume-yes libxcb-render0-dev:arm64 libxcb-shape0-dev:arm64 libxcb-xfixes0-dev:arm64 libxkbcommon-dev:arm64