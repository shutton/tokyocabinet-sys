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

use libc::{c_char, c_int, c_long, c_void};

use tcutil::TCCMP;

/// TCHDB - The hash table database instance.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TCHDB(pub *const c_void);

/// HDBCUR - The Hash Table database cursor;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HDBCUR(pub *const c_void);

bitflags! {
    flags AdditionalFlags: c_int {
        const HDBFOPEN  = 1 << 0, // whether opened
        const HDBFFATAL = 1 << 1  // whether with fatal error
    }
}

bitflags! {
    flags TuningOptions : c_int {
        const HDBTLARGE   = 1<<0, // use 64-bit bucket array
        const HDBTDEFLATE = 1<<1, // compress each page with Deflate
        const HDBTBZIP    = 1<<2, // compress each record with BZIP2
        const HDBTTCBS    = 1<<3, // compress each page with TCBS
        const HDBTEXCODEC = 1<<4  // compress each record with outer functions
    }
}

bitflags! {
    flags OpenModes : c_int {
        const HDBOREADER = 1 << 0, // open as a reader
        const HDBOWRITER = 1 << 1, // open as a writer
        const HDBOCREAT  = 1 << 2, // writer creating
        const HDBOTRUNC  = 1 << 3, // writer truncating
        const HDBONOLCK  = 1 << 4, // open without locking
        const HDBOLCKNB  = 1 << 5, // lock without blocking
        const HDBOTSYNC  = 1 << 6  // synchronize every transaction
    }
}

bitflags! {
    flags CursorPutMode : c_int {
        const HDBCPCURRENT = 0, // current
        const HDBCPBEFORE  = 1, // before
        const HDBCPAFTER   = 2  // after
    }
}


/*
 * In the following, I use *const u8 for void* since void* has no size, so,
 * afaik, using c_void would just force further casts elsewhere.
 */
#[allow(dead_code)]
#[link(name = "tokyocabinet")]
extern {
    // API of ordered tree
    pub fn tchdberrmsg(ecode: c_int) -> *const c_char;
    pub fn tchdbnew() -> TCHDB;
    pub fn tchdbdel(hdb: TCHDB);
    pub fn tchdbecode(hdb: TCHDB)-> c_int;
    pub fn tchdbsetmutex(hdb: TCHDB) -> bool;
    pub fn tchdbsetcmpfunc(hdb: TCHDB, cmp: TCCMP, cmpop: *const u8);
    // TODO: check types since these are defined pretty specifically...
    pub fn tchdbtune(hdb: TCHDB, lmemb: c_int, nmemb: c_int,
                   bnum: c_long, bnum: c_char, fpow: c_char, opts: c_char) -> bool;
    pub fn tchdbsetcache(hdb: TCHDB, lcnum: c_int, ncnum: c_int) -> bool;
    pub fn tchdbsetxmsiz(hdb: TCHDB, xmsiz: c_long) -> bool; // 64bit
    pub fn tchdbsetdfunit(hdb: TCHDB, dfunit: c_int) -> bool; // 32bit 
    pub fn tchdbopen(hdb: TCHDB, path: *const c_char, omode: c_int) -> bool;
    pub fn tchdbclose(hdb: TCHDB) -> bool;
    pub fn tchdbput(hdb: TCHDB, kbuf: *const u8, ksiz: c_int, vbuf: *const u8, vsiz: c_int) -> bool;
    pub fn tchdbputkeep(hdb: TCHDB, kbuf: *const u8, ksiz: c_int, vbuf: *const u8, vsiz: c_int) -> bool;
    pub fn tchdbout(hdb: TCHDB, kbuf: *const u8, ksiz: c_int) -> bool;
    pub fn tchdbget(hdb: TCHDB, kbuf: *const u8, ksiz: c_int, sp: *mut *const c_int) -> *mut u8;
    pub fn tchdbsync(hdb: TCHDB) -> bool;
    pub fn tchdbvanish(hdb: TCHDB) -> bool;
}

#[cfg(test)]
mod test {
    use libc::{c_int, c_void};
    use tchdb::*;
    use tcutil::*;
    use std::slice;
    use std::ffi::{CStr, CString};

    #[test]
    fn test_new_del() {
        unsafe {
            let db = tchdbnew();
            assert!(!db.0.is_null());
            tchdbdel(db);
        }
    }

    #[test]
    fn test_fail_put_with_no_open_call() {
        unsafe {
            let db = tchdbnew();
            assert!(!db.0.is_null());
            let k = b"hello";
            let v = b"world";
            assert!(!tchdbput(db, k.clone().as_ptr(), k.len() as c_int, v.clone().as_ptr(), v.len() as c_int));
            tchdbdel(db);
        }
    }

    #[test]
    fn test_with_basics() {
        unsafe {
            let db = tchdbnew();
            assert!(!db.0.is_null());

            let rustpath = ".tchdb_test_with_basics.tch";
            let cpath = CString::new(rustpath).unwrap();
            if !tchdbopen(db, cpath.as_ptr(), (HDBOWRITER | HDBOCREAT).bits()) {
                let ecode = tchdbecode(db);
                let errmsg = tchdberrmsg(ecode);
                println!("{:?}: {:?}", ecode, CStr::from_ptr(errmsg));
                assert!(false);
            }
            let k = b"hello";
            let v = b"world";
            assert!(tchdbput(db, k.clone().as_ptr(), k.len() as c_int, v.clone().as_ptr(), v.len() as c_int));
            let mut v2_sz: *const i32 = 0 as *const i32;
            let v2_sz_ptr: *mut *const i32 = &mut v2_sz;
            let v2 = tchdbget(db, k.as_ptr(), k.len() as c_int, v2_sz_ptr);
            assert!(!v2.is_null());
            assert_eq!(v.len(), v2_sz as usize);
            let v2_slice = slice::from_raw_parts(v2, v2_sz as usize);
            assert_eq!(v, v2_slice);

            tcfree(v2 as *const c_void);
            assert!(tchdbclose(db));
            tchdbdel(db);
            assert!(::std::fs::remove_file(rustpath).is_ok());
        }
    }
}
