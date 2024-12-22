# Building

1. Download and extract the **builder** from [releases](https://github.com/kaliiiiiiiiii/LiveDebR/releases/latest)
2. Install dependencies with `./debr --debs`
3. Build image with `./debr -c config.json -o debian_amd64.iso` 


# Building the builder
only tested on debian

install dependencies
```bash
# install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# other dependencies
sudo apt install make build-essential
```

build
```bash
git clone https://github.com/kaliiiiiiiiii/LiveDebR.git
cd LiveDebR
make builder OUT_DIR=out
```

the builder can then be found under `out/builder`  and `out/builder.tar.gz`