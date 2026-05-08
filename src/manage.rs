use std::error::Error;
use zbus::Connection;

pub async fn enable(connection: Connection, units: Vec<String>) -> Result<(), Box<dyn Error>> {
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
    Ok(())
}

pub async fn disable(connection: Connection, units: Vec<String>) -> Result<(), Box<dyn Error>> {
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
    Ok(())
}

pub async fn start(connection: Connection, unit: String) -> Result<(), Box<dyn Error>> {
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
    Ok(())
}

pub async fn stop(connection: Connection, unit: String) -> Result<(), Box<dyn Error>> {
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
    Ok(())
}
