# Building the builder
Linux only, tested on debian

Download source code
```bash
git clone https://github.com/kaliiiiiiiiii/LiveDebR.git
cd LiveDebR
```

Install dependencies
```bash
sudo apt install make
sudo make deps
```

Build
```bash
git clone https://github.com/kaliiiiiiiiii/LiveDebR.git
cd LiveDebR
make builder OUT_DIR=out
```

The builder can then be found under `out/builder/` \
In the VSC console, it is accessible over `$debr --help`