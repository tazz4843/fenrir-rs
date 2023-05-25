use crate::{AuthenticationMethod, FenrirBackend, Streams};

/// The `NopBackend` is used by default and does ignore all logging messages.
pub struct NopBackend;

impl FenrirBackend for NopBackend {
    fn send(&self, _: &Streams) -> Result<(), String> {
        Ok(())
    }

    fn authentication_method(&self) -> AuthenticationMethod {
        AuthenticationMethod::None
    }

    fn credentials(&self) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::noop::NopBackend;
    use crate::{AuthenticationMethod, Fenrir};
    use url::Url;

    #[test]
    fn creating_a_noop_instance_without_credentials_works_correctly() {
        let result = Fenrir::<NopBackend>::builder()
            .endpoint(Url::parse("https://loki.example.com").unwrap())
            .build();
        assert_eq!(
            result.backend.authentication_method(),
            AuthenticationMethod::None
        );
        assert_eq!(result.backend.credentials(), None);
    }

    #[test]
    fn creating_a_noop_instance_with_credentials_works_correctly() {
        let result = Fenrir::<NopBackend>::builder()
            .endpoint(Url::parse("https://loki.example.com").unwrap())
            .with_authentication(
                AuthenticationMethod::Basic,
                "username".to_string(),
                "password".to_string(),
            )
            .build();
        assert_eq!(
            result.backend.authentication_method(),
            AuthenticationMethod::None
        );
        assert_eq!(result.backend.credentials(), None);
    }
}
