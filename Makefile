.PHONY: build clean

# Default output directory
OUT_DIR ?= out

builder:
	@echo "building builder"
	cargo build --release

	mkdir -p $(OUT_DIR)/builder
	cp target/release/debr $(OUT_DIR)/builder/
	cp -r debr/assets $(OUT_DIR)/builder
	cp -r debr/config.json $(OUT_DIR)/builder/config.json
	tar -czvf $(OUT_DIR)/builder.tar.gz $(OUT_DIR)/builder/

clean:
	@echo "cleaning build files"
	rm -rf builder*.tar.gz target/ config.json

clean-all:
	@echo "cleaning all build and out files"
	rm -rf builder*.tar.gz target/ $(OUT_DIR)/ config.json
