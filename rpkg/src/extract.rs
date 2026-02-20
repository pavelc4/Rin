use ar::Archive;
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tar::{Archive as TarArchive, EntryType};
use xz2::read::XzDecoder;
use zstd::stream::read::Decoder as ZstdDecoder;


pub fn extract_deb<R: Read>(reader: R, target_dir: &Path) -> anyhow::Result<Vec<String>> {
    let mut archive = Archive::new(reader);
    let mut installed_files = Vec::new();

    while let Some(entry_result) = archive.next_entry() {
        let entry = entry_result?;
        let identifier = String::from_utf8_lossy(entry.header().identifier()).to_string();

        if identifier.starts_with("data.tar") {
            let tar_reader: Box<dyn Read> = if identifier.ends_with(".xz") {
                Box::new(XzDecoder::new(entry))
            } else if identifier.ends_with(".zst") {
                Box::new(ZstdDecoder::new(entry)?)
            } else if identifier.ends_with(".gz") {
                Box::new(GzDecoder::new(entry))
            } else {
                Box::new(entry)
            };

            let mut tar = TarArchive::new(tar_reader);
            for file in tar.entries()? {
                let mut file = file?;
                
                let path = file.path()?.into_owned();
                let path_str = path.to_string_lossy().to_string();
                
                let dest_path = target_dir.join(&path);

                match file.header().entry_type() {
                    EntryType::Directory => {
                        fs::create_dir_all(&dest_path)?;
                    }
                    EntryType::Symlink => {
                        if let Some(link_name) = file.link_name()? {
                            if let Some(parent) = dest_path.parent() {
                                fs::create_dir_all(parent)?;
                            }
                            let _ = fs::remove_file(&dest_path);
                            std::os::unix::fs::symlink(link_name, &dest_path)?;
                            installed_files.push(path_str);
                        }
                    }
                    EntryType::Link => {
                        if let Some(link_name) = file.link_name()? {
                            if let Some(parent) = dest_path.parent() {
                                fs::create_dir_all(parent)?;
                            }
                            let dest_target = target_dir.join(link_name);
                            let _ = fs::remove_file(&dest_path);
                            fs::hard_link(&dest_target, &dest_path)?;
                            installed_files.push(path_str);
                        }
                    }
                    EntryType::Regular => {
                        if let Some(parent) = dest_path.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        
                        let _ = fs::remove_file(&dest_path);
                        let out_file = File::create(&dest_path)?;
                        let permissions = file.header().mode()?;
                        
                        let mut writer = BufWriter::with_capacity(64 * 1024, out_file);
                        io::copy(&mut file, &mut writer)?;
                        
                        let mut perms = fs::metadata(&dest_path)?.permissions();
                        perms.set_mode(permissions);
                        fs::set_permissions(&dest_path, perms)?;

                        installed_files.push(path_str);
                    }
                    _ => {
                        log::debug!("Skipping unsupported tar entry type for: {}", path_str);
                    }
                }
            }
            break;
        }
    }

    Ok(installed_files)
}
