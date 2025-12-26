#[cfg(feature = "android")]
use crate::{Pty, TerminalEngine, renderer::AndroidRenderer};
use jni::JNIEnv;
use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jint, jlong};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

type EngineHandle = jlong;

struct AndroidSession {
    engine: Arc<Mutex<TerminalEngine>>,
    pty: Arc<Mutex<Pty>>,
    // We keep these to ensure they live as long as the session
    // reader_thread: Option<thread::JoinHandle<()>>,
}

static SESSIONS: OnceLock<Arc<Mutex<HashMap<EngineHandle, AndroidSession>>>> = OnceLock::new();
static NEXT_HANDLE: AtomicI64 = AtomicI64::new(1);

fn get_sessions() -> Arc<Mutex<HashMap<EngineHandle, AndroidSession>>> {
    SESSIONS
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .clone()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_createEngine(
    _env: JNIEnv,
    _class: JClass,
    width: jint,
    height: jint,
    font_size: f32,
) -> jlong {
    #[cfg(feature = "android")]
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("RinNative"),
    );

    log::info!("Creating Engine: {}x{}", width, height);

    // 1. Create Renderer & Engine
    let renderer = Box::new(AndroidRenderer::new(font_size));
    let engine = Arc::new(Mutex::new(TerminalEngine::new(
        width as usize,
        height as usize,
        renderer,
    )));

    // 2. Write startup banner
    {
        let mut engine_guard = engine.lock().unwrap();
        let banner = concat!(
            "\x1b[36m",
            "  ____  _       \r\n",
            " |  _ \\(_)_ __  \r\n",
            " | |_) | | '_ \\ \r\n",
            " |  _ <| | | | |\r\n",
            " |_| \\_\\_|_| |_|\r\n",
            "\x1b[0m\r\n",
            " \x1b[90mTerminal v",
            env!("CARGO_PKG_VERSION"),
            "\x1b[0m\r\n",
            " \x1b[90mgithub.com/pavelc4/Rin\x1b[0m\r\n",
            "\r\n",
        );
        let _ = engine_guard.write(banner.as_bytes());
    }

    // 2. Spawn PTY
    let pty = match Pty::spawn("/system/bin/sh", width as u16, height as u16) {
        Ok(pty) => Arc::new(Mutex::new(pty)),
        Err(e) => {
            log::error!("Failed to spawn PTY: {}", e);
            return -1;
        }
    };

    // 3. Spawn Reader Thread (PTY -> Engine)
    let pty_clone = pty.clone();
    let engine_clone = engine.clone();

    thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        let mut reader = {
            let mut pty_guard = pty_clone.lock().unwrap();
            match pty_guard.take_reader() {
                Ok(r) => r,
                Err(e) => {
                    log::error!("Failed to take PTY reader: {}", e);
                    return;
                }
            }
        };

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => {
                    log::info!("PTY closed (EOF)");
                    break;
                }
                Ok(n) => {
                    let mut engine_guard = engine_clone.lock().unwrap();
                    if let Err(e) = engine_guard.write(&buffer[..n]) {
                        log::error!("Failed to write to engine: {}", e);
                    }
                }
                Err(e) => {
                    log::error!("Error reading from PTY: {}", e);
                    break;
                }
            }
        }
    });

    let handle = NEXT_HANDLE.fetch_add(1, Ordering::SeqCst);
    let session = AndroidSession { engine, pty };

    let sessions_arc = get_sessions();
    sessions_arc.lock().unwrap().insert(handle, session);

    log::info!("Engine created with handle: {}", handle);
    handle
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_destroyEngine(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    let sessions_arc = get_sessions();
    sessions_arc.lock().unwrap().remove(&handle);
    log::info!("Engine destroyed: {}", handle);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_write(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    data: JByteArray,
) -> jint {
    match env.convert_byte_array(&data) {
        Ok(bytes) => {
            let sessions_arc = get_sessions();
            let sessions = sessions_arc.lock().unwrap();
            if let Some(session) = sessions.get(&handle) {
                // Write to PTY, not Engine
                let mut pty = session.pty.lock().unwrap();
                match pty.write(&bytes) {
                    Ok(_) => 0,
                    Err(e) => {
                        log::error!("Failed to write to PTY: {}", e);
                        -1
                    }
                }
            } else {
                -2 // Handle not found
            }
        }
        Err(_) => -1, // Convert failed
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_render(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    let sessions_arc = get_sessions();
    let sessions = sessions_arc.lock().unwrap();
    if let Some(session) = sessions.get(&handle) {
        let mut engine = session.engine.lock().unwrap();
        match engine.render() {
            Ok(_) => 0,
            Err(_) => -1,
        }
    } else {
        -1
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_resize(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    width: jint,
    height: jint,
) -> jint {
    let sessions_arc = get_sessions();
    let sessions = sessions_arc.lock().unwrap();
    if let Some(session) = sessions.get(&handle) {
        // Resize both Engine and PTY
        let mut engine = session.engine.lock().unwrap();
        let _ = engine.resize(width as usize, height as usize);

        let mut pty = session.pty.lock().unwrap();
        let _ = pty.resize(width as u16, height as u16);
        0
    } else {
        -1
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_getLine<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    y: jint,
) -> JString<'local> {
    let sessions_arc = get_sessions();
    let sessions = sessions_arc.lock().unwrap();
    if let Some(session) = sessions.get(&handle) {
        let engine = session.engine.lock().unwrap();
        let buffer = engine.buffer();
        let grid = buffer.grid();
        if let Some(row) = grid.row(y as usize) {
            let line: String = row.iter().map(|c| c.character).collect();
            return env
                .new_string(line)
                .unwrap_or_else(|_| env.new_string("").unwrap());
        }
    }
    env.new_string("").unwrap()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_getCursorX(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    let sessions_arc = get_sessions();
    let sessions = sessions_arc.lock().unwrap();
    if let Some(session) = sessions.get(&handle) {
        let engine = session.engine.lock().unwrap();
        engine.buffer().cursor_pos().0 as jint
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rin_RinLib_getCursorY(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    let sessions_arc = get_sessions();
    let sessions = sessions_arc.lock().unwrap();
    if let Some(session) = sessions.get(&handle) {
        let engine = session.engine.lock().unwrap();
        engine.buffer().cursor_pos().1 as jint
    } else {
        0
    }
}
