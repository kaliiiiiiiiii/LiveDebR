.PHONY: build clean

# Default output directory
OUT_DIR ?= out

builder:
	@echo "building builder"
	cargo build --release

	mkdir -p $(OUT_DIR)/builder
	cp target/release/debr $(OUT_DIR)/builder/
	$(OUT_DIR)/builder/debr help | sed '1s/^/```\n/' | sed '$$a```' > help_output.txt
	cp -r debr/assets $(OUT_DIR)/builder
	cp -r debr/config.json $(OUT_DIR)/builder/config.json
	tar -czvf $(OUT_DIR)/builder.tar.gz -C $(OUT_DIR) builder/

build:
	@echo building .iso
	$(OUT_DIR)/builder/debr deps
	$(OUT_DIR)/builder/debr config
	$(OUT_DIR)/builder/debr build

clean:
	@echo "cleaning build files"
	rm -rf builder*.tar.gz target/ config.json

clean-all:
	@echo "cleaning all build and out files"
	rm -rf builder*.tar.gz target/ $(OUT_DIR)/ config.json
