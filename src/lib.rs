#[macro_use]
extern crate diesel;

pub mod infrastructure;

pub use infrastructure::{
    config::*,
    http::run,
    repositories::{get_pool, PostgresPool},
};
