mod cli;
mod proxy;

use clap::Parser;
use cli::Cli;
use colored::Colorize;
use rpkg::manager::PackageManager;



fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .init();

    proxy::handle_multicall();

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
        let installed = pm.list_installed();
        for pkg in installed {
            let name_styled = pkg.info.name.bold();
            let ver_styled = pkg.info.version.green().bold();
            println!("{} {}", name_styled, ver_styled);
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
