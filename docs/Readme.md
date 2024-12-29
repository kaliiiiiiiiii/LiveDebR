Documents the config format

## arch
*string* \
The cpu architecture to to build for. \
Default: `amd64`

## dist
*string* \
The debian distribution to build \
Default: `bookworm`

## archiveAreas
*string* \
List of archive areas to use,seperated by whitespaces \
Default: `main contrib non-free non-free-firmware`

## recommends
*bool* \
Wether to automatically include recommended packages. \
Default: `true`
> **Warning** \
> Disabling this seems to make Debian crash (due to missing hardware drivers?).

## apt
*string* \
The package manager to use (Allowed values: `apt`, `aptitude`). \
Default: `aptitude`


## include
*list[string]* \
List of packages to preinstall.

## purge
*list[string]* \
List of packages to purge form the preinstalled ones. \

## snaps
*list[string]* \
[snap](https://snapcraft.io/docs/installing-snap-on-debian) packages to pre-install.

> **Warning** \
> Snaps are only installed after [`network-online.target`](https://www.freedesktop.org/wiki/Software/systemd/NetworkTarget/).
> If it for some reason failed to run, check if running `sudo /var/snap-download-cache/installer.sh` in the booted system works.

## requires
*list[string]* \
List of paths (or [modules](Modules.md)) of configs to merge into. \
Example: \
```json
{
    "requires"["gnome","dev", "extra_config.json"]
}
```

## eService
*list[string]* \
List of services to be enabled by default

## dService
*list[string]* \
List of services to be disabled by default

## extras
*list[[extra](#extra)]* \
List of extra **apt repositories** to include

### extra
*dict* \
Extra apt repository to include. Example:
```json
{
    "name":"google-chrome",
    "key":"https://dl.google.com/linux/linux_signing_key.pub",
    "src":"http://dl.google.com/linux/chrome/deb/ stable main",
    "add":["google-chrome-stable"]
}
```
**Keys** \
`name`: \
Name of the repository, has to be unique, but can be chosen \
`key` \
URL to the endpoint, which serves the [release key](https://wiki.debian.org/SecureApt). \
Known supported formats: `.pub`, `.asc` \
`src` \
URL to the repo \
`add` \
packages to install from

## keyringer
*bool* \
Wether to automatically install and configure (for [extra](#extra)) [keyringer](https://github.com/kaliiiiiiiiii/LiveDebR/tree/main/keyringer), a package release key updater. \
Default: `true`

## darkMode
*bool* \
Whether to change the theme to dark-mode. \
Default: `true`

## deBootOpts
*string* \
Arguments to pass to [debootstrap](https://linux.die.net/man/8/debootstrap) \
Aequivalent to `debr lb config --debootstrap-options=...`

> **Note**
> `--include=apt-transport-https,ca-certificates,openssl` is parsed into the arguments automatically due to [issue](https://lists.debian.org/debian-live/2021/01/msg00012.html).