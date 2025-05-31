FROM rust:latest


WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() { println!(\"placeholder\"); }" > src/main.rs

RUN cargo build --release || true

COPY ./src ./src

RUN cargo build --release

RUN cargo install --path .

CMD ["da-bot"]