use std::error::Error;
use zbus::Connection;

pub async fn enable(units: Vec<String>) -> Result<(), Box<dyn Error>> {
    let connection = Connection::system().await?;
    let _ = connection
        .call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            "EnableUnitFiles",
            &(units, false, false),
        )
        .await?
        .body();
    //println!("{reply_body:?}");
    Ok(())
}

pub async fn disable(units: Vec<String>) -> Result<(), Box<dyn Error>> {
    let connection = Connection::system().await?;
    let _ = connection
        .call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            "DisableUnitFilesWithFlagsAndInstallInfo",
            &(units, 0u64),
        )
        .await?
        .body();
    //println!("{reply_body:?}");
    Ok(())
}

pub async fn start(unit: String) -> Result<(), Box<dyn Error>> {
    let connection = Connection::system().await?;
    let _ = connection
        .call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            "StartUnit",
            &(unit, "replace"),
        )
        .await?
        .body();
    //println!("{reply_body:?}");
    Ok(())
}

pub async fn stop(unit: String) -> Result<(), Box<dyn Error>> {
    let connection = Connection::system().await?;
    let _ = connection
        .call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            "StopUnit",
            &(unit, "replace"),
        )
        .await?
        .body();
    //println!("{reply_body:?}");
    Ok(())
}