use clap::{Parser, Subcommand};
use rpkg::manager::PackageManager;
use rpkg::DEFAULT_PREFIX;
use std::path::PathBuf;
use std::process::Command;
use std::os::unix::process::CommandExt;
use std::io::Read;

#[derive(Parser, Debug)]
#[command(name = "rpkg", version, about = "Rin Package Manager ")]
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


fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .init();

    let mut args = std::env::args();
    if let Some(arg0) = args.next() {
        let exe_path = PathBuf::from(&arg0);
        if let Some(exe_name) = exe_path.file_name().and_then(|s| s.to_str()) {
            if exe_name != "rpkg" && exe_name != "rpkg_cli" && exe_name != "librpkg_cli.so" {
                let original_path = if exe_path.parent().map_or(true, |p| p.as_os_str().is_empty()) || exe_path.parent().unwrap().as_os_str() == "." {
                    PathBuf::from(DEFAULT_PREFIX).join("usr").join("bin").join(exe_name)
                } else {
                    exe_path.clone()
                };
                
                let mut current = original_path.clone();
                while let Ok(target) = std::fs::read_link(&current) {
                    let next = if target.is_absolute() { target } else { current.parent().unwrap().join(target) };
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

                let mut is_elf = false;
                if let Ok(mut f) = std::fs::File::open(&target_elf) {
                    let mut magic = [0u8; 4];
                    if f.read_exact(&mut magic).is_ok() && &magic == b"\x7FELF" {
                        is_elf = true;
                    }
                }

                let mut interpreter = String::from("/system/bin/sh");
                let mut interpreter_args: Vec<String> = Vec::new();

                if !is_elf {
                    if let Ok(f) = std::fs::File::open(&target_elf) {
                        use std::io::{BufRead, BufReader};
                        let mut reader = BufReader::new(f);
                        let mut first_line = String::new();
                        if reader.read_line(&mut first_line).is_ok() {
                            let first_line = first_line.trim();
                            if first_line.starts_with("#!") {
                                let shebang = first_line[2..].trim();
                                let mut parts = shebang.split_whitespace();
                                if let Some(cmd) = parts.next() {
                                    if cmd.ends_with("/env") {
                                        if let Some(env_cmd) = parts.next() {
                                            interpreter = PathBuf::from(DEFAULT_PREFIX).join("usr/bin").join(env_cmd).to_string_lossy().into_owned();
                                            for p in parts {
                                                interpreter_args.push(p.to_string());
                                            }
                                        }
                                    } else if cmd == "/bin/sh" || cmd == "/system/bin/sh" {
                                        interpreter = String::from("/system/bin/sh");
                                        for p in parts {
                                            interpreter_args.push(p.to_string());
                                        }
                                    } else {
                                        let cmd_path = std::path::Path::new(cmd);
                                        if let Some(name) = cmd_path.file_name() {
                                            interpreter = PathBuf::from(DEFAULT_PREFIX).join("usr/bin").join(name).to_string_lossy().into_owned();
                                        }
                                        for p in parts {
                                            interpreter_args.push(p.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                let lib_path = PathBuf::from(DEFAULT_PREFIX).join("usr").join("lib");
                let err = if is_elf {
                    Command::new("/system/bin/linker64")
                        .arg(&target_elf)
                        .args(multicall_args)
                        .args(args)
                        .env("LD_LIBRARY_PATH", &lib_path)
                        .exec()
                } else {
                    let mut cmd = Command::new(&interpreter);
                    cmd.args(interpreter_args);
                    cmd.arg(&target_elf);
                    cmd.args(args);
                    cmd.env("LD_LIBRARY_PATH", &lib_path);
                    cmd.exec()
                };
                    
                eprintln!("rpkg proxy: failed to exec {}: {}", target_elf.display(), err);
                std::process::exit(1);
            }
        }
    }

    let cli = Cli::parse();
    let mut pm = PackageManager::new(&cli.prefix)?;

    if cli.sync {
        if cli.refresh {
            pm.sync()?;
        }

        if cli.search {
            for query in &cli.targets {
                let results = pm.search(query)?;
                for pkg in results {
                    println!("{} {} ({})", pkg.name, pkg.version, pkg.size);
                    println!("    {}", pkg.description);
                }
            }
            return Ok(());
        }

        if cli.sysupgrade {
            pm.upgrade()?;
        }

        for pkg in &cli.targets {
            pm.install(pkg, cli.force)?;
        }
    } else if cli.remove {
        for pkg in &cli.targets {
            pm.remove(pkg)?;
        }
    } else if cli.query {
        let installed = pm.list_installed();
        for pkg in installed {
            println!("{} {}", pkg.info.name, pkg.info.version);
        }
    } else {
        println!("No operation specified (use -S, -R, or -Q)");
    }

    Ok(())
}
