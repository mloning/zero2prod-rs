use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

// expose value as immutable reference
impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl SubscriberName {
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;
        let invalid_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_invalid_chars = s.chars().any(|g| invalid_chars.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_invalid_chars {
            Err(format!("Invalid subscriber name: {}", s))
        } else {
            Ok(Self(s))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberName;
    use claims::{assert_err, assert_ok};

    #[test]
    fn name_with_256_graphemes_passes() {
        let name = "ä".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn name_longer_than_256_graphemes_passes() {
        let name = "ä".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn name_with_whitespace_only_returns_error() {
        let name = " ".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn name_with_invalid_char_returns_error() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }
    #[test]
    fn valid_name_passes() {
        let name = "Ursula Le Guin".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
