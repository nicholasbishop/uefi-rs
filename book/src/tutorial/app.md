# Creating a UEFI application

## Install dependencies

Follow the [Rust installation instructions] to set up Rust.

## Create a minimal application

Create an empty application and change to that directory:

```sh
cargo new my-uefi-app
cd my-uefi-app
```

In `cargo.toml`, add a few dependencies:

```toml
[dependencies]
log = "0.4"
uefi = "0.16"
uefi-services = "0.13"
```

Replace the contents of `src/main.rs` with this:

```rust
{{#include ../../../uefi-test-runner/examples/hello_world.rs:all}}
```

## TODO

Let's look a quick look at what each part of the program is doing.

```rust
{{#include ../../../uefi-test-runner/examples/hello_world.rs:features}}
```

[Rust installation instructions]: https://www.rust-lang.org/tools/install
