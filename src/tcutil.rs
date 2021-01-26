use libc::{c_char, c_int, c_void, size_t};
/// TCXSTR - Extensible String
#[derive(Debug)]
#[repr(C)]
pub struct TCXSTR {
    ptr: *const c_void,
    size: c_int,
    asize: c_int,
}

// TODO: This probably belongs in a separate tokyocabinet crate
impl From<TCXSTR> for &[u8] {
    fn from(tcxstr: TCXSTR) -> Self {
        unsafe { std::slice::from_raw_parts(tcxstr.ptr as *const u8, tcxstr.size as usize) }
    }
}

impl TCXSTR {
    pub fn as_bytes(&self) -> &'static [u8] {
        unsafe { std::slice::from_raw_parts(self.ptr as *const u8, self.size as usize) }
    }
}

/// TCLISTDATUM - element of a list.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TCLISTDATUM{
    ptr: *const c_char,
    size: c_int,
}

/// TCLIST - Array list.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TCLIST(pub *const c_void);


#[allow(dead_code)]
#[link(name = "tokyocabinet")]
extern {
    pub fn tcmalloc(size: size_t) -> *const c_void;
    pub fn tccalloc(nmemb: size_t, size: size_t) -> *const c_void;
    pub fn tcrealloc(ptr: *const c_void, size: size_t) -> *const c_void;
    pub fn tcmemdup(ptr: *const c_void, size: size_t) -> *const c_void;
    pub fn tcstrdup(str: *const c_void) -> *const c_char;
    pub fn tcfree(ptr: *const c_void);

    pub fn tcxstrnew() -> TCXSTR;
    pub fn tcxstrnew2(str: *const c_char)-> TCXSTR;
    pub fn tcxstrnew3(asiz: c_int) -> TCXSTR;
    pub fn tcxstrdup(xstr: &TCXSTR) -> TCXSTR;
    pub fn tcxstrdel(xstr: &TCXSTR);
    pub fn tcxstrcat(xstr: &TCXSTR, ptr: *const u8, size: c_int);
    pub fn tcxstrcat2(xstr: &TCXSTR, str: *const c_char);
    pub fn tcxstrptr(xstr: &TCXSTR) -> *const u8;
    pub fn tcxstrsize(xstr: &TCXSTR) -> c_int;
    pub fn tcxstrclear(xstr: TCXSTR);
    //pub fn tcxstrprintf(TCXSTR *xstr, const char *format, ...);
    //pub fn tcsprintf(const char *format, ...) -> *const c_char;

    pub fn tclistnew() -> TCLIST;
    pub fn tclistnew2(anum: c_int) -> TCLIST;
    //pub fn tclistnew3(...) -> TCLIST;
    pub fn tclistdup(list: &TCLIST) -> TCLIST;
    pub fn tclistdel(list: TCLIST);
    pub fn tclistnum(list: &TCLIST) -> c_int;

    pub fn tclistval(list: &TCLIST, index: c_int, sp: *const c_int) -> *const u8;

    pub fn tclistval2(list: &TCLIST, index: c_int)-> *const c_char;
    pub fn tclistpush(list: &TCLIST, ptr: *const u8, size: c_int);
    pub fn tclistpush2(list: &TCLIST, str: *const c_char);
    pub fn tclistpop(list: &TCLIST, sp: *const c_int) -> *const u8;
    pub fn tclistpop2(list: &TCLIST) -> *const u8;
    pub fn tclistunshift(list: &TCLIST, ptr: *const u8, size: c_int);
    pub fn tclistunshift2(list: &TCLIST, str: *const c_char);
    pub fn tclistinsert(list: &TCLIST, index: c_int, ptr: *const u8, size: c_int);
    pub fn tclistinsert2(list: &TCLIST, index: c_int, str: *const c_char);
    pub fn tclistremove(list: &TCLIST, index: c_int, sp: *const c_int) -> *const u8;
    pub fn tclistremove2(list: &TCLIST, index: c_int) -> *const c_char;
    pub fn tclistover(list: &TCLIST, index: c_int, ptr: *const u8, size: c_int);
    pub fn tclistover2(list: &TCLIST, index: c_int, str: *const c_char);
    pub fn tclistsort(list: &mut TCLIST);
    pub fn tclistlsearch(list: &TCLIST, ptr: *const u8, size: c_int) -> c_int;
    pub fn tclistbsearch(list: &TCLIST, ptr: *const u8, size: c_int) -> c_int;
    pub fn tclistclear(list: &mut TCLIST);
    pub fn tclistdump(list: &TCLIST, sp: *const c_int)-> *const u8;
    pub fn tclistload(ptr: *const u8, size: c_int) -> TCLIST;
}

pub type TCCMP = extern "C" fn(aptr: *const c_char, asiz: c_int, bptr: *const c_char, bsiz: c_int, op: *const u8);
pub type TCCODEC = extern "C" fn(ptr: *const u8, size: c_int, sp: *const c_int, op: *const u8);
pub type TCPDPROC = extern "C" fn(vbuf: *const u8, vsiz: c_int, sp: *const c_int, op: *const u8);
pub type TCITER = extern "C" fn(kbuf: *const u8, ksiz: c_int, vbuf: *const u8, vsiz: c_int, op: *const u8);

bitflags! {
    flags ErrorCodes : c_int {
        const TCESUCCESS = 0,     /* success */
        const TCETHREAD  = 1,     /* threading error */
        const TCEINVALID = 2,     /* invalid operation */
        const TCENOFILE  = 3,     /* file not found */
        const TCENOPERM  = 4,     /* no permission */
        const TCEMETA    = 5,     /* invalid meta data */
        const TCERHEAD   = 6,     /* invalid record header */
        const TCEOPEN    = 7,     /* open error */
        const TCECLOSE   = 8,     /* close error */
        const TCETRUNC   = 9,     /* trunc error */
        const TCESYNC    = 10,    /* sync error */
        const TCESTAT    = 11,    /* stat error */
        const TCESEEK    = 12,    /* seek error */
        const TCEREAD    = 13,    /* read error */
        const TCEWRITE   = 14,    /* write error */
        const TCEMMAP    = 15,    /* mmap error */
        const TCELOCK    = 16,    /* lock error */
        const TCEUNLINK  = 17,    /* unlink error */
        const TCERENAME  = 18,    /* rename error */
        const TCEMKDIR   = 19,    /* mkdir error */
        const TCERMDIR   = 20,    /* rmdir error */
        const TCEKEEP    = 21,    /* existing record */
        const TCENOREC   = 22,    /* no record found */
        const TCEMISC    = 9999   /* miscellaneous error */
    }
}
