use metrics::counter;

#[allow(unused)]
pub fn register_command_call(command: &'static str) {
    counter!(command).increment(1);
}
