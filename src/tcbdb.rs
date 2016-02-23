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

/// TCBDB - The B+ tree database instance.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TCBDB(pub *const c_void);

/// TCBDBCUR - The B+ tree database cursor;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TCBDBCUR(pub *const c_void);

bitflags! {
    flags AdditionalFlags: c_int {
        /// whether opened
        const BDBFOPEN  = 1 << 0,//HDBFOPEN,
        /// whether with fatal error
        const BDBFFATAL = 1 << 1,//HDBFATAL
    }
}

bitflags! {
    flags TuningOptions : c_int {
        /// use 64-bit bucket array
        const BDBTLARGE   = 1<<0,
        /// compress each page with Deflate
        const BDBTDEFLATE = 1<<1,
        /// compress each record with BZIP2
        const BDBTBZIP    = 1<<2,
        /// compress each page with TCBS
        const BDBTTCBS    = 1<<3,
        /// compress each record with outer functions
        const BDBTEXCODEC = 1<<4
    }
}

bitflags! {
    flags OpenModes : c_int {
        /// open as a reader
        const BDBOREADER = 1 << 0,
        /// open as a writer
        const BDBOWRITER = 1 << 1,
        /// writer creating
        const BDBOCREAT  = 1 << 2,
        /// writer truncating
        const BDBOTRUNC  = 1 << 3,
        /// open without locking
        const BDBONOLCK  = 1 << 4,
        /// lock without blocking
        const BDBOLCKNB  = 1 << 5,
        /// synchronize every transaction
        const BDBOTSYNC  = 1 << 6
    }
}

bitflags! {
    flags CursorPutMode : c_int {
        /// current
        const BDBCPCURRENT = 0,
        /// before
        const BDBCPBEFORE  = 1,
        /// after
        const BDBCPAFTER   = 2
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
    pub fn tcbdberrmsg(ecode: c_int) -> *const c_char;
    pub fn tcbdbnew() -> TCBDB;
    pub fn tcbdbdel(bdb: TCBDB);
    pub fn tcbdbecode(bdb: TCBDB)-> c_int;
    pub fn tcbdbsetmutex(bdb: TCBDB) -> bool;
    pub fn tcbdbsetcmpfunc(bdb: TCBDB, cmp: TCCMP, cmpop: *const u8);
    // TODO: check types since these are defined pretty specifically...
    pub fn tcbdbtune(bdb: TCBDB, lmemb: c_int, nmemb: c_int,
                   bnum: c_long, bnum: c_char, fpow: c_char, opts: c_char) -> bool;
    pub fn tcbdbsetcache(bdb: TCBDB, lcnum: c_int, ncnum: c_int) -> bool;
    pub fn tcbdbsetxmsiz(bdb: TCBDB, xmsiz: c_long) -> bool; // 64bit
    pub fn tcbdbsetdfunit(bdb: TCBDB, dfunit: c_int) -> bool; // 32bit 
    pub fn tcbdbopen(bdb: TCBDB, path: *const c_char, omode: c_int) -> bool;
    pub fn tcbdbclose(bdb: TCBDB) -> bool;
    pub fn tcbdbput(bdb: TCBDB, kbuf: *const u8, ksiz: c_int, vbuf: *const u8, vsiz: c_int) -> bool;
    pub fn tcbdbputkeep(bdb: TCBDB, kbuf: *const u8, ksiz: c_int, vbuf: *const u8, vsiz: c_int) -> bool;
    pub fn tcbdbout(bdb: TCBDB, kbuf: *const u8, ksiz: c_int) -> bool;
    pub fn tcbdbget(bdb: TCBDB, kbuf: *const u8, ksiz: c_int, sp: *mut *const c_int) -> *mut u8;
    pub fn tcbdbsync(bdb: TCBDB) -> bool;
    pub fn tcbdbvanish(bdb: TCBDB) -> bool;
}

#[cfg(test)]
mod test {
    use libc::{c_int, c_void};
    use tcbdb::*;
    use tcutil::*;
    use std::slice;
    use std::ffi::{CStr, CString};

    #[test]
    fn test_new_del() {
        unsafe {
            let db = tcbdbnew();
            assert!(!db.0.is_null());
            tcbdbdel(db);
        }
    }

    #[test]
    fn test_fail_put_with_no_open_call() {
        unsafe {
            let db = tcbdbnew();
            assert!(!db.0.is_null());
            let k = b"hello";
            let v = b"world";
            assert!(!tcbdbput(db, k.clone().as_ptr(), k.len() as c_int, v.clone().as_ptr(), v.len() as c_int));
            tcbdbdel(db);
        }
    }

    #[test]
    fn test_with_basics() {
        unsafe {
            let db = tcbdbnew();
            assert!(!db.0.is_null());

            let rustpath = ".tcbdb_test_with_basics.tcb";
            let cpath = CString::new(rustpath).unwrap();
            if !tcbdbopen(db, cpath.as_ptr(), (BDBOWRITER | BDBOCREAT).bits()) {
                let ecode = tcbdbecode(db);
                let errmsg = tcbdberrmsg(ecode);
                println!("{:?}: {:?}", ecode, CStr::from_ptr(errmsg));
                assert!(false);
            }
            let k = b"hello";
            let v = b"world";
            assert!(tcbdbput(db, k.clone().as_ptr(), k.len() as c_int, v.clone().as_ptr(), v.len() as c_int));
            let mut v2_sz: *const i32 = 0 as *const i32;
            let v2_sz_ptr: *mut *const i32 = &mut v2_sz;
            let v2 = tcbdbget(db, k.as_ptr(), k.len() as c_int, v2_sz_ptr);
            assert!(!v2.is_null());
            assert_eq!(v.len(), v2_sz as usize);
            let v2_slice = slice::from_raw_parts(v2, v2_sz as usize);
            assert_eq!(v, v2_slice);

            tcfree(v2 as *const c_void);
            assert!(tcbdbclose(db));
            tcbdbdel(db);
            assert!(::std::fs::remove_file(rustpath).is_ok());
        }
    }
}
