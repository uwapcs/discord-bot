FROM debian:stretch-slim
COPY target/release/discord-bot /usr/bin
RUN apt-get update && apt-get install -y libssl1.1 cowsay fortune
RUN cp /usr/games/* /usr/bin

RUN mkdir -p /bot/config
VOLUME /bot/config
WORKDIR /bot
ENTRYPOINT /usr/bin/discord-bot
