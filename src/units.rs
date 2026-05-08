use std::error::Error;
use zbus::{Connection, message::Body};

pub async fn list_user_units() -> Result<Body, Box<dyn Error>> {
    list_units(Connection::session().await?).await
}

pub async fn list_system_units() -> Result<Body, Box<dyn Error>> {
    list_units(Connection::system().await?).await
}

async fn list_units(connection: Connection) -> Result<Body, Box<dyn Error>> {
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
