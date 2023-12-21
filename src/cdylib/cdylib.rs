// See https://github.com/rusqlite/rusqlite/pull/1425
macro_rules! sqlite3_extension_init {
    ($func: ident) => {
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        #[no_mangle]
        pub extern "C" fn sqlite3_extension_init(
            db: *mut ::rusqlite::ffi::sqlite3,
            pz_err_msg: *mut *mut std::os::raw::c_char,
            p_api: *mut ::rusqlite::ffi::sqlite3_api_routines,
        ) -> std::os::raw::c_int {
            if p_api.is_null() {
                return ::rusqlite::ffi::SQLITE_ERROR;
            }

            let res = unsafe { ::rusqlite::Connection::extension_init2(db, p_api) }.and_then($func);

            if let Err(err) = res {
                return unsafe { ::rusqlite::to_sqlite_error(&err, pz_err_msg) };
            }

            ::rusqlite::ffi::SQLITE_OK
        }
    };
}

use rusqlite::ffi::SQLITE_NOTICE;
use rusqlite::trace::log;
use rusqlite::Connection;

sqlite3_extension_init!(init_loadable_extension);

fn init_loadable_extension(db: Connection) -> ::rusqlite::Result<()> {
    sqlite_hashes::register_hash_functions(&db)?;
    log(SQLITE_NOTICE, "Loaded sqlite_hashes extension");
    Ok(())
}
