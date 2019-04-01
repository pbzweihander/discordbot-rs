FROM clux/muslrust:1.33.0-stable

WORKDIR /
RUN USER=root cargo new --bin discordbot
WORKDIR /discordbot

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/x86_64-unknown-linux-musl/release/deps/discordbot*
RUN cargo build --release

FROM scratch

COPY --from=0 /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=0 /discordbot/target/x86_64-unknown-linux-musl/release/discordbot .

CMD [ "./discordbot" ]
