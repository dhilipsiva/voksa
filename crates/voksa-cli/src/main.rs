fn main() {
    println!("voksa {}", voksa_core::VERSION);
}

#[cfg(test)]
mod tests {
    #[test]
    fn core_version_is_wired_up() {
        assert_eq!(voksa_core::VERSION, env!("CARGO_PKG_VERSION"));
    }
}
