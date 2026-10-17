use {
    crate::Result,
    std::{
        rc::Rc,
        fs::File,
        mem::MaybeUninit,
    },
};

pub type DebugLog = Rc<MaybeUninit<File>>;

#[macro_export]
macro_rules! debug {
    ($file:expr $(,$($tt:tt)*)?) => {{
        #[cfg(feature = "debug-logging")]
        unsafe {
            use std::io::Write;
            let _ = write!($file.assume_init_ref() $(, $($tt)*)?);
        }
    }}
}

#[macro_export]
macro_rules! debugln {
    ($file:expr $(,$($tt:tt)*)?) => {{
        #[cfg(feature = "debug-logging")]
        unsafe {
            use std::io::Write;
            let _ = writeln!($file.assume_init_ref() $(, $($tt)*)?);
        }
    }}
}

pub use crate::{debug, debugln};

pub fn open_file() -> Result<DebugLog> {
    #[cfg(feature = "debug-logging")]
    {
        use std::io::Write;
        let time = chrono::Local::now();
        let mut file = File::create("semmel-debug.log")?;
        writeln!(file, "SEMMELSCRIPT DEBUG LOG, {}", time.format("%Y-%m-%d %H:%M:%S"))?;
        return Ok(Rc::new(MaybeUninit::new(file)))
    }
    #[allow(unreachable_code)]
    return Ok(Rc::new(MaybeUninit::zeroed()))
}
