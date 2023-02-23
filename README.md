# RGREP

A clone of grep built in rust

Currently bare-bones but supports both piped data and file + directory searching

## Compiling for DOS

To compile to a DOS executable you need binutils and llvm-tools-preview

```bash
cargo install cargo-binutils
rustup component add llvm-tools-preview
```

Then either build the project and generate a COM file with:

```bash
cargo build --profile dos --features no_colour
cargo objcopy --profile dos -- -O binary --binary-architecture=i386:x86 rust_dos.com
```

or run:

```bash
make dos
```
