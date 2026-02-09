use validator::ValidateEmail;

// use unicode_segmentation::UnicodeSegmentation;
#[derive(Debug, Clone)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    /// We are rejecting RFC5322 due to not being supported by the upstream crate `validator`
    /// also that, it supports weird unicodes.
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        if s.validate_email() {
            return Ok(Self(s));
        }

        Err(format!("{s} not a valid Email."))
    }
}

impl std::fmt::Display for SubscriberEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::SubscriberEmail;
    use claims::assert_err;
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;
    use fake::rand::SeedableRng;
    use fake::rand::rngs::StdRng;

    #[derive(Debug, Clone)]
    struct ValidEmailFixature(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixature {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            // API changes in `fake` crate, the constraints for g is now stricter and requires a type that implements `Rng` trait which comes from `rand` crate.
            let seed = u64::arbitrary(g);
            // This returns a `StdRng` which is determinstic. We need it to be deterministic so that our tests are reproducible, otherwise on one run, tests can fail and on other run not.
            let mut rng = StdRng::seed_from_u64(seed);
            let email = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_passed(email: ValidEmailFixature) -> bool {
        SubscriberEmail::parse(email.0).is_ok()
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn missing_at_is_rejected() {
        let email = "fsdfsfhskh.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn missing_subject_is_rejected() {
        let email = "@gp.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
}
