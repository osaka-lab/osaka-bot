FROM rust

WORKDIR /app
COPY . .

RUN chown -R 1000:1000 /app

RUN cargo install --path .

USER 1000:1000

CMD ["osaka-bot"]