.PHONY: buildder build config clean clean-config clean-all

# Default output directory
OUT_DIR ?= out

builder:
	@echo "building builder"
	cargo build --release

	# place builder files
	mkdir -p $(OUT_DIR)/builder
	cp target/release/debr $(OUT_DIR)/builder/
	$(OUT_DIR)/builder/debr help | sed '1s/^/```\n/' | sed '$$a```' > help_output.txt
	cp -r debr/assets $(OUT_DIR)/builder
	cp config.json $(OUT_DIR)/builder/config.json

	# place keyringer
	cp target/release/keyringer $(OUT_DIR)/builder/assets/

	tar -czvf $(OUT_DIR)/builder.tar.gz -C $(OUT_DIR) builder/

build:
	@echo building .iso
	$(OUT_DIR)/builder/debr deps
	$(OUT_DIR)/builder/debr config
	$(OUT_DIR)/builder/debr build

config:
	$(MAKE) clean
	out/builder/debr config
clean:
	rm -rf builder*.tar.gz target/
clean-config:
	rm -rf $(OUT_DIR)/live/config
