use std::fs;

use jsonwebkey as jwk;

pub fn load_from_file(path: &str) -> jwk::JsonWebKey {
    let jwt_str = fs::read_to_string(path).unwrap_or_else(|_| panic!("Unable to read file {}", path));
    jwt_str.parse().expect("Malformed jwk")
}

mod tests {
    #[test]
    fn should_load_wallet_correctly() {
        super::load_from_file("res/test_wallet.json");
    }
}
