# https://dev.to/sergeyzenchenko/actix-web-in-docker-how-to-build-small-and-secure-images-2mjd#distroless
FROM rust:latest as build
ENV PKG_CONFIG_ALLOW_CROSS=1
# https://github.com/diesel-rs/diesel/blob/master/guide_drafts/backend_installation.md#postgresql
WORKDIR /usr/src/group-expenses
COPY . .
RUN cargo install --path .

FROM debian:slim
RUN apt-get update && apt-get install -y libpq-dev && rm -rf /var/lib/apt/lists/*
COPY --from=build /usr/local/cargo/bin/group-expenses /usr/local/bin/group-expenses
CMD ["group-expenses"]
