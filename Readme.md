# Live Debian Build Rust
Build debian live based on a json config with rust

# Building

Installation
```bash
curl -L -o builder_linux_amd64_deb.tar.gz https://github.com/kaliiiiiiiiii/LiveDebR/releases/latest/download/builder_linux_amd64_deb.tar.gz
tar -xzvf builder_linux_amd64_deb.tar.gz
cd builder
```

download build from releases
```bash
curl -L -o live-image-amd64.hybrid.iso.gz https://github.com/kaliiiiiiiiii/LiveDebR/releases/latest/download/live-image-amd64.hybrid.iso.gz
gunzip live-image-amd64.hybrid.iso.gz
```

## Usage
The usage can be found in [release](https://github.com/kaliiiiiiiiii/LiveDebR/releases/latest)

where config file should be in the form [config.json](https://github.com/kaliiiiiiiiii/LiveDebR/blob/main/config.json)

> TODO: build propper docs

# References

- [nodiscc/debian-live-config](https://github.com/nodiscc/debian-live-config)