use std::collections::HashSet;
use std::fs::{create_dir_all,File, metadata, set_permissions};
use std::os::unix::fs::PermissionsExt;
use std::io::{self, Write};
use std::path::Path;
use ordermap::OrderSet;

const DEF_SCRIPT: &str = "#!/bin/bash\n\
echo \"I: running $0\"\n\n\
set -e\n";

const AS_USER: &str = r#"#!/bin/bash

function as-user() {
        _display_id=":$(find /tmp/.X11-unix/* | sed 's#/tmp/.X11-unix/X##' | head -n 1)"
        _username="user"
        _user_id=$(id -u "$_username")
        _environment=("DISPLAY=$_display_id" "DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/$_user_id/bus")
        sudo -Hu "$_username" "${_environment[@]}" "$@"
    }
"#;

pub fn services(e_service: &HashSet<String>, d_service: &HashSet<String>) -> std::io::Result<String> {
    // https://github.com/nodiscc/debian-live-config/blob/55677bbd1d8fcfe522f090fb0d77bb1e16027f1d/config/hooks/normal/0350-update-default-services-status.hook.chroot
    let mut script = String::from(DEF_SCRIPT);

    let gen_for_loop = |services: &HashSet<String>, action: &str| {
        if services.is_empty() {
            return String::new();
        }

        let services_str = escape_to_list(&services);
        let loop_action = if action == "Disabling" { "disable" } else { "enable" };
        let systemctl_action = if action == "Disabling" { "stop" } else { "start" };

        let mut loop_script = String::new();
        loop_script.push_str(&format!("for service in {}; do\n", services_str));
        loop_script.push_str(&format!("    echo \"{} $service\"\n", action));
        loop_script.push_str(&format!("    systemctl {} \"$service\" || true\n", loop_action));
        loop_script.push_str(&format!("    systemctl {} \"$service\" || true\n", systemctl_action));
        loop_script.push_str("done\n");

        loop_script
    };

    if !d_service.is_empty() {
        script.push_str(&gen_for_loop(d_service, "Disabling"));
    }
    if !e_service.is_empty() {
        if !d_service.is_empty() { script.push_str("\n"); }  // Add a newline between blocks if both are present
        script.push_str(&gen_for_loop(e_service, "Enabling"));
    }

    Ok(script)
}

pub fn apt_install(packages: &HashSet<String>, apt:&str) -> std::io::Result<String> {
    let mut script = String::from(DEF_SCRIPT);
    script.push_str("mv /tmp/apt-keyrings-cache-debr/*.gpg /etc/apt/keyrings/\n");
    script.push_str("rm -rf /tmp/apt-keyrings-cache-debr/\n");
    script.push_str(&format!("{} update\n\n", apt));
    

    let packages_str = escape_to_list(&packages);

    script.push_str(&format!(
        "echo \"Installing packages: {}\"\n",
        packages_str.replace("\"", "")
    ));
    let mut no_recommends = " --no-install-recommends";
    if apt == "aptitude"{
        no_recommends = "";
    }
    script.push_str(&format!(
        "DEBIAN_FRONTEND=noninteractive {} install -y{} {}\n",
        apt,no_recommends, packages_str
    ));
    script.push_str("\n");
    script.push_str("echo \"Packages installed successfully.\"\n");

    Ok(script)
}

pub fn snap_install_from(packages: &OrderSet<String>, temp_path: &str) -> io::Result<String> {
    let mut script = String::from(DEF_SCRIPT);

    let packages_str = packages.iter()
        .map(|p| format!("\"{}\"", p.replace("\"", "\\\""))) // Escape quotes
        .collect::<Vec<String>>()
        .join(" ");
    script.push_str("sleep 1\n");
    script.push_str(&format!("snap_cache=\"{}\"\n", temp_path));
    script.push_str(&format!("for package in {}; do\n", packages_str));
    script.push_str("    set +e");
    script.push_str("    echo \"Attempting to ack $package\"\n");
    script.push_str("    snap ack \"$snap_cache/$package.assert\"\n");
    script.push_str("    echo \"Attempting to install $package\"\n");
    script.push_str("    snap install --classic \"$snap_cache/$package.snap\"\n");
    script.push_str("    set -e");
    script.push_str("    rm -f \"$snap_cache/$package.snap\"\n");
    script.push_str("    rm -f \"$snap_cache/$package.assert\"\n");
    script.push_str("done\n");
    script.push_str("rm -rf \"$snap_cache\"\n");
    script.push_str("rm /etc/systemd/system/snapd_installer.service\n");
    script.push_str("systemctl disable \"snapd_installer.service\"");

    Ok(script)
}

pub fn apt_purge(packages: &HashSet<String>) -> io::Result<String> {
    let mut script = String::from(DEF_SCRIPT);

    let packages_str = escape_to_list(packages);

    script.push_str(&format!("for package in {}; do\n", packages_str));
    script.push_str("    set +e\n");
    script.push_str("    apt purge --autoremove -y \"$package\"\n");
    script.push_str("    exit_code=$?\n");
    script.push_str("    set -e\n");
    script.push_str("    if [ $exit_code -eq 100 ]; then\n");
    script.push_str("        continue\n");
    script.push_str("    elif [ $exit_code -ne 0 ]; then\n");
    script.push_str("        echo \"Error: apt purge failed with exit code $exit_code\"\n");
    script.push_str("        exit $exit_code\n");
    script.push_str("    fi\n");
    script.push_str("done\n");

    script.push_str("echo \"Purge complete\"\n");

    Ok(script)
}


pub fn gnome_set_dark() -> io::Result<String> {
    let mut script = String::from(AS_USER);
    script.push_str("set +e\n\n");
    script.push_str("sleep 1\n");
    script.push_str("as-user  gsettings set org.gnome.desktop.interface color-scheme 'prefer-dark'\n");
    script.push_str("as-user  gsettings set org.gnome.desktop.interface gtk-theme 'Adwaita-dark'\n");
    script.push_str("as-user  gsettings set org.gnome.desktop.wm.preferences button-layout ':minimize,maximize,close'\n");
    script.push_str("as-user  gsettings set org.gnome.shell favorite-apps \"['code_code.desktop', 'google-chrome.desktop', 'org.gnome.Terminal.desktop']\"\n");
    script.push_str("set -e\n");
    script.push_str("rm /etc/systemd/system/apply_gnome_settings.service\n");
    script.push_str("rm /lib/debr_util_scripts/apply_gnome_settings.sh");
    script.push_str("systemctl disable \"apply_gnome_settings.service\"");
    Ok(script)
}

pub fn add_hook(name: &str, content: &String, live_dir: &Path, at_boot: bool) -> std::io::Result<()> {
    let hook_dir = if at_boot {
        live_dir.join("config/includes.chroot_after_packages/lib/live/config/")  // Boot-time hooks
    } else {
        live_dir.join("config/hooks/normal/")  // Chroot hooks
    };


    create_dir_all(&hook_dir)?;
    let hook_path = hook_dir.join(name);
    let mut hook_file = File::create(&hook_path)?;
    hook_file.write_all(content.as_bytes())?;

    if at_boot{chmod_x(hook_path)?}

    Ok(())
}


fn escape_to_list(set: &HashSet<String>) -> String {
    let escaped = set
        .iter()
        .map(|p| format!("\"{}\"", p.replace("\"", "\\\""))) // Escape quotes
        .collect::<Vec<String>>()
        .join(" ");
    return escaped;
}

pub fn chmod_x<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let metadata = metadata(&path)?;
    let mut permissions = metadata.permissions();
    
    let mode = permissions.mode() | 0o111;
    
    permissions.set_mode(mode);
    set_permissions(path, permissions)?;
    Ok(())
}

pub fn logger_wrap(script: &str) -> String {
    let mut wrapped_script = String::new();

    wrapped_script.push_str("#!/bin/bash\n");
    wrapped_script.push_str("LOG_FILE=/tmp/debr_boot_script.log\n");
    wrapped_script.push_str("set -e \n");
    wrapped_script.push_str("exec &>> $LOG_FILE\n");
    wrapped_script.push_str("echo \"I: running $0\"\n\n");

    // Append the original script
    wrapped_script.push_str(script);

    wrapped_script.push_str("\n");

    // Final message indicating script execution completion
    wrapped_script.push_str("echo \"Script completed successfully.\" >> $LOG_FILE\n");

    wrapped_script
}