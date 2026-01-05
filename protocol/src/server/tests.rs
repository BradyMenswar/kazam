#[cfg(test)]
mod tests {
    use crate::{parse_server_message, ServerMessage};

    #[test]
    fn test_parse_challstr() {
        let line = "|challstr|4|1234abc";
        let message = parse_server_message(line).unwrap();

        assert_eq!(message, ServerMessage::Challstr("4|1234abc".into()))
    }

    #[test]
    fn test_parse_challstr_invalid() {
        let line = "|challstr|";
        let result = parse_server_message(line);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unknown() {
        let line = "|someunknown|data";
        let message = parse_server_message(line).unwrap();

        assert_eq!(message, ServerMessage::Raw("|someunknown|data".to_string()));
    }

    #[test]
    fn test_parse_empty() {
        let line = "";
        let message = parse_server_message(line).unwrap();

        assert_eq!(message, ServerMessage::Raw("".to_string()));
    }
}
