use std::error::Error;

use zbus::Connection;
use zbus_polkit::policykit1::{AuthorityProxy, AuthorizationResult, CheckAuthorizationFlags, Subject};

pub async fn authenticate() -> Result<AuthorizationResult, Box<dyn Error>> {
    let connection = Connection::system().await?;
    let proxy = AuthorityProxy::new(&connection).await?;
    let subject = Subject::new_for_owner(std::process::id(), None, None)?;
    let result = proxy.check_authorization(
        &subject,
        "org.freedesktop.systemd1.manage-unit-files",
        &std::collections::HashMap::new(),
        CheckAuthorizationFlags::AllowUserInteraction.into(),
        "",
    ).await?;

    Ok(result)
}