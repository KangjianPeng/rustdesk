FROM debian:trixie-slim

WORKDIR /
ARG DEBIAN_FRONTEND=noninteractive
RUN apt update -y && \
    apt install --yes --no-install-recommends \
        g++ \
        gcc \
        git \
        curl \
        libgtk-3-dev \
        libdav1d-dev \
        libvpx-dev \
        libyuv-dev \
        libopus-dev \
        libaom-dev \
        pkg-config \
        libclang-dev \
        cmake \
        libwebkit2gtk-4.1-dev \
        libavcodec-dev \
        libavformat-dev \
        libavutil-dev \
        clang \
        libxcb-randr0-dev \
        libxdo-dev \
        libxfixes-dev \
        libxcb-shape0-dev \
        libxcb-xfixes0-dev \
        libasound2-dev \
        libpulse-dev \
        make \
        libssl-dev \
        sudo \
        libgstreamer1.0-dev \
        libgstreamer-plugins-base1.0-dev \
        ca-certificates \
        ninja-build && \
        rm -rf /var/lib/apt/lists/*

RUN groupadd -r user && \
    useradd -r -g user user --home /home/user && \
    mkdir -p /home/user/rustdesk && \
    chown -R user: /home/user && \
    echo "user ALL=(ALL) NOPASSWD:ALL" | sudo tee /etc/sudoers.d/user

WORKDIR /home/user

USER user
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rustup.sh && \
    chmod +x rustup.sh && \
    ./rustup.sh -y

USER root
ENV HOME=/home/user
COPY ./entrypoint.sh /
ENTRYPOINT ["/entrypoint.sh"]
