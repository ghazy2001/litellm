#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Connect(#[from] litellm_redis_client::ConnectError),
    #[error(
        "Redis setting credential_provider names a Python object, which this host cannot supply"
    )]
    NoHostCredentialProvider,
    #[error("{stage} failed")]
    Credential {
        stage: CredentialStage,
        #[source]
        cause: litellm_error::Source,
    },
    #[error("credential acquisition exceeded {timeout:?}")]
    CredentialTimeout {
        timeout: std::time::Duration,
        #[source]
        cause: tokio::time::error::Elapsed,
    },
    #[error("Redis credential has expired")]
    ExpiredCredential,
    #[error("no {provider} credential is available for Redis")]
    MissingCredential { provider: &'static str },
}

#[derive(Clone, Copy, Debug, strum::Display, strum::VariantArray)]
pub enum CredentialStage {
    #[strum(serialize = "caller identity")]
    Identity,
    #[strum(serialize = "Redis token signing")]
    Signing,
    #[strum(serialize = "Redis token exchange")]
    Exchange,
    #[strum(serialize = "Redis token response")]
    Response,
}

impl Error {
    pub(crate) fn credential<E: std::error::Error + Send + Sync + 'static>(
        stage: CredentialStage,
    ) -> impl FnOnce(E) -> Self {
        move |cause| Self::Credential {
            stage,
            cause: litellm_error::Source::new(cause),
        }
    }

    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::Connect(_) => "connection",
            Self::NoHostCredentialProvider => "host-credential-provider",
            Self::Credential { stage, .. } => match stage {
                CredentialStage::Identity => "caller-identity",
                CredentialStage::Signing => "token-signing",
                CredentialStage::Exchange => "token-exchange",
                CredentialStage::Response => "token-response",
            },
            Self::CredentialTimeout { .. } => "acquisition-timeout",
            Self::ExpiredCredential => "expired-credential",
            Self::MissingCredential { .. } => "missing-credential",
        }
    }
}
