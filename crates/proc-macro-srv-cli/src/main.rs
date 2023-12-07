//! A standalone binary for `proc-macro-srv`.
//! Driver for proc macro server
use std::io;

fn main() -> std::io::Result<()> {
    let v = std::env::var("RUST_ANALYZER_INTERNALS_DO_NOT_USE");
    match v.as_deref() {
        Ok("this is unstable") => {
            // very well, if you must
        }
        _ => {
            eprintln!("If you're rust-analyzer, you can use this tool by exporting RUST_ANALYZER_INTERNALS_DO_NOT_USE='this is unstable'.");
            eprintln!("If not, you probably shouldn't use this tool. But do what you want: I'm an error message, not a cop.");
            std::process::exit(122);
        }
    }

    run()
}

#[cfg(not(any(feature = "sysroot-abi", rust_analyzer)))]
fn run() -> io::Result<()> {
    panic!("proc-macro-srv-cli requires the `sysroot-abi` feature to be enabled");
}

#[cfg(any(feature = "sysroot-abi", rust_analyzer))]
fn run() -> io::Result<()> {
    use proc_macro_api::msg::{self, Message};
    use std::cell::RefCell;

    let current_response_prefix = RefCell::new(String::new());

    let read_request = |buf: &mut String| msg::Request::read(&mut io::stdin().lock(), buf);

    let write_response = |msg: msg::Response| {
        use std::io::Write;

        let out = &mut io::stdout().lock();
        let prefix = current_response_prefix.borrow();
        if !prefix.is_empty() {
            out.write_all(prefix.as_bytes())?
        }
        msg.write(out)
    };

    let mut srv = proc_macro_srv::ProcMacroSrv::default();
    let mut buf = String::new();

    while let Some(req) = read_request(&mut buf)? {
        let res = match req {
            msg::Request::ListMacros { dylib_path } => {
                msg::Response::ListMacros(srv.list_macros(&dylib_path))
            }
            msg::Request::ExpandMacro(task) => msg::Response::ExpandMacro(srv.expand(task)),
            msg::Request::ApiVersionCheck {} => {
                msg::Response::ApiVersionCheck(proc_macro_api::msg::CURRENT_API_VERSION)
            }
            msg::Request::SetExpanderSettings { response_prefix } => {
                *current_response_prefix.borrow_mut() = response_prefix;
                msg::Response::SetExpanderSettings {}
            }
        };
        write_response(res)?
    }

    Ok(())
}
