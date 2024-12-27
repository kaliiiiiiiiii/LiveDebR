.PHONY: buildder build config clean clean-config clean-all

# Default output directory
OUT_DIR ?= out
DEBR = $(OUT_DIR)/builder/debr

builder:
	@echo "building builder"
	cargo build --release

	# place builder files
	mkdir -p $(OUT_DIR)/builder
	cp target/release/debr $(OUT_DIR)/builder/
	$(OUT_DIR)/builder/debr help | sed '1s/^/```\n/' | sed '$$a```' > debr_usage.md
	cp -r debr/assets $(OUT_DIR)/builder
	cp config.json $(OUT_DIR)/builder/config.json

	# place keyringer
	mkdir -p $(OUT_DIR)/builder/assets/keyringer
	cp target/release/keyringer $(OUT_DIR)/builder/assets/keyringer
	cp -r keyringer/assets $(OUT_DIR)/builder/assets/keyringer

	tar -czvf $(OUT_DIR)/builder.tar.gz -C $(OUT_DIR) builder/

build:
	$(MAKE) clean-live
	@echo building .iso
	$(DEBR) deps
	$(DEBR) config
	$(DEBR) build

config:
	$(MAKE) clean-live
	$(DEBR) config

clean-live:
	-target/release/debr clean

clean:
	$(MAKE) clean-live
	-rm -rf out/builder/ out/builder.tar.gz

deps:
	apt install -y curl make build-essential libssl-dev pkg-config
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
	