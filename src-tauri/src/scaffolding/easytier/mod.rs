use easytier::VERSION as EASYTIER_VERSION;
use easytier::utils::find_free_tcp_port;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_free_tcp_port() {
        dbg!(EASYTIER_VERSION);
        let port = find_free_tcp_port(1024..65535).unwrap();
        dbg!(port);
        assert!(port > 0);
    }
}
