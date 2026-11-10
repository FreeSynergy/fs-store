//! A stand-in for a background server in the tests: it does nothing for a minute. The tests copy it to the name of a
//! program and start it as `<name> serve`.
fn main() {
    std::thread::sleep(std::time::Duration::from_secs(60));
}
