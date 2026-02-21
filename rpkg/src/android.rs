#[cfg(feature = "android")]
use crate::manager::PackageManager;
#[cfg(feature = "android")]
use jni::objects::{JClass, JString};
#[cfg(feature = "android")]
use jni::sys::jstring;
#[cfg(feature = "android")]
use jni::EnvUnowned;

#[cfg(feature = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_rpkg_RpkgLib_execute<'local>(mut env: EnvUnowned<'local>, _class: JClass<'local>, prefix: JString<'local>, op: JString<'local>, args: JString<'local>,) -> jstring {
    
    let prefix_str = prefix.to_string();
    let op_str = op.to_string();
    let args_str = args.to_string();
    let outcome = env.with_env(|env| -> Result<JString<'_>, jni::errors::Error> {
        let mut pm = match PackageManager::new(&prefix_str) {
            Ok(pm) => pm,
            Err(e) => {
                let msg = format!("Failed to initialize PackageManager: {}", e);
                log::error!("{}", msg);
                return JString::from_str(env, msg);
            }
        };

        let result = match op_str.as_str() {
            "sync" => match pm.sync() {
                Ok(_) => "Sync completed successfully.".to_string(),
                Err(e) => format!("Failed to sync: {}", e),
            },
            "install" => match pm.install(&args_str, false) {
                Ok(_) => format!("Package '{}' installed successfully.", args_str),
                Err(e) => format!("Failed to install '{}': {}", args_str, e),
            },
            "remove" => match pm.remove(&args_str) {
                Ok(_) => format!("Package '{}' removed successfully.", args_str),
                Err(e) => format!("Failed to remove '{}': {}", args_str, e),
            },
            "upgrade" => match pm.upgrade() {
                Ok(_) => "Upgrade completed successfully.".to_string(),
                Err(e) => format!("Failed to upgrade: {}", e),
            },
            "search" => match pm.search(&args_str) {
                Ok(results) => {
                    if results.is_empty() {
                        "No packages found.".to_string()
                    } else {
                        results
                            .iter()
                            .map(|p| format!("{} v{}\n{}\n", p.name, p.version, p.description))
                            .collect::<Vec<_>>()
                            .join("\n")
                    }
                }
                Err(e) => format!("Failed to search: {}", e),
            },
            _ => format!("Unknown operation: {}", op_str),
        };

        JString::from_str(env, result)
    });

    outcome
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
        .into_raw()
}
