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

# Building

Following commands may be important for starting development when building
```bash
sudo $debr clean
sudo $debr config
sudo $debr build
```


# Adding keys to the configuration
1. Add key to [debr/post_cfg/json_cfg.rs](debr/post_cfg/json_cfg.rs)
2. Add the parsing logic for the key to [debr/post_cfg.rs](debr/post_cfg.rs)
3. Document the key in [docs/Readme.md](docs/Readme.md)

# Developing modules
Modules are added to debr/assets/modules/module_name.json