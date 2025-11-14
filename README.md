# mcping-rs

Simple program that takes in a Minecraft server address, and outputs its status in JSON format.

It is written in Rust, so it is 🚀🚀 **blazingly fast** 🚀🚀.

It uses the [rust_mc_proto](https://crates.io/crates/rust_mc_proto) crate to talk with Minecraft.

## Usage
Build the program with `cargo build --release`. Resulting binary can be found at `target/release/mcping-rs`.

```
$ ./mcping-rs mc.example.com
{"version":{"name":"1.21.10","protocol":773},"enforcesSecureChat":true,"description":"A Minecraft Server","players":{"max":20,"online":0}}
```