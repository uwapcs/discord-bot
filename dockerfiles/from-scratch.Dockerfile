FROM rust:latest as builder

ADD . /usr/src/bot
WORKDIR /usr/src/bot

RUN apt-get update && apt-get install libssl-dev
RUN cargo install --path . --force

FROM debian:stretch-slim
COPY --from=builder /usr/local/cargo/bin/discord-bot /usr/bin
RUN apt-get update && apt-get install libssl1.1 cowsay
RUN cp /usr/games/cowsay /usr/bin

RUN mkdir -p /bot/config
VOLUME /bot/config
WORKDIR /bot
ENTRYPOINT /usr/bin/discord-bot
