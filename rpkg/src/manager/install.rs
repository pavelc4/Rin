use crate::core::extract::extract_deb;
use crate::core::index::PackageIndex;
use crate::core::resolver::Resolver;
use crate::core::types::InstalledPackage;
use crate::manager::PackageManager;
use colored::Colorize;
use std::collections::HashSet;
use std::io::Write;

impl PackageManager {
    pub fn install(&mut self, package_names: &[String], force: bool) -> anyhow::Result<()> {
        println!("{}", ":: Resolving dependencies...".blue().bold());

        let index = PackageIndex::from_cache(&self.index_path()).map_err(|e| {
            anyhow::anyhow!(
                "Failed to read package index. Did you run sync? Error: {}",
                e
            )
        })?;

        let installed_set: HashSet<String> = if force {
            HashSet::new()
        } else {
            self.installed.keys().cloned().collect()
        };
        let resolver = Resolver::new(&index, installed_set);

        let mut to_install = Vec::new();
        for package_name in package_names {
            let reqs = resolver.resolve(package_name)?;
            for req in reqs {
                if !to_install
                    .iter()
                    .any(|p: &crate::core::types::PackageInfo| p.name == req.name)
                {
                    to_install.push(req);
                }
            }
        }

        if to_install.is_empty() {
            println!(" there is nothing to do");
            return Ok(());
        }

        let mut pkg_strings = Vec::new();
        let mut total_download_size: u64 = 0;
        let mut total_installed_size: u64 = 0;

        for pkg in &to_install {
            pkg_strings.push(format!("{}-{}", pkg.name, pkg.version));
            total_download_size += pkg.size;
            total_installed_size += pkg.installed_size;
        }

        println!(
            "\nPackages ({})  {}",
            to_install.len(),
            pkg_strings.join("  ")
        );
        println!(
            "\nTotal Download Size:   {:.2} MiB",
            total_download_size as f64 / 1048576.0
        );
        println!(
            "Total Installed Size:  {:.2} MiB",
            total_installed_size as f64 / 1048576.0
        );

        print!("\n{} ", ":: Proceed with installation? [Y/n]".blue().bold());
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().eq_ignore_ascii_case("n") {
            return Ok(());
        }

        println!("{}", ":: Retrieving packages...".blue().bold());

        let mut downloaded_files = Vec::new();
        for pkg in &to_install {
            let url = format!("{}/{}", self.repo.url, pkg.filename);
            let rsp = ureq::get(&url).call()?;

            let mut reader = rsp.into_body().into_reader();
            let mut buffer = Vec::new();
            let mut chunk = vec![0; 8192];

            let total_size = pkg.size;
            let mut downloaded: u64 = 0;
            let start_time = std::time::Instant::now();
            let mut last_print = std::time::Instant::now();

            let mut name_ver = format!("{}-{}", pkg.name, pkg.version);
            if name_ver.len() > 18 {
                name_ver.truncate(15);
                name_ver.push_str("...");
            }

            loop {
                use std::io::Read;
                let n = reader.read(&mut chunk)?;
                if n == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..n]);
                downloaded += n as u64;

                let now = std::time::Instant::now();
                if now.duration_since(last_print).as_millis() > 100 {
                    let percent = if total_size > 0 {
                        (downloaded as f64 / total_size as f64) * 100.0
                    } else {
                        100.0
                    };

                    let elapsed = start_time.elapsed().as_secs_f64();
                    let speed_kb = if elapsed > 0.0 {
                        (downloaded as f64 / 1024.0) / elapsed
                    } else {
                        0.0
                    };

                    let bar_len = 20;
                    let filled_len = if total_size > 0 {
                        (bar_len as f64 * (downloaded as f64 / total_size as f64)) as usize
                    } else {
                        bar_len
                    };

                    let mut bar = String::with_capacity(bar_len);
                    for i in 0..bar_len {
                        if i < filled_len {
                            bar.push('#');
                        } else if i == filled_len {
                            bar.push('C');
                        } else {
                            bar.push('-');
                        }
                    }

                    print!(
                        "\x1b[2K\r{:<18} {:>5.1} MiB {:>6.1} KiB/s [{}] {:>3.0}%",
                        name_ver,
                        downloaded as f64 / 1048576.0,
                        speed_kb,
                        bar.cyan(),
                        percent
                    );
                    std::io::stdout().flush().unwrap();
                    last_print = now;
                }
            }

            let final_speed_kb = if start_time.elapsed().as_secs_f64() > 0.0 {
                (total_size as f64 / 1024.0) / start_time.elapsed().as_secs_f64()
            } else {
                0.0
            };
            print!(
                "\x1b[2K\r{:<18} {:>5.1} MiB {:>6.1} KiB/s [{}] 100%\n",
                name_ver,
                total_size as f64 / 1048576.0,
                final_speed_kb,
                "#".repeat(20).cyan(),
            );
            std::io::stdout().flush().unwrap();

            downloaded_files.push((pkg.clone(), buffer));
        }

        println!("{}", ":: Executing package hooks...".blue().bold());
        for (i, (pkg, buffer)) in downloaded_files.into_iter().enumerate() {
            print!(
                "({}/{}) installing {}... ",
                i + 1,
                to_install.len(),
                pkg.name
            );
            std::io::stdout().flush().unwrap();

            let cursor = std::io::Cursor::new(buffer);
            let installed_files = extract_deb(cursor, &self.prefix)?;

            let installed_pkg = InstalledPackage {
                info: pkg.clone(),
                files: installed_files,
                install_time: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs(),
                explicit: package_names.contains(&pkg.name),
                required_by: vec![],
            };

            self.installed.insert(pkg.name.clone(), installed_pkg);
            self.save_database()?;
            println!("DONE");
        }

        println!(
            "\n{} Successfully installed {} packages.",
            "::".blue().bold(),
            to_install.len()
        );
        Ok(())
    }
}
