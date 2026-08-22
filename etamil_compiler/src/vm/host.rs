//! Where the VM's input and output actually go.
//!
//! The interpreter used to call `std::fs` and `println!` directly, which tied
//! it to an operating system: it could not be built for the browser, and it
//! could not be tested without touching the disk. Both are now behind this
//! module.
//!
//! This is a `#[cfg]`-selected module rather than a trait the VM holds. A trait
//! would be the more general answer, but it would mean threading a `&mut dyn
//! Host` through every method of a 2,000-line interpreter and changing every
//! signature on the way. Swapping `fs::write(..)` for `host::write(..)` changes
//! the call and nothing else. If the VM ever needs two hosts alive at once --
//! two programs running side by side in one process -- this becomes a trait,
//! and the call sites do not move again.
//!
//! On wasm, output accumulates in a buffer the embedder drains, and files live
//! in memory. Both are `thread_local`: wasm32 is single-threaded, so there is
//! nothing to lock, and a native build gets one host per thread, which is what
//! a thread-per-request server wants.

// --- Native -----------------------------------------------------------------

#[cfg(not(target_family = "wasm"))]
mod imp {
    use std::fs;
    use std::fs::OpenOptions;
    use std::io::Write as _;

    pub fn print_line(text: &str) {
        println!("{}", text);
    }

    pub fn read_line() -> Result<String, String> {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| e.to_string())?;
        Ok(input)
    }

    pub fn read_to_string(path: &str) -> Result<String, String> {
        fs::read_to_string(path).map_err(|e| e.to_string())
    }

    pub fn write(path: &str, contents: &[u8]) -> Result<(), String> {
        fs::write(path, contents).map_err(|e| e.to_string())
    }

    pub fn append_line(path: &str, data: &str) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        writeln!(file, "{}", data).map_err(|e| e.to_string())
    }

    /// Flush both streams and end the process.
    ///
    /// Exiting does not unwind, so anything still buffered would be lost --
    /// including the summary line that explains the status.
    pub fn exit(status: i32) -> Result<(), String> {
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();
        std::process::exit(status);
    }
}

// --- Browser ----------------------------------------------------------------

#[cfg(target_family = "wasm")]
mod imp {
    use std::cell::RefCell;
    use std::collections::HashMap;

    thread_local! {
        /// Everything the program has printed, waiting to be drained.
        static OUTPUT: RefCell<String> = const { RefCell::new(String::new()) };
        /// The program's filesystem. Starts empty every run.
        static FILES: RefCell<HashMap<String, Vec<u8>>> =
            RefCell::new(HashMap::new());
    }

    pub fn print_line(text: &str) {
        OUTPUT.with(|out| {
            let mut out = out.borrow_mut();
            out.push_str(text);
            out.push('\n');
        });
    }

    /// There is no console to read from. Returning an error rather than an
    /// empty string means `உள்ளிடு` reports why it did nothing instead of
    /// silently yielding "".
    pub fn read_line() -> Result<String, String> {
        Err("உள்ளிடு உலாவியில் கிடைக்காது  \
             (input is not available in the browser)"
            .to_string())
    }

    pub fn read_to_string(path: &str) -> Result<String, String> {
        FILES.with(|files| {
            files
                .borrow()
                .get(path)
                .ok_or_else(|| {
                    format!("கோப்பு '{}' இல்லை  (no such file '{}')", path, path)
                })
                .and_then(|bytes| {
                    String::from_utf8(bytes.clone()).map_err(|_| {
                        format!(
                            "கோப்பு '{}' UTF-8 அல்ல  ('{}' is not valid UTF-8)",
                            path, path
                        )
                    })
                })
        })
    }

    pub fn write(path: &str, contents: &[u8]) -> Result<(), String> {
        FILES.with(|files| {
            files.borrow_mut().insert(path.to_string(), contents.to_vec());
        });
        Ok(())
    }

    pub fn append_line(path: &str, data: &str) -> Result<(), String> {
        FILES.with(|files| {
            let mut files = files.borrow_mut();
            let entry = files.entry(path.to_string()).or_default();
            entry.extend_from_slice(data.as_bytes());
            entry.push(b'\n');
        });
        Ok(())
    }

    /// A non-zero exit is the program's own failure and is reported as one.
    /// There is no process to end.
    pub fn exit(status: i32) -> Result<(), String> {
        if status == 0 {
            Ok(())
        } else {
            Err(format!(
                "நிரல் {} நிலையுடன் நின்றது  (the program exited with status {})",
                status, status
            ))
        }
    }

    /// Take everything printed so far, leaving the buffer empty.
    pub fn take_output() -> String {
        OUTPUT.with(|out| std::mem::take(&mut *out.borrow_mut()))
    }

    /// Clear output and files. Called before each run so one program cannot
    /// see the last one's leftovers.
    pub fn reset() {
        OUTPUT.with(|out| out.borrow_mut().clear());
        FILES.with(|files| files.borrow_mut().clear());
    }

    /// Seed a file before a run, so an example can read input it did not write.
    pub fn put_file(path: &str, contents: &str) {
        let _ = write(path, contents.as_bytes());
    }

    /// Paths the program wrote, sorted, so the embedder can show them.
    pub fn file_names() -> Vec<String> {
        FILES.with(|files| {
            let mut names: Vec<String> = files.borrow().keys().cloned().collect();
            names.sort();
            names
        })
    }
}

pub use imp::*;
