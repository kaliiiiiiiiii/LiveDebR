# Usage

download builder
```bash
curl -L -o builder_linux_amd64_deb.tar.gz https://github.com/kaliiiiiiiiii/LiveDebR/releases/download/{$tag}/builder_linux_amd64_deb.tar.gz
tar -xzvf builder_linux_amd64_deb.tar.gz
cd builder
```

download build
```bash
curl -L -o live-image-amd64.hybrid.iso.gz https://github.com/kaliiiiiiiiii/LiveDebR/releases/download/{$tag}/live-image-amd64.hybrid.iso.gz
gunzip live-image-amd64.hybrid.iso.gz
```

run
```bash
sudo ./debr deps
```