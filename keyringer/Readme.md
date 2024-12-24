# Usage
installs automatically and creates a service, that runs on startup and daily


#### configuration
place a configuration at `/etc/keyringer/keyrings.json`
```json
{
    "microsoft-archive-keyring": "https://packages.microsoft.com/keys/microsoft.asc",
    "google-chrome":"https://dl.google.com/linux/linux_signing_key.pub"
}
```

the files then will be placed in `/etc/apt/keyrings" (here: `microsoft-archive-keyring.gpg` and  `google-chrome.gpg`)

#### uninstallation
uninstalls keyringer, keyrings and keyrings.json

```bash
keyringer uninstall
```