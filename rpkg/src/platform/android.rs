// SAFETY: JNI entry point called from Java/Kotlin (RpkgLib.execute).
// #[unsafe(no_mangle)] + extern "system" required for JNI ABI.
// EnvUnowned and JString/JClass references are valid only within the JNI call frame.
// into_raw() releases ownership of the returned jstring to the JNI framework.

use crate::manager::PackageManager;
use jni::strings::JNIStr;
use jni::EnvUnowned;
use jni::objects::{JClass, JString};
use jni::sys::jstring;
use std::ffi::CString;

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_rpkg_RpkgLib_execute<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    prefix: JString<'local>,
    op: JString<'local>,
    args: JString<'local>,
) -> jstring {
    let outcome = env.with_env(|env| -> Result<JString<'_>, jni::errors::Error> {
        let prefix_str: String = {
            #[allow(deprecated)]
            let s = env.get_string(&prefix)?;
            s.into()
        };
        let op_str: String = {
            #[allow(deprecated)]
            let s = env.get_string(&op)?;
            s.into()
        };
        let args_str: String = {
            #[allow(deprecated)]
            let s = env.get_string(&args)?;
            s.into()
        };

        let mut pm = match PackageManager::new(&prefix_str) {
            Ok(pm) => pm,
            Err(e) => {
                let msg = format!("Failed to initialize PackageManager: {}", e);
                log::error!("{}", msg);
                let class_cstr = CString::new("java/lang/RuntimeException").unwrap();
                let msg_cstr = CString::new(msg).unwrap();
                let class = env.find_class(JNIStr::from_cstr(&class_cstr).unwrap())?;
                let _ = env.throw_new(&class, JNIStr::from_cstr(&msg_cstr).unwrap());
                return Err(jni::errors::Error::JavaException);
            }
        };

        let result = match op_str.as_str() {
            "sync" => match pm.sync() {
                Ok(_) => "Sync completed successfully.".to_string(),
                Err(e) => format!("Failed to sync: {}", e),
            },
            "install" => match pm.install(&[args_str.clone()], false) {
                Ok(_) => format!("Package '{}' installed successfully.", args_str),
                Err(e) => format!("Failed to install '{}': {}", args_str, e),
            },
            "remove" => match pm.remove(&[args_str.clone()]) {
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

    // SAFETY: resolve() throws Java RuntimeException via ThrowRuntimeExAndDefault
    // if a JNI error occurred. into_raw() hands the raw jstring to JNI; the caller
    // (Kotlin RpkgLib.execute) owns the returned string.
    outcome
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
        .into_raw()
}
