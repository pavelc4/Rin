use super::session::TerminalSession;
use jni::objects::{JByteArray, JClass, JIntArray, JString};
use jni::sys::{jint, jlong};
use jni::EnvUnowned;
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Once, OnceLock, RwLock};

static LOGGER_INIT: Once = Once::new();

fn ensure_logger() {
    #[cfg(feature = "android")]
    LOGGER_INIT.call_once(|| {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("RinNative"),
        );
    });
}

type EngineHandle = jlong;

const SESSION_ERR: jint = -1;

static SESSIONS: OnceLock<Arc<RwLock<HashMap<EngineHandle, TerminalSession>>>> = OnceLock::new();
static NEXT_HANDLE: AtomicI64 = AtomicI64::new(1);

fn get_sessions() -> Arc<RwLock<HashMap<EngineHandle, TerminalSession>>> {
    SESSIONS
        .get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
        .clone()
}

fn get_jstring(env: &mut EnvUnowned<'_>, s: &JString<'_>) -> String {
    env.with_env(|env| -> jni::errors::Result<String> {
        #[allow(deprecated)]
        let java_str = env.get_string(s).map(|s| String::from(s))?;
        Ok(java_str)
    })
    .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

fn with_session<R>(handle: jlong, f: impl FnOnce(&TerminalSession) -> R) -> Option<R> {
    let sessions = get_sessions();
    let sessions = sessions.read().unwrap();
    sessions.get(&handle).map(f)
}

fn create_banner(
    is_root: bool,
    has_storage_permission: bool,
    _home_dir: &str,
    _username: &str,
) -> String {
    let mut banner = String::from(concat!(
        "\x1b[36m",
        r"  ____  _       ",
        "\r\n",
        r" |  _ \(_)_ __  ",
        "\r\n",
        r" | |_) | | '_ \ ",
        "\r\n",
        r" |  _ <| | | | |",
        "\r\n",
        r" |_| \_\_|_| |_|",
        "\r\n",
        "\x1b[0m\r\n",
        " \x1b[90mTerminal v",
        env!("CARGO_PKG_VERSION"),
        "\x1b[0m\r\n",
        " \x1b[90mgithub.com/pavelc4/Rin\x1b[0m\r\n",
        "\r\n",
    ));

    if is_root {
        banner.push_str(concat!(
            " \x1b[31m\x1b[1mROOT SESSION\x1b[0m\r\n",
            " \x1b[33mType '\x1b[1mhelp\x1b[0m\x1b[33m' for available commands\x1b[0m\r\n",
            "\r\n",
        ));
    } else {
        banner.push_str(concat!(
            " \x1b[33mType '\x1b[1mhelp\x1b[0m\x1b[33m' for available commands\x1b[0m\r\n",
            "\r\n",
        ));
    }

    if !has_storage_permission {
        banner.push_str(concat!(
            " \x1b[31m\x1b[1mStorage permission required!\x1b[0m\r\n",
            " \x1b[33mRun '\x1b[1mrin-perm-storage\x1b[0m\x1b[33m' to grant access\x1b[0m\r\n",
            " \x1b[90mPackage operations will fail without permission\x1b[0m\r\n",
            "\r\n",
        ));
    } else {
        banner.push_str(concat!(
            " \x1b[32mStorage permission granted\x1b[0m\r\n",
            "\r\n",
        ));
    }

    banner
}

fn create_engine_inner(
    width: jint,
    height: jint,
    font_size: f32,
    home_dir: &str,
    username: &str,
    has_storage_permission: bool,
    is_root: bool,
    su_path: Option<&str>,
) -> jlong {
    ensure_logger();

    let username_str = if username.is_empty() { "user" } else { username };

    let label = if is_root { "Root Engine" } else { "Engine" };
    log::info!(
        "Creating {}: {}x{}, HOME={}, USER={}",
        label,
        width,
        height,
        home_dir,
        username_str
    );

    let session = TerminalSession::new(
        width as usize,
        height as usize,
        font_size,
        home_dir,
        username_str,
        su_path,
    );

    let buffer = session.get_buffer();
    let mut engine = buffer.lock().unwrap();
    let banner = create_banner(is_root, has_storage_permission, home_dir, username_str);
    let _ = engine.write(banner.as_bytes());

    let handle = NEXT_HANDLE.fetch_add(1, Ordering::SeqCst);
    get_sessions().write().unwrap().insert(handle, session);

    log::info!("{} created with handle: {}", label, handle);
    handle
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_createEngine(
    mut env: EnvUnowned,
    _class: JClass,
    width: jint,
    height: jint,
    font_size: f32,
    home_dir: JString,
    username: JString,
    has_storage_permission: jint,
) -> jlong {
    let home_dir_str = get_jstring(&mut env, &home_dir);
    let username_str = get_jstring(&mut env, &username);
    create_engine_inner(
        width,
        height,
        font_size,
        &home_dir_str,
        &username_str,
        has_storage_permission != 0,
        false,
        None,
    )
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_createRootEngine(
    mut env: EnvUnowned,
    _class: JClass,
    width: jint,
    height: jint,
    font_size: f32,
    home_dir: JString,
    username: JString,
    has_storage_permission: jint,
    su_path: JString,
) -> jlong {
    let home_dir_str = get_jstring(&mut env, &home_dir);
    let username_str = get_jstring(&mut env, &username);
    let su_path_str = get_jstring(&mut env, &su_path);
    create_engine_inner(
        width,
        height,
        font_size,
        &home_dir_str,
        &username_str,
        has_storage_permission != 0,
        true,
        Some(&su_path_str),
    )
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_destroyEngine(
    _env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) {
    get_sessions().write().unwrap().remove(&handle);
    log::info!("Engine destroyed: {}", handle);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_write(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    data: JByteArray,
) -> jint {
    let outcome = env.with_env(|env| env.convert_byte_array(&data));
    let bytes: Vec<u8> = outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>();
    with_session(handle, |session| match session.write(&bytes) {
        Ok(_) => 0,
        Err(e) => {
            log::error!("Failed to write to PTY: {}", e);
            SESSION_ERR
        }
    })
    .unwrap_or(SESSION_ERR)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_writeToEngine(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    data: JByteArray,
) -> jint {
    let outcome = env.with_env(|env| env.convert_byte_array(&data));
    let bytes: Vec<u8> = outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>();
    with_session(handle, |session| match session.write_to_engine(&bytes) {
        Ok(_) => 0,
        Err(e) => {
            log::error!("Failed to write to engine: {}", e);
            -1
        }
    })
    .unwrap_or(-2)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_render(
    _env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jint {
    with_session(handle, |session| match session.render() {
        Ok(_) => 0,
        Err(_) => -1,
    })
    .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_resize(
    _env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    width: jint,
    height: jint,
) -> jint {
    with_session(handle, |session| match session.resize(width as usize, height as usize) {
        Ok(_) => 0,
        Err(_) => -1,
    })
    .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_getLine<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
    y: jint,
) -> JString<'local> {
    let line = with_session(handle, |session| {
        let buffer = session.get_buffer();
        let engine = buffer.lock().unwrap();
        let buffer = engine.buffer();
        let grid = buffer.grid();
        grid.row(y as usize).map(|row| row.iter().map(|c| c.character).collect::<String>())
    })
    .flatten()
    .unwrap_or_default();
    env.with_env(|env| env.new_string(line))
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_getCursorX(
    _env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jint {
    with_session(handle, |session| {
        let buffer = session.get_buffer();
        let engine = buffer.lock().unwrap();
        engine.buffer().cursor_pos().0 as jint
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_getCursorY(
    _env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jint {
    with_session(handle, |session| {
        let buffer = session.get_buffer();
        let engine = buffer.lock().unwrap();
        engine.buffer().cursor_pos().1 as jint
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_getCellData<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
    y: jint,
) -> JString<'local> {
    let result = with_session(handle, |session| {
        let buffer = session.get_buffer();
        let engine = buffer.lock().unwrap();
        let buffer = engine.buffer();
        let grid = buffer.grid();
        grid.row(y as usize).map(|row| {
            let mut result = String::with_capacity(row.len() * 32);
            for cell in row.iter() {
                if cell.wide_spacer {
                    continue;
                }

                let style = &cell.style;
                let (fg, bg) = if style.reverse {
                    (&style.bg, &style.fg)
                } else {
                    (&style.fg, &style.bg)
                };

                let _ = write!(
                    result,
                    "{}\t{},{},{}\t{},{},{}",
                    cell.character, fg.r, fg.g, fg.b, bg.r, bg.g, bg.b
                );
                result.push('\t');

                if style.bold {
                    result.push('b');
                }
                if style.italic {
                    result.push('i');
                }
                if style.dim {
                    result.push('d');
                }
                if cell.wide {
                    result.push('w');
                }

                result.push('\n');
            }
            result
        })
    })
    .flatten()
    .unwrap_or_default();
    env.with_env(|env| env.new_string(result))
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_getCellDataOptimized<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
    y: jint,
) -> JIntArray<'local> {
    let data = with_session(handle, |session| {
        let buffer = session.get_buffer();
        let engine = buffer.lock().unwrap();
        let buffer = engine.buffer();
        let grid = buffer.grid();
        grid.row(y as usize).map(|row| {
            let mut data = Vec::with_capacity(row.len() * 3);
            for cell in row.iter() {
                if cell.wide_spacer {
                    continue;
                }

                let style = &cell.style;
                let (fg, bg) = if style.reverse {
                    (&style.bg, &style.fg)
                } else {
                    (&style.fg, &style.bg)
                };

                let mut char_flags = (cell.character as u32) & 0x001F_FFFF;
                if style.bold {
                    char_flags |= 1 << 21;
                }
                if style.italic {
                    char_flags |= 1 << 22;
                }
                if style.dim {
                    char_flags |= 1 << 23;
                }
                if cell.wide {
                    char_flags |= 1 << 24;
                }

                let fg_packed = ((fg.r as u32) << 16) | ((fg.g as u32) << 8) | (fg.b as u32);
                let bg_packed = ((bg.r as u32) << 16) | ((bg.g as u32) << 8) | (bg.b as u32);

                data.push(char_flags as i32);
                data.push(fg_packed as i32);
                data.push(bg_packed as i32);
            }
            data
        })
    })
    .flatten()
    .unwrap_or_default();

    env.with_env(|env| -> jni::errors::Result<jni::objects::JIntArray> {
        let jarray = env.new_int_array(data.len())?;
        env.set_int_array_region(&jarray, 0, &data)?;
        Ok(jarray)
    })
    .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_hasDirtyRows(
    _env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> bool {
    with_session(handle, |session| {
        let buffer = session.get_buffer();
        let engine = buffer.lock().unwrap();
        engine.buffer().grid().has_dirty_rows()
    })
    .unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_clearDirty(
    _env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) {
    with_session(handle, |session| {
        let buffer = session.get_buffer();
        let mut engine = buffer.lock().unwrap();
        engine.buffer_mut().grid_mut().clear_dirty();
    });
}
