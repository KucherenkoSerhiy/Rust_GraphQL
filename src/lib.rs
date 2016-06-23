#![crate_name = "rust_sql"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]

#[macro_use]
extern crate mysql;
extern crate mio;
extern crate eventual;
extern crate bytes;

#[macro_use]
extern crate log;
extern crate env_logger;

mod def;
mod connection;