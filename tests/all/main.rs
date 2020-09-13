#[macro_use]
extern crate diesel_migrations;

embed_migrations!();

mod configuration;
mod graphql;
mod health_check;
mod helpers;
