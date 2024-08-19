use std::error::Error;
use zbus::Connection;

pub async fn enable(units: Vec<String>) -> Result<(), Box<dyn Error>> {
    let connection = Connection::system().await?;
    let reply_body = connection
        .call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            "EnableUnitFiles",
            &(units, false, false),
        )
        .await?
        .body();
    println!("{reply_body:?}");
    Ok(())
}

pub async fn disable(units: Vec<String>) -> Result<(), Box<dyn Error>> {
    let connection = Connection::system().await?;
    let reply_body = connection
        .call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            "DisableUnitFilesWithFlagsAndInstallInfo",
            &(units, 0u64),
        )
        .await?
        .body();
    println!("{reply_body:?}");
    Ok(())
}

pub async fn is_enabled(unit: String) -> Result<bool, Box<dyn Error>> {
    let connection = Connection::system().await?;
    let reply_body = connection
        .call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            "GetUnitFileState",
            &(unit),
        )
        .await?
        .body();
    let state = reply_body.deserialize::<&str>().unwrap();
    Ok(match state {
        "enabled" | "enabled-runtime" | "static" | "alias" | "indirect" | "generated" => true,
        _ => false,
    })
}
