# Usage
You can download a prebuilt image from [mega]({MEGAURL}) and then run
```bash
gunzip -k -c $HOME/Downloads/live-image-amd64-*.hybrid.iso.gz > live-image-amd64.hybrid.iso
```

### Build image yourself

Download the builder
```bash
curl -L -o builder_linux_amd64_deb.tar.gz https://github.com/kaliiiiiiiiii/LiveDebR/releases/download/{tag}/builder_linux_amd64_deb.tar.gz
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