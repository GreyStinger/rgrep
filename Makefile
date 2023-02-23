all:
	cargo build --release

install: all
	cp target/release/rgrep /bin/rgrep

dos:
	cargo build --profile dos --features no_colour
	cargo objcopy --profile dos -- -O binary --binary-architecture=i386:x86 rust_dos.com

uninstall:
	rm /bin/rgrep

.PHONY: clean

clean:
	rm -fr ./target
