//! A module which contains the implementation for the [`FenrirBackend`] trait which uses the `ureq`
//! crate for network communication.
use crate::{AuthenticationMethod, FenrirBackend, SerializationFn, Streams};
use std::any::TypeId;
use url::Url;

/// A [`FenrirBackend`] implementation which uses the [ureq](https://crates.io/crates/ureq) crate to
/// send logging messages to a Loki endpoint.
pub(crate) struct UreqBackend {
    /// The loki endpoint which is used to send log information to
    pub(crate) endpoint: Url,
    /// The authentication method to use when sending the log messages to the remote [`UreqBackend::endpoint`]
    pub(crate) authentication: AuthenticationMethod,
    /// The credentials to use to authenticate against the remote [`UreqBackend::endpoint`]
    pub(crate) credentials: String,
}

impl FenrirBackend for UreqBackend {
    fn send(&self, streams: &Streams, serializer: SerializationFn) -> Result<(), String> {
        use std::time::Duration;
        use ureq::AgentBuilder;
        debug!("UreqBackend::send() called, sending log messages to Loki.");

        let log_stream_text = serializer(streams).unwrap();
        trace!("Sending log messages to Loki: {}", log_stream_text);

        let post_url = self.endpoint.clone().join("/loki/api/v1/push").unwrap();
        let agent = AgentBuilder::new().timeout(Duration::from_secs(10)).build();
        let mut request = agent.request_url("POST", &post_url);
        request = request.set("Content-Type", "application/json; charset=utf-8");
        match self.authentication {
            AuthenticationMethod::None => {}
            AuthenticationMethod::Basic => {
                request = request.set(
                    "Authorization",
                    format!("Basic {}", self.credentials).as_str(),
                );
            }
        }
        let response = request.send_string(log_stream_text.as_str());
        trace!("Response from Loki: {:?}", response);
        if let Err(e) = response {
            error!("Failed to send log messages to Loki: {}", e);
            return Err(e.to_string());
        }

        Ok(())
    }

    fn internal_type(&self) -> TypeId {
        use std::any::Any;

        TypeId::of::<Self>().type_id()
    }

    fn authentication_method(&self) -> AuthenticationMethod {
        self.authentication.clone()
    }

    fn credentials(&self) -> Option<String> {
        if self.credentials.len() > 0 {
            return Some(self.credentials.clone());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::ureq::UreqBackend;
    use crate::{AuthenticationMethod, Fenrir, NetworkingBackend, SerializationFormat};
    use std::any::{Any, TypeId};
    use url::Url;

    #[test]
    fn creating_a_ureq_instance_without_credentials_works_correctly() {
        let result = Fenrir::builder()
            .endpoint(Url::parse("https://loki.example.com").unwrap())
            .network(NetworkingBackend::Ureq)
            .format(SerializationFormat::Json)
            .build();
        assert_eq!(
            result.backend.authentication_method(),
            AuthenticationMethod::None
        );
        assert_eq!(result.backend.credentials(), None);
        assert_eq!(
            result.backend.internal_type(),
            TypeId::of::<UreqBackend>().type_id()
        );
    }

    #[test]
    fn creating_a_ureq_instance_with_credentials_works_correctly() {
        let result = Fenrir::builder()
            .endpoint(Url::parse("https://loki.example.com").unwrap())
            .network(NetworkingBackend::Ureq)
            .format(SerializationFormat::Json)
            .with_authentication(
                AuthenticationMethod::Basic,
                "username".to_string(),
                "password".to_string(),
            )
            .build();
        assert_eq!(
            result.backend.authentication_method(),
            AuthenticationMethod::Basic
        );
        assert_eq!(
            result.backend.credentials(),
            Some("dXNlcm5hbWU6cGFzc3dvcmQ=".to_string())
        );
        assert_eq!(
            result.backend.internal_type(),
            TypeId::of::<UreqBackend>().type_id()
        );
    }
}
