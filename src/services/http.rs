use std::time::Duration;

use ureq::Agent;
use ureq::tls::{RootCerts, TlsConfig};

pub(super) fn platform_agent(connect_timeout: Duration, global_timeout: Duration) -> Agent {
    Agent::config_builder()
        .tls_config(
            TlsConfig::builder()
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .timeout_connect(Some(connect_timeout))
        .timeout_global(Some(global_timeout))
        .build()
        .into()
}
