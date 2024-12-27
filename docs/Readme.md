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
Default: `false`

## apt
*string* \
The package manager to use (Allowed values: `apt`, `aptitude`). \
Default: `apt`


## include
*list[string]* \
List of packages to preinstall.

## exclude
*list[string]* \
Blacklist of packages **not** to preinstall.
> **Warning**
> This currently is not implemented and won't have an effect.

## requires
*list[string]* \
List of paths (or [modules](Modules.md)) of configs to merge into. \
Example: \
```json
{
    "requires"["vsc", "extra_config.json"]
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
    "name":"microsoft-archive-keyring",
    "key":"https://packages.microsoft.com/keys/microsoft.asc",
    "src":"deb https://packages.microsoft.com/repos/code stable main",
    "add":["code"]
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
Wether to automatically install and configure [keyringer](https://github.com/kaliiiiiiiiii/LiveDebR/tree/main/keyringer), a package release key updater. \
Default: `true`


## deBootOpts
*string* \
Arguments to pass to [debootstrap](https://linux.die.net/man/8/debootstrap) \
Aequivalent to `debr lb config --debootstrap-options=...`

> **Note**
> `--include=apt-transport-https,ca-certificates,openssl` is parsed into the arguments automatically due to [issue](https://lists.debian.org/debian-live/2021/01/msg00012.html).