use ar::Archive;
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::{BufWriter, Read};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tar::{Archive as TarArchive, EntryType};
use xz2::read::XzDecoder;
use zstd::stream::read::Decoder as ZstdDecoder;

const PKG_EMBEDDED_PREFIX: &str = "data/data/com.termux/files/";
const PKG_ABS_SEARCH:  &[u8] = b"/data/data/com.termux/files";
const PKG_ABS_REPLACE: &[u8] = b"/data/data/com.rin////files";
const _: () = assert!(PKG_ABS_SEARCH.len() == PKG_ABS_REPLACE.len(), "binary patch strings must be the same byte length");


fn strip_upstream(raw: &str) -> Option<&str> {
    let normalized = raw.trim_start_matches("./");

    if let Some(stripped) = normalized.strip_prefix(PKG_EMBEDDED_PREFIX) {
        if stripped.is_empty() {
            return None;
        }
        Some(stripped)
    } else if normalized.starts_with("data/") || normalized.is_empty() || normalized == "." {
        None
    } else {
        Some(normalized)
    }
}

fn patch_content(content: &[u8]) -> Vec<u8> {
    let mut out = content.to_vec();
    
    if out.windows(PKG_ABS_SEARCH.len()).any(|w| w == PKG_ABS_SEARCH) {
        let mut temp = Vec::with_capacity(out.len());
        let mut cur = &out[..];
        while let Some(pos) = cur.windows(PKG_ABS_SEARCH.len())
            .position(|w| w == PKG_ABS_SEARCH)
        {
            temp.extend_from_slice(&cur[..pos]);
            temp.extend_from_slice(PKG_ABS_REPLACE);
            cur = &cur[pos + PKG_ABS_SEARCH.len()..];
        }
        temp.extend_from_slice(cur);
        out = temp;
    }

    if out.starts_with(b"\x7FELF") {
        if let Ok(elf) = goblin::elf::Elf::parse(&out) {
            if let Some(interp) = elf.interpreter {
                if interp.contains("com.termux") || interp.contains("com.rin") {
                    let is_64bit = elf.is_64;
                    let interp_owned = interp.as_bytes().to_vec();
                    let interp_len = interp_owned.len();
                    drop(elf);

                    let system_linker: &[u8] = if is_64bit {
                        b"/system/bin/linker64\0"
                    } else {
                        b"/system/bin/linker\0"
                    };

                    if let Some(pos) = out.windows(interp_len).position(|w| w == interp_owned) {
                        if system_linker.len() <= interp_len + 1 {
                            for (i, &b) in system_linker.iter().enumerate() {
                                out[pos + i] = b;
                            }
                            for i in system_linker.len()..interp_len + 1 {
                                if pos + i < out.len() {
                                    out[pos + i] = 0;
                                }
                            }
                            log::debug!("Patched ELF interpreter to {:?}", std::str::from_utf8(system_linker).unwrap_or(""));
                        }
                    }
                }
            }
        }
    }

    out
}

fn clean_link_target(link: &Path) -> PathBuf {
    let s = link.to_string_lossy();
    let stripped = s.trim_start_matches('/');
    if let Some(rel) = stripped.strip_prefix(PKG_EMBEDDED_PREFIX) {
        PathBuf::from(rel)
    } else {
        link.to_path_buf()
    }
}

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
            for file_res in tar.entries()? {
                let mut file = file_res?;

                let raw_path = file.path()?.into_owned();
                let raw_str = raw_path.to_string_lossy();

                let clean_str = match strip_upstream(&raw_str) {
                    Some(s) => s.to_owned(),
                    None => {
                        log::debug!("Skipping upstream-only entry: {}", raw_str);
                        continue;
                    }
                };

                let dest_path = target_dir.join(&clean_str);

                match file.header().entry_type() {
                    EntryType::Directory => {
                        fs::create_dir_all(&dest_path)?;
                    }
                    EntryType::Symlink => {
                        if let Some(link_target) = file.link_name()? {
                            if let Some(parent) = dest_path.parent() {
                                fs::create_dir_all(parent)?;
                            }
                            let cleaned_target = clean_link_target(&link_target);
                            let _ = fs::remove_file(&dest_path);
                            let final_target = if link_target.is_absolute() {
                                target_dir.join(&cleaned_target)
                            } else {
                                cleaned_target
                            };
                            std::os::unix::fs::symlink(&final_target, &dest_path)?;
                            installed_files.push(clean_str);
                        }
                    }
                    EntryType::Link => {
                        if let Some(link_target) = file.link_name()? {
                            if let Some(parent) = dest_path.parent() {
                                fs::create_dir_all(parent)?;
                            }
                            let cleaned_target = clean_link_target(&link_target);
                            let abs_target = if link_target.is_absolute() {
                                target_dir.join(&cleaned_target)
                            } else {
                                target_dir.join(&cleaned_target)
                            };
                            let _ = fs::remove_file(&dest_path);
                            if abs_target.exists() {
                                if fs::hard_link(&abs_target, &dest_path).is_err() {
                                    fs::copy(&abs_target, &dest_path)?;
                                }
                            } else {
                                log::debug!("HardLink source missing, skipping: {}", abs_target.display());
                            }
                            installed_files.push(clean_str);
                        }
                    }
                    EntryType::Regular => {
                        if let Some(parent) = dest_path.parent() {
                            fs::create_dir_all(parent)?;
                        }

                        let permissions = file.header().mode()?;
                        let is_executable = (permissions & 0o111) != 0;

                        let mut content = Vec::new();
                        file.read_to_end(&mut content)?;
                        
                        let is_elf = content.starts_with(b"\x7FELF");
                        let patched = patch_content(&content);

                        let dest_str = dest_path.to_string_lossy();
                        let is_library = dest_str.contains("/usr/lib/") || dest_str.contains("/lib/") || dest_str.contains(".so");

                        if is_executable && !is_library {
                            let elf_dest_path = dest_path.with_extension("elf");
                            let _ = fs::remove_file(&elf_dest_path);
                            
                            let out_file = File::create(&elf_dest_path)?;
                            let mut writer = BufWriter::with_capacity(64 * 1024, out_file);
                            std::io::Write::write_all(&mut writer, &patched)?;

                            let mut perms = fs::metadata(&elf_dest_path)?.permissions();
                            perms.set_mode(permissions & 0o666); // Strip execute bit
                            fs::set_permissions(&elf_dest_path, perms)?;

                            let _ = fs::remove_file(&dest_path);
                            let rpkg_proxy = PathBuf::from(crate::DEFAULT_PREFIX).join("usr/bin/rpkg");
                            std::os::unix::fs::symlink(&rpkg_proxy, &dest_path)?;

                        } else {
                            let _ = fs::remove_file(&dest_path);
                            let out_file = File::create(&dest_path)?;
                            let mut writer = BufWriter::with_capacity(64 * 1024, out_file);
                            std::io::Write::write_all(&mut writer, &patched)?;

                            let mut perms = fs::metadata(&dest_path)?.permissions();
                            perms.set_mode(permissions);
                            fs::set_permissions(&dest_path, perms)?;
                        }

                        installed_files.push(clean_str);
                    }
                    _ => {
                        log::debug!("Skipping unsupported entry type: {}", clean_str);
                    }
                }
            }
            break;
        }
    }

    Ok(installed_files)
}
