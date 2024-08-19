use std::error::Error;
use zbus::{message::Body, Connection};

pub async fn list_units() -> Result<Body, Box<dyn Error>> {
    let connection = Connection::system().await?;
    Ok(connection
        .call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            "ListUnitsByPatterns",
            &(Vec::<&str>::new(), Vec::<&str>::new()),
        )
        .await?
        .body())
}
