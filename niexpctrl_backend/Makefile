# Default target (will run when you type `make` with no arguments)
all: test

# Define the test target
test:
	@echo "Running tests..."
	cargo test

run:
	@echo "Running the main function..."
	cargo run

dev:
	@echo "Starting real-time build and watch.."
	cargo watch -x build

# Define the export target
export:
	@echo "Building python and rust library ..."
	cargo build
	maturin develop

export_optimized:
	@echo "Building python and rust library with heavy optimization ..."
	cargo build --release
	maturin develop --release

publish: 
	@echo "Updating tests, docs, and exporting to python, and publishing crate"
	cargo fmt && cargo doc && cargo test 
	git add .
	git commit -am "publish update" && git push 
	maturin develop --release
	cargo publish