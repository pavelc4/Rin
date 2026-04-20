use clap::Parser;
use colored::Colorize;
use rpkg::manager::PackageManager;
use rpkg::DEFAULT_PREFIX;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;


#[derive(Parser, Debug)]
#[command(name = "rpkg", version, about = "Rin Package Manager")]
struct Cli {
    #[arg(long, default_value = DEFAULT_PREFIX)]
    prefix: PathBuf,

    #[arg(short = 'S', long)]
    sync: bool,

    #[arg(short = 'R', long)]
    remove: bool,

    #[arg(short = 'Q', long)]
    query: bool,

    #[arg(short = 'y', long)]
    refresh: bool,

    #[arg(short = 'u', long)]
    sysupgrade: bool,

    #[arg(short = 's', long)]
    search: bool,

    #[arg(short = 'f', long)]
    force: bool,

    targets: Vec<String>,
}


fn handle_multicall() {
    let mut args = std::env::args();
    if let Some(arg0) = args.next() {
        let exe_path = PathBuf::from(&arg0);
        if let Some(exe_name) = exe_path.file_name().and_then(|s| s.to_str()) {
            if exe_name != "rpkg" && exe_name != "rpkg_cli" && exe_name != "librpkg_cli.so" {
                execute_proxied_binary(&exe_path, exe_name, args);
            }
        }
    }
}

fn detect_elf(path: &Path) -> bool {
    std::fs::File::open(path)
        .ok()
        .and_then(|mut f| {
            let mut magic = [0u8; 4];
            f.read_exact(&mut magic).ok().map(|_| magic == *b"\x7FELF")
        })
        .unwrap_or(false)
}

fn resolve_interpreter(target_elf: &Path) -> (String, Vec<String>) {
    use std::io::{BufRead, BufReader};

    let default = (String::from("/system/bin/sh"), Vec::new());

    let f = match std::fs::File::open(target_elf) {
        Ok(f) => f,
        Err(_) => return default,
    };

    let mut reader = BufReader::new(f);
    let mut first_line = String::new();
    if reader.read_line(&mut first_line).is_err() {
        return default;
    }

    let first_line = first_line.trim();
    if !first_line.starts_with("#!") {
        return default;
    }

    let shebang = first_line[2..].trim();
    let mut parts = shebang.split_whitespace();
    let cmd = match parts.next() {
        Some(c) => c,
        None => return default,
    };

    let interpreter_args: Vec<String> = parts.map(|p| p.to_string()).collect();

    let interpreter = if cmd.ends_with("/env") {
        match interpreter_args.first() {
            Some(env_cmd) => PathBuf::from(DEFAULT_PREFIX)
                .join("usr/bin")
                .join(env_cmd)
                .to_string_lossy()
                .into_owned(),
            None => return default,
        }
    } else if cmd == "/bin/sh" || cmd == "/system/bin/sh" {
        String::from("/system/bin/sh")
    } else {
        let name = std::path::Path::new(cmd)
            .file_name()
            .unwrap_or(std::ffi::OsStr::new(cmd));
        PathBuf::from(DEFAULT_PREFIX)
            .join("usr/bin")
            .join(name)
            .to_string_lossy()
            .into_owned()
    };

    let args = if cmd.ends_with("/env") {
        interpreter_args.into_iter().skip(1).collect()
    } else {
        interpreter_args
    };

    (interpreter, args)
}

fn execute_proxied_binary(exe_path: &Path, exe_name: &str, args: std::env::Args) -> ! {
    let original_path = if exe_path.parent().map_or(true, |p| p.as_os_str().is_empty())
        || exe_path.parent().unwrap().as_os_str() == "."
    {
        PathBuf::from(DEFAULT_PREFIX)
            .join("usr")
            .join("bin")
            .join(exe_name)
    } else {
        exe_path.to_path_buf()
    };

    let mut current = original_path.clone();
    while let Ok(target) = std::fs::read_link(&current) {
        let next = if target.is_absolute() {
            target
        } else {
            current.parent().unwrap().join(target)
        };
        if next.file_name().and_then(|n| n.to_str()) == Some("rpkg") {
            break;
        }
        current = next;
    }

    let target_elf = PathBuf::from(format!("{}.elf", current.display()));
    let resolved_name = current.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let mut multicall_args = Vec::new();
    if resolved_name != exe_name {
        if resolved_name == "coreutils" {
            multicall_args.push(format!("--coreutils-prog={}", exe_name));
        } else if resolved_name == "busybox" || resolved_name == "toybox" {
            multicall_args.push(exe_name.to_string());
        }
    }

    let is_elf = detect_elf(&target_elf);
    let lib_path = PathBuf::from(DEFAULT_PREFIX).join("usr").join("lib");

    let err = if is_elf {
        Command::new("/system/bin/linker64")
            .arg(&target_elf)
            .args(multicall_args)
            .args(args)
            .env("LD_LIBRARY_PATH", &lib_path)
            .exec()
    } else {
        let (interpreter, interpreter_args) = resolve_interpreter(&target_elf);
        let mut cmd = Command::new(&interpreter);
        cmd.args(interpreter_args);
        cmd.arg(&target_elf);
        cmd.args(args);
        cmd.env("LD_LIBRARY_PATH", &lib_path);
        cmd.exec()
    };

    eprintln!(
        "rpkg proxy: failed to exec {}: {}",
        target_elf.display(),
        err
    );
    std::process::exit(1);
}


fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .init();

    handle_multicall();

    let cli = Cli::parse();
    let mut pm = PackageManager::new(&cli.prefix)?;

    if cli.sync {
        if cli.refresh {
            pm.sync()?;
        }

        if cli.search {
            for query in &cli.targets {
                let results = pm.search(query)?;
                if results.is_empty() {
                    continue;
                }
                for pkg in &results {
                    let name_styled = pkg.name.bold();
                    let ver_styled = pkg.version.green().bold();
                    let installed_tag =
                        if pm.list_installed().iter().any(|i| i.info.name == pkg.name) {
                            " [installed]".cyan().bold()
                        } else {
                            "".normal()
                        };
                    println!("rin/{} {} {}", name_styled, ver_styled, installed_tag);
                    println!("    {}", pkg.description);
                }
            }
            return Ok(());
        }

        if cli.sysupgrade {
            pm.upgrade()?;
        }

        if !cli.targets.is_empty() {
            pm.install(&cli.targets, cli.force)?;
        }
    } else if cli.remove {
        if !cli.targets.is_empty() {
            pm.remove(&cli.targets)?;
        }
    } else if cli.query {
        for pkg in pm.list_installed() {
            println!("{} {}", pkg.info.name.bold(), pkg.info.version.green().bold());
        }
    } else {
        println!(
            "{}",
            "error: no operation specified (use -h for help)"
                .red()
                .bold()
        );
    }

    Ok(())
}
