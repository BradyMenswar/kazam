#[cfg(test)]
mod tests {
    use crate::{ServerMessage, parse_server_message};

    #[test]
    fn test_parse_challstr() {
        let line = "|challstr|4|1234abc";
        let message = parse_server_message(line).unwrap();

        assert_eq!(message, ServerMessage::Challstr("4|1234abc".into()))
    }
}
