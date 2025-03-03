current: bound

debug:
	cargo build
	PATH=/Users/khang/repos/ln/ln/target/debug:$$PATH git -C ~/repos/ip ln -n 200

bound:
	cargo build
	PATH=/Users/khang/repos/ln/ln/target/debug:$$PATH git -C ~/repos/ip ln --bound

install:
	cargo install --locked --force --path .
