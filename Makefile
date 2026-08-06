cargo = cargo
venv = .venv

.PHONY: all build run check clean

all: build

build:
	$(cargo) build

run:
	$(cargo) run

check:
	$(cargo) check
	$(cargo) fmt --check
	$(cargo) clippy --all-targets --all-features -- -D warnings
	$(cargo) deny check

clean:
	$(cargo) clean
	rm -rf $(venv)
