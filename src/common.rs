use directories::BaseDirs;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Paths {
    pub codex: PathBuf,
    pub auth: PathBuf,
    pub profiles: PathBuf,
    pub usage: PathBuf,
    pub usage_lock: PathBuf,
    pub labels: PathBuf,
}

pub fn command_name() -> &'static str {
    static COMMAND_NAME: OnceLock<String> = OnceLock::new();
    COMMAND_NAME
        .get_or_init(|| {
            if let Ok(value) = env::var("CODEX_PROFILES_COMMAND") {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
            env::args_os()
                .next()
                .and_then(|arg| {
                    Path::new(&arg)
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| name.to_string())
                })
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| "codex-profiles".to_string())
        })
        .as_str()
}

pub fn package_command_name() -> &'static str {
    "codex-profiles"
}

pub fn resolve_paths() -> Result<Paths, String> {
    let home_dir =
        resolve_home_dir().ok_or_else(|| "Error: could not resolve home directory".to_string())?;
    let codex_dir = home_dir.join(".codex");
    let auth = codex_dir.join("auth.json");
    let profiles = codex_dir.join("profiles");
    let usage = profiles.join("usage.tsv");
    let usage_lock = profiles.join("usage.lock");
    let labels = profiles.join("labels.json");
    Ok(Paths {
        codex: codex_dir,
        auth,
        profiles,
        usage,
        usage_lock,
        labels,
    })
}

fn resolve_home_dir() -> Option<PathBuf> {
    if let Some(path) = env::var_os("CODEX_PROFILES_HOME") {
        let path = PathBuf::from(path);
        if !path.as_os_str().is_empty() {
            return Some(path);
        }
    }
    if let Some(base_dirs) = BaseDirs::new() {
        return Some(base_dirs.home_dir().to_path_buf());
    }
    if let Some(path) = env::var_os("HOME") {
        let path = PathBuf::from(path);
        if !path.as_os_str().is_empty() {
            return Some(path);
        }
    }
    if let Some(path) = env::var_os("USERPROFILE") {
        let path = PathBuf::from(path);
        if !path.as_os_str().is_empty() {
            return Some(path);
        }
    }
    match (env::var_os("HOMEDRIVE"), env::var_os("HOMEPATH")) {
        (Some(drive), Some(path)) => {
            let mut out = PathBuf::from(drive);
            out.push(path);
            if out.as_os_str().is_empty() {
                None
            } else {
                Some(out)
            }
        }
        _ => None,
    }
}

pub fn ensure_paths(paths: &Paths) -> Result<(), String> {
    if paths.profiles.exists() && !paths.profiles.is_dir() {
        return Err(format!(
            "Error: {} exists and is not a directory",
            paths.profiles.display()
        ));
    }

    fs::create_dir_all(&paths.profiles).map_err(|err| {
        format!(
            "Error: cannot create profiles directory {}: {err}",
            paths.profiles.display()
        )
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o700);
        if let Err(err) = fs::set_permissions(&paths.profiles, perms) {
            return Err(format!(
                "Error: cannot set permissions on {}: {err}",
                paths.profiles.display()
            ));
        }
    }

    ensure_file_or_absent(&paths.usage)?;
    ensure_file_or_absent(&paths.labels)?;

    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&paths.usage)
        .map_err(|err| {
            format!(
                "Error: cannot write usage file {}: {err}",
                paths.usage.display()
            )
        })?;

    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&paths.usage_lock)
        .map_err(|err| {
            format!(
                "Error: cannot write usage lock file {}: {err}",
                paths.usage_lock.display()
            )
        })?;

    Ok(())
}

pub fn write_atomic(path: &Path, contents: &[u8]) -> Result<(), String> {
    let permissions = fs::metadata(path).ok().map(|meta| meta.permissions());
    write_atomic_with_permissions(path, contents, permissions)
}

pub fn write_atomic_with_mode(path: &Path, contents: &[u8], mode: u32) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = fs::Permissions::from_mode(mode);
        write_atomic_with_permissions(path, contents, Some(permissions))
    }
    #[cfg(not(unix))]
    {
        let _ = mode;
        write_atomic_with_permissions(path, contents, None)
    }
}

fn write_atomic_with_permissions(
    path: &Path,
    contents: &[u8],
    permissions: Option<fs::Permissions>,
) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| {
        format!(
            "Error: cannot resolve parent directory for {}",
            path.display()
        )
    })?;
    if !parent.as_os_str().is_empty() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Error: cannot create directory {}: {err}", parent.display()))?;
    }

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("Error: invalid file name {}", path.display()))?;
    let pid = std::process::id();
    let mut attempt = 0u32;
    loop {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("Error: failed to get time: {err}"))?
            .as_nanos();
        let tmp_name = format!(".{file_name}.tmp-{pid}-{nanos}-{attempt}");
        let tmp_path = parent.join(tmp_name);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        if let Some(permissions) = permissions.as_ref() {
            use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
            options.mode(permissions.mode());
        }
        let mut tmp_file = match options.open(&tmp_path) {
            Ok(file) => file,
            Err(err) => {
                attempt += 1;
                if attempt < 5 {
                    continue;
                }
                return Err(format!(
                    "Error: failed to create temp file for {}: {err}",
                    path.display()
                ));
            }
        };

        tmp_file.write_all(contents).map_err(|err| {
            format!(
                "Error: failed to write temp file for {}: {err}",
                path.display()
            )
        })?;

        if let Some(permissions) = permissions {
            fs::set_permissions(&tmp_path, permissions).map_err(|err| {
                format!(
                    "Error: failed to set temp file permissions for {}: {err}",
                    path.display()
                )
            })?;
        }

        tmp_file.sync_all().map_err(|err| {
            format!(
                "Error: failed to write temp file for {}: {err}",
                path.display()
            )
        })?;

        match fs::rename(&tmp_path, path) {
            Ok(()) => return Ok(()),
            Err(err) => {
                #[cfg(windows)]
                {
                    if path.exists() {
                        let _ = fs::remove_file(path);
                    }
                    if fs::rename(&tmp_path, path).is_ok() {
                        return Ok(());
                    }
                }
                let _ = fs::remove_file(&tmp_path);
                return Err(format!(
                    "Error: failed to replace {}: {err}",
                    path.display()
                ));
            }
        }
    }
}

pub fn copy_atomic(source: &Path, dest: &Path) -> Result<(), String> {
    let permissions = fs::metadata(source)
        .map_err(|err| {
            format!(
                "Error: failed to read metadata for {}: {err}",
                source.display()
            )
        })?
        .permissions();
    let contents = fs::read(source)
        .map_err(|err| format!("Error: failed to read {}: {err}", source.display()))?;
    write_atomic_with_permissions(dest, &contents, Some(permissions))
}

fn ensure_file_or_absent(path: &Path) -> Result<(), String> {
    if path.exists() && !path.is_file() {
        return Err(format!(
            "Error: {} exists and is not a file",
            path.display()
        ));
    }
    Ok(())
}
