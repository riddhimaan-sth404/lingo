use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const TAG_NIL: i64 = 0;
pub const TAG_BOOL: i64 = 1;
pub const TAG_INT: i64 = 2;
pub const TAG_FLOAT: i64 = 3;
pub const TAG_STR: i64 = 4;
pub const TAG_LIST: i64 = 5;
pub const TAG_CHANNEL: i64 = 6;
pub const TAG_MAP: i64 = 7;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LingoValue {
    pub tag: i64,
    pub data: i64,
}

pub struct LingoString {
    pub ref_count: AtomicUsize,
    pub bytes: Vec<u8>,
}

pub struct LingoList {
    pub ref_count: AtomicUsize,
    pub items: Vec<LingoValue>,
}

pub struct LingoMap {
    pub ref_count: AtomicUsize,
    pub entries: Vec<(LingoValue, LingoValue)>,
}

pub struct LingoChannel {
    pub tx: Mutex<Sender<LingoValue>>,
    pub rx: Mutex<Receiver<LingoValue>>,
}

// Deep clone a value for safe cross-thread transfer
pub unsafe fn deep_clone_value(val: LingoValue) -> LingoValue {
    match val.tag {
        TAG_STR => {
            if val.data == 0 {
                return val;
            }
            let s = &*(val.data as *const LingoString);
            let bytes = s.bytes.clone();
            let raw = Box::into_raw(Box::new(LingoString {
                ref_count: AtomicUsize::new(1),
                bytes,
            })) as usize as i64;
            LingoValue { tag: TAG_STR, data: raw }
        }
        TAG_LIST => {
            if val.data == 0 {
                return val;
            }
            let l = &*(val.data as *const LingoList);
            let mut items = Vec::with_capacity(l.items.len());
            for item in &l.items {
                items.push(deep_clone_value(*item));
            }
            let raw = Box::into_raw(Box::new(LingoList {
                ref_count: AtomicUsize::new(1),
                items,
            })) as usize as i64;
            LingoValue { tag: TAG_LIST, data: raw }
        }
        TAG_MAP => {
            if val.data == 0 {
                return val;
            }
            let m = &*(val.data as *const LingoMap);
            let mut entries = Vec::with_capacity(m.entries.len());
            for (k, v) in &m.entries {
                entries.push((deep_clone_value(*k), deep_clone_value(*v)));
            }
            let raw = Box::into_raw(Box::new(LingoMap {
                ref_count: AtomicUsize::new(1),
                entries,
            })) as usize as i64;
            LingoValue { tag: TAG_MAP, data: raw }
        }
        _ => val,
    }
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_nil(out: *mut LingoValue) {
    *out = LingoValue { tag: TAG_NIL, data: 0 };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_bool(out: *mut LingoValue, b: i64) {
    *out = LingoValue { tag: TAG_BOOL, data: if b != 0 { 1 } else { 0 } };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_int(out: *mut LingoValue, i: i64) {
    *out = LingoValue { tag: TAG_INT, data: i };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_float(out: *mut LingoValue, f: f64) {
    *out = LingoValue { tag: TAG_FLOAT, data: f.to_bits() as i64 };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_str(out: *mut LingoValue, ptr: *const u8, len: usize) {
    let bytes = if ptr.is_null() || len == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(ptr, len).to_vec()
    };
    let raw = Box::into_raw(Box::new(LingoString { ref_count: AtomicUsize::new(1), bytes })) as usize as i64;
    *out = LingoValue { tag: TAG_STR, data: raw };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_is_truthy(val: *const LingoValue) -> i64 {
    let v = *val;
    match v.tag {
        TAG_NIL => 0,
        TAG_BOOL => v.data,
        TAG_INT => if v.data != 0 { 1 } else { 0 },
        TAG_FLOAT => {
            let f = f64::from_bits(v.data as u64);
            if f != 0.0 { 1 } else { 0 }
        }
        TAG_STR => {
            if v.data == 0 { return 0; }
            let s = &*(v.data as *const LingoString);
            if !s.bytes.is_empty() { 1 } else { 0 }
        }
        TAG_LIST => {
            if v.data == 0 { return 0; }
            let l = &*(v.data as *const LingoList);
            if !l.items.is_empty() { 1 } else { 0 }
        }
        TAG_MAP => {
            if v.data == 0 { return 0; }
            let m = &*(v.data as *const LingoMap);
            if !m.entries.is_empty() { 1 } else { 0 }
        }
        _ => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_add(out: *mut LingoValue, a: *const LingoValue, b: *const LingoValue) {
    let a = *a;
    let b = *b;

    if a.tag == TAG_STR || b.tag == TAG_STR {
        let sa = val_to_string(a);
        let sb = val_to_string(b);
        let combined = format!("{sa}{sb}");
        let bytes = combined.into_bytes();
        let raw = Box::into_raw(Box::new(LingoString { ref_count: AtomicUsize::new(1), bytes })) as usize as i64;
        *out = LingoValue { tag: TAG_STR, data: raw };
        return;
    }

    if a.tag == TAG_LIST && b.tag == TAG_LIST {
        let mut items = Vec::new();
        if a.data != 0 {
            let la = &*(a.data as *const LingoList);
            items.extend_from_slice(&la.items);
        }
        if b.data != 0 {
            let lb = &*(b.data as *const LingoList);
            items.extend_from_slice(&lb.items);
        }
        let raw = Box::into_raw(Box::new(LingoList {
            ref_count: AtomicUsize::new(1),
            items,
        })) as usize as i64;
        *out = LingoValue { tag: TAG_LIST, data: raw };
        return;
    }

    if a.tag == TAG_LIST {
        let mut items = Vec::new();
        if a.data != 0 {
            let la = &*(a.data as *const LingoList);
            items.extend_from_slice(&la.items);
        }
        items.push(b);
        let raw = Box::into_raw(Box::new(LingoList {
            ref_count: AtomicUsize::new(1),
            items,
        })) as usize as i64;
        *out = LingoValue { tag: TAG_LIST, data: raw };
        return;
    }

    if a.tag == TAG_MAP && b.tag == TAG_MAP {
        let mut entries = Vec::new();
        if a.data != 0 {
            let ma = &*(a.data as *const LingoMap);
            entries.extend_from_slice(&ma.entries);
        }
        if b.data != 0 {
            let mb = &*(b.data as *const LingoMap);
            for eb in &mb.entries {
                let mut found = false;
                for ea in &mut entries {
                    if lingo_val_eq(&ea.0, &eb.0) != 0 {
                        ea.1 = eb.1;
                        found = true;
                        break;
                    }
                }
                if !found {
                    entries.push(*eb);
                }
            }
        }
        let raw = Box::into_raw(Box::new(LingoMap {
            ref_count: AtomicUsize::new(1),
            entries,
        })) as usize as i64;
        *out = LingoValue { tag: TAG_MAP, data: raw };
        return;
    }

    if a.tag == TAG_INT && b.tag == TAG_INT {
        *out = LingoValue { tag: TAG_INT, data: a.data + b.data };
        return;
    }

    if a.tag == TAG_FLOAT || b.tag == TAG_FLOAT {
        let fa = if a.tag == TAG_FLOAT { f64::from_bits(a.data as u64) } else { a.data as f64 };
        let fb = if b.tag == TAG_FLOAT { f64::from_bits(b.data as u64) } else { b.data as f64 };
        *out = LingoValue { tag: TAG_FLOAT, data: (fa + fb).to_bits() as i64 };
        return;
    }

    *out = LingoValue { tag: TAG_INT, data: a.data + b.data };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_sub(out: *mut LingoValue, a: *const LingoValue, b: *const LingoValue) {
    let a = *a;
    let b = *b;
    if a.tag == TAG_INT && b.tag == TAG_INT {
        *out = LingoValue { tag: TAG_INT, data: a.data - b.data };
        return;
    }
    let fa = if a.tag == TAG_FLOAT { f64::from_bits(a.data as u64) } else { a.data as f64 };
    let fb = if b.tag == TAG_FLOAT { f64::from_bits(b.data as u64) } else { b.data as f64 };
    *out = LingoValue { tag: TAG_FLOAT, data: (fa - fb).to_bits() as i64 };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_mul(out: *mut LingoValue, a: *const LingoValue, b: *const LingoValue) {
    let a = *a;
    let b = *b;
    if a.tag == TAG_INT && b.tag == TAG_INT {
        *out = LingoValue { tag: TAG_INT, data: a.data * b.data };
        return;
    }
    let fa = if a.tag == TAG_FLOAT { f64::from_bits(a.data as u64) } else { a.data as f64 };
    let fb = if b.tag == TAG_FLOAT { f64::from_bits(b.data as u64) } else { b.data as f64 };
    *out = LingoValue { tag: TAG_FLOAT, data: (fa * fb).to_bits() as i64 };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_div(out: *mut LingoValue, a: *const LingoValue, b: *const LingoValue) {
    let a = *a;
    let b = *b;
    let fa = if a.tag == TAG_FLOAT { f64::from_bits(a.data as u64) } else { a.data as f64 };
    let fb = if b.tag == TAG_FLOAT { f64::from_bits(b.data as u64) } else { b.data as f64 };
    if fb == 0.0 {
        eprintln!("Panic: Division by zero");
        std::process::exit(1);
    }
    *out = LingoValue { tag: TAG_FLOAT, data: (fa / fb).to_bits() as i64 };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_mod(out: *mut LingoValue, a: *const LingoValue, b: *const LingoValue) {
    let a = *a;
    let b = *b;
    if b.data == 0 {
        eprintln!("Panic: Modulo by zero");
        std::process::exit(1);
    }
    *out = LingoValue { tag: TAG_INT, data: a.data % b.data };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_and(out: *mut LingoValue, a: *const LingoValue, b: *const LingoValue) {
    let ta = lingo_val_is_truthy(a);
    let tb = lingo_val_is_truthy(b);
    *out = LingoValue { tag: TAG_BOOL, data: if ta != 0 && tb != 0 { 1 } else { 0 } };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_or(out: *mut LingoValue, a: *const LingoValue, b: *const LingoValue) {
    let ta = lingo_val_is_truthy(a);
    let tb = lingo_val_is_truthy(b);
    *out = LingoValue { tag: TAG_BOOL, data: if ta != 0 || tb != 0 { 1 } else { 0 } };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_neq(a: *const LingoValue, b: *const LingoValue) -> i64 {
    if lingo_val_eq(a, b) == 0 { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_eq(a: *const LingoValue, b: *const LingoValue) -> i64 {
    let a = *a;
    let b = *b;
    if a.tag != b.tag { return 0; }
    match a.tag {
        TAG_NIL => 1,
        TAG_BOOL | TAG_INT => if a.data == b.data { 1 } else { 0 },
        TAG_FLOAT => {
            let fa = f64::from_bits(a.data as u64);
            let fb = f64::from_bits(b.data as u64);
            if fa == fb { 1 } else { 0 }
        }
        TAG_STR => {
            if a.data == 0 || b.data == 0 { return if a.data == b.data { 1 } else { 0 }; }
            let sa = &*(a.data as *const LingoString);
            let sb = &*(b.data as *const LingoString);
            if sa.bytes == sb.bytes { 1 } else { 0 }
        }
        _ => if a.data == b.data { 1 } else { 0 },
    }
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_lt(a: *const LingoValue, b: *const LingoValue) -> i64 {
    let a = *a;
    let b = *b;
    if a.tag == TAG_INT && b.tag == TAG_INT {
        return if a.data < b.data { 1 } else { 0 };
    }
    let fa = if a.tag == TAG_FLOAT { f64::from_bits(a.data as u64) } else { a.data as f64 };
    let fb = if b.tag == TAG_FLOAT { f64::from_bits(b.data as u64) } else { b.data as f64 };
    if fa < fb { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_gt(a: *const LingoValue, b: *const LingoValue) -> i64 {
    let a = *a;
    let b = *b;
    if a.tag == TAG_INT && b.tag == TAG_INT {
        return if a.data > b.data { 1 } else { 0 };
    }
    let fa = if a.tag == TAG_FLOAT { f64::from_bits(a.data as u64) } else { a.data as f64 };
    let fb = if b.tag == TAG_FLOAT { f64::from_bits(b.data as u64) } else { b.data as f64 };
    if fa > fb { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_gte(a: *const LingoValue, b: *const LingoValue) -> i64 {
    let a = *a;
    let b = *b;
    if a.tag == TAG_INT && b.tag == TAG_INT {
        return if a.data >= b.data { 1 } else { 0 };
    }
    let fa = if a.tag == TAG_FLOAT { f64::from_bits(a.data as u64) } else { a.data as f64 };
    let fb = if b.tag == TAG_FLOAT { f64::from_bits(b.data as u64) } else { b.data as f64 };
    if fa >= fb { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_lte(a: *const LingoValue, b: *const LingoValue) -> i64 {
    let a = *a;
    let b = *b;
    if a.tag == TAG_INT && b.tag == TAG_INT {
        return if a.data <= b.data { 1 } else { 0 };
    }
    let fa = if a.tag == TAG_FLOAT { f64::from_bits(a.data as u64) } else { a.data as f64 };
    let fb = if b.tag == TAG_FLOAT { f64::from_bits(b.data as u64) } else { b.data as f64 };
    if fa <= fb { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn lingo_list_new(out: *mut LingoValue) {
    let boxed = Box::new(LingoList {
        ref_count: AtomicUsize::new(1),
        items: Vec::new(),
    });
    let raw = Box::into_raw(boxed) as usize as i64;
    *out = LingoValue { tag: TAG_LIST, data: raw };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_list_push(list_val: *const LingoValue, val: *const LingoValue) {
    let list_v = *list_val;
    if list_v.tag != TAG_LIST || list_v.data == 0 { return; }
    let list = &mut *(list_v.data as *mut LingoList);
    list.items.push(*val);
}

#[no_mangle]
pub unsafe extern "C" fn lingo_list_get(out: *mut LingoValue, list_val: *const LingoValue, idx: i64) {
    let list_v = *list_val;
    if list_v.tag != TAG_LIST || list_v.data == 0 {
        *out = LingoValue { tag: TAG_NIL, data: 0 };
        return;
    }
    let list = &*(list_v.data as *const LingoList);
    if idx < 0 || (idx as usize) >= list.items.len() {
        eprintln!("Panic: Index {} out of bounds (len: {})", idx, list.items.len());
        std::process::exit(1);
    }
    *out = list.items[idx as usize];
}

#[no_mangle]
pub unsafe extern "C" fn lingo_map_new(out: *mut LingoValue) {
    let boxed = Box::new(LingoMap {
        ref_count: AtomicUsize::new(1),
        entries: Vec::new(),
    });
    let raw = Box::into_raw(boxed) as usize as i64;
    *out = LingoValue { tag: TAG_MAP, data: raw };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_map_insert(map_val: *const LingoValue, key: *const LingoValue, val: *const LingoValue) {
    let mv = *map_val;
    if mv.tag != TAG_MAP || mv.data == 0 { return; }
    let m = &mut *(mv.data as *mut LingoMap);
    for entry in &mut m.entries {
        if lingo_val_eq(&entry.0, key) != 0 {
            entry.1 = *val;
            return;
        }
    }
    m.entries.push((*key, *val));
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_get(out: *mut LingoValue, target: *const LingoValue, index_or_key: *const LingoValue) {
    let t = *target;
    let k = *index_or_key;

    match t.tag {
        TAG_LIST => {
            if t.data == 0 {
                *out = LingoValue { tag: TAG_NIL, data: 0 };
                return;
            }
            let list = &*(t.data as *const LingoList);
            let idx = if k.tag == TAG_INT { k.data } else { 0 };
            if idx < 0 || (idx as usize) >= list.items.len() {
                eprintln!("Panic: Index {} out of bounds (len: {})", idx, list.items.len());
                std::process::exit(1);
            }
            *out = list.items[idx as usize];
        }
        TAG_MAP => {
            if t.data == 0 {
                *out = LingoValue { tag: TAG_NIL, data: 0 };
                return;
            }
            let m = &*(t.data as *const LingoMap);
            for (ek, ev) in &m.entries {
                if lingo_val_eq(ek, &k) != 0 {
                    *out = *ev;
                    return;
                }
            }
            *out = LingoValue { tag: TAG_NIL, data: 0 };
        }
        TAG_STR => {
            if t.data == 0 {
                *out = LingoValue { tag: TAG_NIL, data: 0 };
                return;
            }
            let s = &*(t.data as *const LingoString);
            let idx = if k.tag == TAG_INT { k.data } else { 0 };
            if idx < 0 || (idx as usize) >= s.bytes.len() {
                *out = LingoValue { tag: TAG_NIL, data: 0 };
                return;
            }
            let single_byte = vec![s.bytes[idx as usize]];
            let raw = Box::into_raw(Box::new(LingoString {
                ref_count: AtomicUsize::new(1),
                bytes: single_byte,
            })) as usize as i64;
            *out = LingoValue { tag: TAG_STR, data: raw };
        }
        _ => {
            *out = LingoValue { tag: TAG_NIL, data: 0 };
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_set(target: *const LingoValue, index_or_key: *const LingoValue, val: *const LingoValue) {
    let t = *target;
    let k = *index_or_key;

    match t.tag {
        TAG_LIST => {
            if t.data == 0 { return; }
            let list = &mut *(t.data as *mut LingoList);
            let idx = if k.tag == TAG_INT { k.data } else { 0 };
            if idx < 0 || (idx as usize) >= list.items.len() {
                eprintln!("Panic: Index {} out of bounds for assignment (len: {})", idx, list.items.len());
                std::process::exit(1);
            }
            list.items[idx as usize] = *val;
        }
        TAG_MAP => {
            if t.data == 0 { return; }
            let m = &mut *(t.data as *mut LingoMap);
            for entry in &mut m.entries {
                if lingo_val_eq(&entry.0, &k) != 0 {
                    entry.1 = *val;
                    return;
                }
            }
            m.entries.push((k, *val));
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "C" fn lingo_val_len(out: *mut LingoValue, val: *const LingoValue) {
    let v = *val;
    let len = match v.tag {
        TAG_STR => {
            if v.data == 0 { 0 } else { (&*(v.data as *const LingoString)).bytes.len() }
        }
        TAG_LIST => {
            if v.data == 0 { 0 } else { (&*(v.data as *const LingoList)).items.len() }
        }
        TAG_MAP => {
            if v.data == 0 { 0 } else { (&*(v.data as *const LingoMap)).entries.len() }
        }
        _ => 0,
    };
    *out = LingoValue { tag: TAG_INT, data: len as i64 };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_channel_new(out: *mut LingoValue) {
    let (tx, rx) = channel();
    let chan = Arc::new(LingoChannel {
        tx: Mutex::new(tx),
        rx: Mutex::new(rx),
    });
    let raw = Arc::into_raw(chan) as usize as i64;
    *out = LingoValue { tag: TAG_CHANNEL, data: raw };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_channel_send(chan_val: *const LingoValue, val: *const LingoValue) {
    let cv = *chan_val;
    if cv.tag != TAG_CHANNEL || cv.data == 0 {
        return;
    }
    let chan = &*(cv.data as *const LingoChannel);
    let cloned_val = deep_clone_value(*val);
    if let Ok(tx) = chan.tx.lock() {
        let _ = tx.send(cloned_val);
    }
}

#[no_mangle]
pub unsafe extern "C" fn lingo_channel_recv(out: *mut LingoValue, chan_val: *const LingoValue) {
    let cv = *chan_val;
    if cv.tag != TAG_CHANNEL || cv.data == 0 {
        *out = LingoValue { tag: TAG_NIL, data: 0 };
        return;
    }
    let chan = &*(cv.data as *const LingoChannel);
    if let Ok(rx) = chan.rx.lock() {
        if let Ok(received) = rx.recv() {
            *out = received;
            return;
        }
    }
    *out = LingoValue { tag: TAG_NIL, data: 0 };
}

#[no_mangle]
pub extern "C" fn lingo_sleep_ms(ms: i64) {
    if ms > 0 {
        thread::sleep(Duration::from_millis(ms as u64));
    }
}

#[no_mangle]
pub extern "C" fn lingo_time_now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn lingo_file_read(out: *mut LingoValue, path_val: *const LingoValue) {
    let p_str = val_to_string(*path_val);
    match std::fs::read_to_string(&p_str) {
        Ok(content) => {
            let bytes = content.into_bytes();
            let raw = Box::into_raw(Box::new(LingoString { ref_count: AtomicUsize::new(1), bytes })) as usize as i64;
            *out = LingoValue { tag: TAG_STR, data: raw };
        }
        Err(_) => {
            *out = LingoValue { tag: TAG_NIL, data: 0 };
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn lingo_file_write(path_val: *const LingoValue, content_val: *const LingoValue) -> i64 {
    let p_str = val_to_string(*path_val);
    let c_str = val_to_string(*content_val);
    if std::fs::write(&p_str, c_str).is_ok() { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn lingo_file_exists(path_val: *const LingoValue) -> i64 {
    let p_str = val_to_string(*path_val);
    if std::path::Path::new(&p_str).exists() { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn lingo_env_get(out: *mut LingoValue, key_val: *const LingoValue) {
    let k_str = val_to_string(*key_val);
    match std::env::var(&k_str) {
        Ok(v) => {
            let bytes = v.into_bytes();
            let raw = Box::into_raw(Box::new(LingoString { ref_count: AtomicUsize::new(1), bytes })) as usize as i64;
            *out = LingoValue { tag: TAG_STR, data: raw };
        }
        Err(_) => {
            *out = LingoValue { tag: TAG_NIL, data: 0 };
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn lingo_math_sqrt(out: *mut LingoValue, val: *const LingoValue) {
    let v = *val;
    let num = if v.tag == TAG_FLOAT { f64::from_bits(v.data as u64) } else { v.data as f64 };
    *out = LingoValue { tag: TAG_FLOAT, data: num.sqrt().to_bits() as i64 };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_math_sin(out: *mut LingoValue, val: *const LingoValue) {
    let v = *val;
    let num = if v.tag == TAG_FLOAT { f64::from_bits(v.data as u64) } else { v.data as f64 };
    *out = LingoValue { tag: TAG_FLOAT, data: num.sin().to_bits() as i64 };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_math_cos(out: *mut LingoValue, val: *const LingoValue) {
    let v = *val;
    let num = if v.tag == TAG_FLOAT { f64::from_bits(v.data as u64) } else { v.data as f64 };
    *out = LingoValue { tag: TAG_FLOAT, data: num.cos().to_bits() as i64 };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_math_floor(out: *mut LingoValue, val: *const LingoValue) {
    let v = *val;
    let num = if v.tag == TAG_FLOAT { f64::from_bits(v.data as u64) } else { v.data as f64 };
    *out = LingoValue { tag: TAG_INT, data: num.floor() as i64 };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_math_ceil(out: *mut LingoValue, val: *const LingoValue) {
    let v = *val;
    let num = if v.tag == TAG_FLOAT { f64::from_bits(v.data as u64) } else { v.data as f64 };
    *out = LingoValue { tag: TAG_INT, data: num.ceil() as i64 };
}

#[no_mangle]
pub unsafe extern "C" fn lingo_math_abs(out: *mut LingoValue, val: *const LingoValue) {
    let v = *val;
    if v.tag == TAG_INT {
        *out = LingoValue { tag: TAG_INT, data: v.data.abs() };
    } else {
        let num = if v.tag == TAG_FLOAT { f64::from_bits(v.data as u64) } else { v.data as f64 };
        *out = LingoValue { tag: TAG_FLOAT, data: num.abs().to_bits() as i64 };
    }
}

fn val_to_string(val: LingoValue) -> String {
    match val.tag {
        TAG_NIL => "None".to_string(),
        TAG_BOOL => if val.data != 0 { "true".to_string() } else { "false".to_string() },
        TAG_INT => val.data.to_string(),
        TAG_FLOAT => f64::from_bits(val.data as u64).to_string(),
        TAG_STR => unsafe {
            if val.data == 0 { return String::new(); }
            let s = &*(val.data as *const LingoString);
            String::from_utf8_lossy(&s.bytes).to_string()
        },
        TAG_LIST => unsafe {
            if val.data == 0 { return "[]".to_string(); }
            let l = &*(val.data as *const LingoList);
            let mut buf = String::from("[");
            for (i, item) in l.items.iter().enumerate() {
                if i > 0 { buf.push_str(", "); }
                buf.push_str(&val_to_string(*item));
            }
            buf.push(']');
            buf
        },
        TAG_MAP => unsafe {
            if val.data == 0 { return "{}".to_string(); }
            let m = &*(val.data as *const LingoMap);
            let mut buf = String::from("{");
            for (i, (k, v)) in m.entries.iter().enumerate() {
                if i > 0 { buf.push_str(", "); }
                buf.push_str(&val_to_string(*k));
                buf.push_str(": ");
                buf.push_str(&val_to_string(*v));
            }
            buf.push('}');
            buf
        },
        TAG_CHANNEL => "<channel>".to_string(),
        _ => format!("<unknown: {}>", val.tag),
    }
}

#[no_mangle]
pub unsafe extern "C" fn lingo_println_val(val: *const LingoValue) {
    println!("{}", val_to_string(*val));
}

#[no_mangle]
pub unsafe extern "C" fn lingo_print_val(val: *const LingoValue) {
    print!("{}", val_to_string(*val));
}
