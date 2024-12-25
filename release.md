# Usage

Download prebuilt iso image
```bash
curl -L -o live-image-amd64.hybrid.iso.gz https://github.com/kaliiiiiiiiii/LiveDebR/releases/download/{$tag}/live-image-amd64.hybrid.iso.gz
gunzip live-image-amd64.hybrid.iso.gz
```

### Build image yourself

Download the builder
```bash
curl -L -o builder_linux_amd64_deb.tar.gz https://github.com/kaliiiiiiiiii/LiveDebR/releases/download/{$tag}/builder_linux_amd64_deb.tar.gz
tar -xzvf builder_linux_amd64_deb.tar.gz
cd builder
```

Run debr
```bash
sudo ./debr deps
sudo ./debr config
sudo ./debr build
```

The builded .iso image file can then be found under `out/live`