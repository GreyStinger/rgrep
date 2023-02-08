all:
	cargo build --release

install: all
	cp target/release/rgrep /bin/rgrep

uninstall:
	rm /bin/rgrep

.PHONY: clean

clean:
	rm -fr ./target
