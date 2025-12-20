#[cfg(feature = "android")]
use crate::{TerminalEngine, renderer::AndroidRenderer};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

type EngineHandle = usize;

static mut ENGINES: Option<Arc<Mutex<HashMap<EngineHandle, TerminalEngine>>>> = None;
static mut NEXT_HANDLE: usize = 1;

fn get_engines() -> Arc<Mutex<HashMap<EngineHandle, TerminalEngine>>> {
    unsafe {
        ENGINES.get_or_insert_with(|| {
            Arc::new(Mutex::new(HashMap::new()))
        }).clone()
    }
}

#[no_mangle]
pub extern "C" fn terminal_engine_create(width: usize, height: usize, font_size: f32) -> EngineHandle {
    let renderer = Box::new(AndroidRenderer::new(font_size));
    let engine = TerminalEngine::new(width, height, renderer);

    let handle = unsafe {
        let h = NEXT_HANDLE;
        NEXT_HANDLE += 1;
        h
    };

    get_engines().lock().unwrap().insert(handle, engine);
    handle
}

#[no_mangle]
pub extern "C" fn terminal_engine_destroy(handle: EngineHandle) {
    get_engines().lock().unwrap().remove(&handle);
}

#[no_mangle]
pub extern "C" fn terminal_engine_write(
    handle: EngineHandle,
    data_ptr: *const u8,
    data_len: usize,
) -> i32 {
    if data_ptr.is_null() {
        return -1;
    }

    let data = unsafe { std::slice::from_raw_parts(data_ptr, data_len) };

    let mut engines = get_engines().lock().unwrap();
    if let Some(engine) = engines.get_mut(&handle) {
        match engine.write(data) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn terminal_engine_render(handle: EngineHandle) -> i32 {
    let mut engines = get_engines().lock().unwrap();
    if let Some(engine) = engines.get_mut(&handle) {
        match engine.render() {
            Ok(_) => 0,
            Err(_) => -1,
        }
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn terminal_engine_resize(
    handle: EngineHandle,
    width: usize,
    height: usize,
) -> i32 {
    let mut engines = get_engines().lock().unwrap();
    if let Some(engine) = engines.get_mut(&handle) {
        match engine.resize(width, height) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    } else {
        -1
    }
}
