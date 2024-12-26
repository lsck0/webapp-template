use metrics::counter;

pub fn register_endpoint_call(command: &'static str) {
    counter!(command).increment(1);
}
