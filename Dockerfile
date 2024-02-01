FROM rust:slim-bullseye as base-image

RUN apt update && apt install -y \
    cmake=3.18.4-2+deb11u1 \
    clang=1:11.0-51+nmu5 \
    build-essential=12.9 \
    llvm=1:11.0-51+nmu5

FROM base-image as builder
ADD simulator /app/simulator
ADD radix-engine /app/radix-engine
ADD radix-engine-interface /app/radix-engine-interface
ADD sbor /app/sbor
ADD sbor-derive /app/sbor-derive
ADD sbor-derive-common /app/sbor-derive-common
ADD utils app/utils
ADD scrypto-schema /app/scrypto-schema
ADD radix-engine-common /app/radix-engine-common
ADD radix-engine-derive /app/radix-engine-derive
ADD native-sdk /app/native-sdk
ADD radix-engine-macros /app/radix-engine-macros
ADD radix-engine-profiling /app/radix-engine-profiling
ADD radix-engine-store-interface /app/radix-engine-store-interface
ADD radix-engine-stores /app/radix-engine-stores
ADD transaction /app/transaction
ADD radix-engine-queries /app/radix-engine-queries
ADD assets /app/assets

WORKDIR /app

RUN cargo install --path ./simulator

FROM base-image
COPY --from=builder /app/simulator/target/release/scrypto /usr/local/bin/scrypto
RUN rustup target add wasm32-unknown-unknown
WORKDIR /src

ENTRYPOINT ["scrypto", "build", "--path", "/src"]
