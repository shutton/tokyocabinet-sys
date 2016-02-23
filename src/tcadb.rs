//
// Copyright 2016 Ewan Higgs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use libc::{c_char, c_int, c_void};

/// TCADB - The Abstract tree database instance.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TCADB(pub *const c_void);

bitflags! {
    flags AdditionalFlags: c_int {
        /// whether opened
        const BDBFOPEN  = 1 << 0,//HDBFOPEN,
        /// whether with fatal error
        const BDBFFATAL = 1 << 1,//HDBFATAL
    }
}

/*
 * In the following, I use *const u8 for void* since void* has no size, so,
 * afaik, using c_void would just force further casts elsewhere.
 */
#[allow(dead_code)]
#[link(name = "tokyocabinet")]
extern {
    pub fn tcadbnew() -> TCADB;
    pub fn tcadbdel(db: TCADB);
    pub fn tcadbopen(db: TCADB, name: *const c_char);
    pub fn tcadbclose(db: TCADB);
    pub fn tcadbput(db: TCADB, kbuf: *const u8, ksiz: c_int, vbuf: *const u8, vsiz: c_int) -> bool;
    pub fn tcadbputkeep(db: TCADB, kbuf: *const u8, ksiz: c_int, vbuf: *const u8, vsiz: c_int) -> bool;
    pub fn tcadbputkeep2(db: TCADB, kstr: *const c_char, vstr: *const c_char) -> bool;
    pub fn tcadbout(db: TCADB, kbuf: *const u8, ksiz: c_int);
    pub fn tcadbget(db: TCADB, kbuf: *const u8, ksiz: c_int, sp: *mut *const c_int) -> *const u8;
    pub fn tcadbsync(db: TCADB) -> bool;
    pub fn tcadbvanish(db: TCADB) -> bool;
}

#[cfg(test)]
mod test {
    use libc::{c_int, c_void};
    use tcadb::*;
    use tcutil::*;
    use std::slice;
    use std::ffi::CString;

    #[test]
    fn test_new_del() {
        unsafe {
            let db = tcadbnew();
            assert!(!db.0.is_null());
            tcadbdel(db);
        }
    }

    #[test]
    fn test_fail_put_with_no_open_call() {
        unsafe {
            let db = tcadbnew();
            assert!(!db.0.is_null());
            let k = b"hello";
            let v = b"world";
            assert!(!tcadbput(db, k.clone().as_ptr(), k.len() as c_int, v.clone().as_ptr(), v.len() as c_int));
            tcadbdel(db);
        }
    }

    #[test]
    fn test_with_basics() {
        unsafe {
            let db = tcadbnew();
            assert!(!db.0.is_null());

            let rustpath = "+"; // "+" is shorthand for in memory b+tree in tc
            let cpath = CString::new(rustpath).unwrap();
            tcadbopen(db, cpath.as_ptr());
            let k = b"hello";
            let v = b"world";
            assert!(tcadbput(db, k.clone().as_ptr(), k.len() as c_int, v.clone().as_ptr(), v.len() as c_int));
            let mut v2_sz: *const i32 = 0 as *const i32;
            let v2_sz_ptr: *mut *const i32 = &mut v2_sz;
            let v2 = tcadbget(db, k.as_ptr(), k.len() as c_int, v2_sz_ptr);
            assert!(!v2.is_null());
            assert_eq!(v.len(), v2_sz as usize);
            let v2_slice = slice::from_raw_parts(v2, v2_sz as usize);
            assert_eq!(v, v2_slice);

            tcfree(v2 as *const c_void);
            tcadbclose(db);
            tcadbdel(db);
        }
    }
}
