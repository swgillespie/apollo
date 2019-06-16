FROM golang:1.12.5-stretch

RUN apt-get update \
    && apt-get install -y curl file sudo build-essential

RUN curl https://sh.rustup.rs > sh.rustup.rs \
    && sh sh.rustup.rs -y \
    && . $HOME/.cargo/env \
    && echo 'source $HOME/.cargo/env' >> $HOME/.bashrc \
    && rustup update \
    && rustup target add x86_64-unknown-linux-musl

ADD src src
ADD benches benches
ADD Cargo.toml Cargo.toml
ADD Makefile Makefile

RUN . $HOME/.cargo/env \
    && cargo build --target x86_64-unknown-linux-musl --release --target-dir build/ \
    && cp ./build/x86_64-unknown-linux-musl/release/apollo ./build/apollo

ADD apollod apollod
RUN cd apollod && go build -o ../build/apollod .
WORKDIR /go/build
CMD ["./apollod"]