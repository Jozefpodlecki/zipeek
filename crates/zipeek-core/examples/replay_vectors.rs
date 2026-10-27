#[path = "../tests/support/mod.rs"]
mod support;

#[unsafe(no_mangle)]
pub extern "C" fn round_golden_digest() -> u64 {
    zipeek_core::practice::fnv1a64(&support::report())
}

fn main() {
    print!("{}", support::report());
    println!("digest={:016x}", round_golden_digest());
}
