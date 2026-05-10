use std::collections::BTreeMap;

use zbus::{Connection, Result, proxy, zvariant::ObjectPath};

#[proxy(
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1",
    interface = "org.freedesktop.systemd1.Manager"
)]
pub trait Manager {
    #[zbus(allow_interactive_auth)]
    fn enable_unit_files(
        &self,
        files: Vec<String>,
        runtime: bool,
        force: bool,
    ) -> Result<(bool, Vec<(String, String, String)>)>;

    #[zbus(allow_interactive_auth)]
    fn disable_unit_files_with_flags_and_install_info(
        &self,
        files: Vec<String>,
        flags: u64,
    ) -> Result<(bool, Vec<(String, String, String)>)>;

    #[zbus(allow_interactive_auth)]
    fn start_unit(&self, name: String, mode: &str) -> Result<String>;

    #[zbus(allow_interactive_auth)]
    fn stop_unit(&self, name: String, mode: &str) -> Result<String>;
}

pub async fn manager_proxy(connection: &Connection) -> Result<ManagerProxy<'_>> {
    ManagerProxy::builder(connection).build().await
}

pub async fn enable(connection: Connection, units: Vec<String>) -> Result<()> {
    let _ = manager_proxy(&connection)
        .await?
        .enable_unit_files(units, false, false)
        .await?;
    Ok(())
}

pub async fn disable(connection: Connection, units: Vec<String>) -> Result<()> {
    let _ = manager_proxy(&connection)
        .await?
        .disable_unit_files_with_flags_and_install_info(units, 0)
        .await?;
    Ok(())
}

pub async fn start(connection: Connection, unit: String) -> Result<()> {
    let _ = manager_proxy(&connection)
        .await?
        .start_unit(unit, "replace")
        .await?;
    Ok(())
}

pub async fn stop(connection: Connection, unit: String) -> Result<()> {
    let _ = manager_proxy(&connection)
        .await?
        .stop_unit(unit, "replace")
        .await?;
    Ok(())
}

type RawUnitInfo<'a> = Vec<(
    String,
    String,
    String,
    String,
    String,
    String,
    ObjectPath<'a>,
    u32,
    String,
    ObjectPath<'a>,
)>;

#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub struct UnitInfo {
    pub description: String,
    pub loaded: String,
    pub active: String,
    pub subunit: String,
}

pub async fn list_units(connection: Connection) -> Result<BTreeMap<String, UnitInfo>> {
    let mut result = BTreeMap::new();
    for (name, _, description, loaded, active, subunit, _, _, _, _) in connection
        .call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            "ListUnits",
            &(),
        )
        .await?
        .body()
        .deserialize::<RawUnitInfo>()?
    {
        let _ = result.insert(
            name,
            UnitInfo {
                description,
                loaded,
                active,
                subunit,
            },
        );
    }
    Ok(result)
}
