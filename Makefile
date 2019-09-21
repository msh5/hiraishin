.PHONY: build install

build:
	cargo build

install:
	cargo install --path . --force