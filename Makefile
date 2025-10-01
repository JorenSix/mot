
run:
	cargo run

release:
	cargo build --release

# for macos aarch64, adapt as needed
install:
	sudo cp target/release/mot /usr/local/bin

macos_aarch:
	#rustup target add aarch64-apple-darwin
	cargo build --release --target=aarch64-apple-darwin

macos_intel:
	cargo build --release --target=x86_64-apple-darwin

rpi_install:
	rustup target add armv7-unknown-linux-gnueabihf
	rustup toolchain install stable-armv7-unknown-linux-gnueabihf

rpi_release:
	cargo build --release --target=armv7-unknown-linux-gnueabihf


linux_install:
	rustup toolchain install stable-x86_64-unknown-linux-gnu
	rustup target add x86_64-unknown-linux-gnu

linux_release:
	cargo build --release --target=x86_64-unknown-linux-gnu

#install linker and rust target
win_install:
	brew install mingw-w64
	rustup target add x86_64-pc-windows-gnu
	rustup toolchain install stable-x86_64-pc-windows-gnu

win_release:
	cargo build --release --target=x86_64-pc-windows-gnu
	cp target/x86_64-pc-windows-gnu/release/mot.exe executables/

#https://rust-lang.github.io/rustup-components-history/
list:
	rustup target list
