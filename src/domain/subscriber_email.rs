use validator::ValidateEmail;

#[derive(Debug)]
pub struct SubscriberEmail(String);

// expose value as immutable reference
impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        if ValidateEmail::validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("Invalid subscriber email: {}", s))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claims::assert_err;
    use fake::{faker::internet::en::SafeEmail, Fake};
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let email = SafeEmail().fake_with_rng(&mut rng);
            ValidEmailFixture(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_email_passes(email: ValidEmailFixture) -> bool {
        dbg!(&email.0);
        SubscriberEmail::parse(email.0.clone()).is_ok()
    }

    #[test]
    fn empty_email_returns_error() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_returns_error() {
        let email = "usuladomain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_without_subject_returns_error() {
        let email = "@domain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_without_domain_returns_error() {
        let email = "ursula@".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
}
