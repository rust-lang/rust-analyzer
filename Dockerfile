FROM debian:stretch as base
USER root
RUN apt-get update && \
    apt-get install -y \
      musl-dev musl-tools file nano git zlib1g-dev cmake make g++ curl pkgconf \
      linux-headers-amd64 ca-certificates xutils-dev \
      --no-install-recommends && \
    rm -rf /var/lib/apt/lists/*
ENV MUSL_PREFIX=/musl
RUN mkdir /workdir && mkdir $MUSL_PREFIX
WORKDIR /libworkdir
RUN ln -s /usr/include/x86_64-linux-gnu/asm /usr/include/x86_64-linux-musl/asm && \
    ln -s /usr/include/asm-generic /usr/include/x86_64-linux-musl/asm-generic && \
    ln -s /usr/include/linux /usr/include/x86_64-linux-musl/linux
RUN curl -sL https://zlib.net/zlib-1.2.11.tar.gz | tar xz
RUN cd zlib-1.2.11 && \
    CC="musl-gcc -fPIE -pie" LDFLAGS="-L/musl/lib/" CFLAGS="-I/musl/include" \
      ./configure --prefix=$MUSL_PREFIX && \
    make -j$(nproc) && \
    make install

FROM base as rust-base
ARG RUST_VER=1.43.0
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y  --default-toolchain ${RUST_VER}
ENV PATH=/root/.cargo/bin:$PATH
RUN cargo install cargo-tree
RUN rustup target add x86_64-unknown-linux-musl
ENV PATH=$MUSL_PREFIX/bin:$PATH \
    PKG_CONFIG_ALLOW_CROSS=true \
    PKG_CONFIG_ALL_STATIC=true \
    PKG_CONFIG_PATH=$MUSL_PREFIX/lib/pkgconfig \
    SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt \
    SSL_CERT_DIR=/etc/ssl/certs \
    LIBZ_SYS_STATIC=1
WORKDIR /workdir

FROM rust-base as builder
ADD . /workdir
RUN PKG_CONFIG_ALLOW_CROSS=1 cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:3.11.3
COPY --from=builder /workdir/target/x86_64-unknown-linux-musl/release/rust-analyzer /usr/local/bin/
ENTRYPOINT ["rust-analyzer"]
