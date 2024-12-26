/// Wraps [std::process::Command] to execute a shell command.
///
/// # Example
///
/// ```rust
/// let name = "World";
/// if run!("echo Hello {}", name).success() {
///    //...
/// }
/// ```
macro_rules! run {
    ($command:literal $( , $command_args:expr )* ) => {
        if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
            let cmd = format!($command $( , $command_args )*);
            tracing::info!("Executing: {}", cmd);
            std::process::Command::new("sh")
                .args(["-c", &cmd])
                .spawn()
                .expect("failed to execute process")
                .wait_with_output()
                .expect("failed to wait for process")
        } else if cfg!(target_os = "windows") {
            panic!("Windows is unsupported, feel free to add support.")
        } else {
            panic!("Unknown/Unsupported OS")
        }
    };
}
pub(crate) use run;

pub trait OutputExt {
    fn success(&self) -> bool;
}

impl OutputExt for std::process::Output {
    fn success(&self) -> bool {
        self.status.success()
    }
}
