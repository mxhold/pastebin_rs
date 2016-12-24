# pastebin_rs

A simple pastebin server in [Rust](https://www.rust-lang.org) using
[Iron](https://github.com/iron/iron)

## Usage

Start with `cargo run`, then:

```bash
$ echo 'Hello, world!' | curl http://localhost:3000 --data-binary @-
http://localhost:3000/a2c6abfe-4999-4c81-8bfb-aed050529e98

$ curl http://localhost:3000/a2c6abfe-4999-4c81-8bfb-aed050529e98
Hello, world!
```
