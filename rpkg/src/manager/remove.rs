use crate::manager::PackageManager;
use colored::Colorize;
use std::fs;
use std::io::Write;

impl PackageManager {
    pub fn remove(&mut self, package_names: &[String]) -> anyhow::Result<()> {
        let mut to_remove = Vec::new();
        for name in package_names {
            if self.installed.contains_key(name) {
                to_remove.push(name.clone());
            } else {
                println!("{}: target not found: {}", "error".red().bold(), name);
            }
        }

        if to_remove.is_empty() {
            println!(" there is nothing to do");
            return Ok(());
        }

        println!("\nPackages ({})  {}", to_remove.len(), to_remove.join("  "));

        print!("\n{} ", ":: Proceed with removal? [Y/n]".blue().bold());
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().eq_ignore_ascii_case("n") {
            return Ok(());
        }

        println!("{}", ":: Removing packages...".blue().bold());
        for (i, name) in to_remove.iter().enumerate() {
            print!("({}/{}) removing {}... ", i + 1, to_remove.len(), name);
            std::io::stdout().flush().unwrap();

            if let Some(pkg) = self.installed.remove(name) {
                for file_path in &pkg.files {
                    let absolute_path = self.prefix.join(file_path);
                    if absolute_path.exists()
                        && (absolute_path.is_file() || absolute_path.is_symlink())
                    {
                        let _ = fs::remove_file(&absolute_path);
                    }
                }
            }
            println!("DONE");
        }

        self.save_database()?;
        println!(
            "\n{} Successfully removed {} packages.",
            "::".blue().bold(),
            to_remove.len()
        );
        Ok(())
    }
}
