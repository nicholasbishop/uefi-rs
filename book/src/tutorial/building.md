# Building

## Install dependencies

Install the nightly [toolchain] and the `rust-src` component:
```sh
rustup toolchain install nightly --component rust-src
```

_Nightly is currently required because uefi-rs uses some unstable
features. The [`build-std`] feature we use to build the standard
libraries is also unstable._

## Build the application

Run this command to build the application:

```sh
cargo +nightly build --target x86_64-unknown-uefi \
    -Zbuild-std=core,compiler_builtins,alloc \
    -Zbuild-std-features=compiler-builtins-mem
```

This will produce an x86-64 executable:
`target/x86_64-unknown-uefi/debug/my-uefi-app.efi`.

[toolchain]: https://rust-lang.github.io/rustup/concepts/toolchains.html
[`build-std`]: https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#build-std

## Simplifying the build command

The above build command is verbose and not easy to remember. With a bit
of configuration we can simplify it a lot.

First, use `rustup` to use the nightly toolchain by default for the
application:

```sh
rustup override set nightly
```

Now create a `.cargo` directory in the root of the project:

```sh
mkdir .cargo
```

Create `.cargo/config.toml` with these contents:

```toml
[build]
target = "x86_64-unknown-uefi"

[unstable]
build-std = ["core", "compiler_builtins", "alloc"]
build-std-features = ["compiler-builtins-mem"]
```

Now you can build much more simply:

```sh
cargo build
```
