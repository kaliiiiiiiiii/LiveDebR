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
```
Usage: debr [OPTIONS] [COMMAND]

Commands:
  deps    Install dependencies
  config  Initialize build
  clean   Clean build
  build   Build live
  help    Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>    Path to the configuration file [default: config.json]
  -o, --out-dir <OUT_DIR>  Path for the live-debian-build to use [default: out]
  -h, --help               Print help
  -V, --version            Print version
```

where config should be in the form [config.json](https://github.com/kaliiiiiiiiii/LiveDebR/blob/main/config.json)

The builded images can then be found under `out-dir/live` 


# Building the builder
only tested on debian

install dependencies
```bash
# install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# other dependencies
sudo apt install make build-essential libssl-dev pkg-config
```

build
```bash
git clone https://github.com/kaliiiiiiiiii/LiveDebR.git
cd LiveDebR
make builder OUT_DIR=out
```

the builder can then be found under `out/builder`  and `out/builder.tar.gz`