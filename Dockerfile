# https://dev.to/sergeyzenchenko/actix-web-in-docker-how-to-build-small-and-secure-images-2mjd#distroless
FROM rust:latest as build

# Damn you sqlx!
ARG DATABASE_URL
ENV PKG_CONFIG_ALLOW_CROSS=1 DATABASE_URL=${DATABASE_URL}

WORKDIR /usr/src/group-expenses
COPY . .

RUN cargo test && cargo install --path .

FROM gcr.io/distroless/cc-debian10

COPY --from=build /usr/local/cargo/bin/group-expenses /usr/local/bin/group-expenses

CMD ["group-expenses"]
