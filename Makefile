.PHONY: build clean

# Default output directory
OUT_DIR ?= out
KEYRINGER_OUT = $(OUT_DIR)/builder/assets/keyringer

builder:
	@echo "building builder"
	cargo build --release

	# place builder files
	mkdir -p $(OUT_DIR)/builder
	cp target/release/debr $(OUT_DIR)/builder/
	$(OUT_DIR)/builder/debr help | sed '1s/^/```\n/' | sed '$$a```' > help_output.txt
	cp -r debr/assets $(OUT_DIR)/builder
	cp -r debr/config.json $(OUT_DIR)/builder/config.json

	# place keyringer files
	mkdir -p $(KEYRINGER_OUT)
	cp target/release/keyringer $(KEYRINGER_OUT)
	cp keyringer/Readme.md $(KEYRINGER_OUT)
	cp keyringer/assets/keyringer.service $(KEYRINGER_OUT)


	tar -czvf $(OUT_DIR)/builder.tar.gz -C $(OUT_DIR) builder/

build:
	@echo building .iso
	$(OUT_DIR)/builder/debr deps
	$(OUT_DIR)/builder/debr config
	$(OUT_DIR)/builder/debr build

clean:
	@echo "cleaning build files"
	rm -rf builder*.tar.gz target/ config.json
clean-config:
	@echo "cleaning all build and out files"
	rm -rf $(OUT_DIR)/live/config
clean-all:
	@echo "cleaning all build and out files"
	rm -rf builder*.tar.gz target/ $(OUT_DIR)/ config.json
