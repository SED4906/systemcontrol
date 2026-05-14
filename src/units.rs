use std::collections::BTreeMap;

use zbus::{Result, blocking::Connection, proxy, zvariant::ObjectPath};

type R = Result<(bool, Vec<(String, String, String)>)>;

#[proxy(
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1",
    interface = "org.freedesktop.systemd1.Manager"
)]
pub trait Manager {
    #[zbus(allow_interactive_auth)]
    fn enable_unit_files(&self, files: Vec<String>, runtime: bool, force: bool) -> R;

    #[zbus(allow_interactive_auth)]
    fn disable_unit_files_with_flags_and_install_info(&self, files: Vec<String>, flags: u64) -> R;

    #[zbus(allow_interactive_auth)]
    fn start_unit(&self, name: String, mode: &str) -> Result<String>;

    #[zbus(allow_interactive_auth)]
    fn stop_unit(&self, name: String, mode: &str) -> Result<String>;
}

pub fn manager_proxy(connection: &Connection) -> Result<ManagerProxyBlocking<'_>> {
    ManagerProxyBlocking::builder(connection).build()
}

pub fn enable(connection_fn: fn() -> Result<Connection>, units: Vec<String>) -> Result<()> {
    manager_proxy(&connection_fn()?)?.enable_unit_files(units, false, false)?;
    Ok(())
}

pub fn disable(connection_fn: fn() -> Result<Connection>, units: Vec<String>) -> Result<()> {
    manager_proxy(&connection_fn()?)?.disable_unit_files_with_flags_and_install_info(units, 0)?;
    Ok(())
}

pub fn start(connection_fn: fn() -> zbus::Result<Connection>, unit: String) -> Result<()> {
    manager_proxy(&connection_fn()?)?.start_unit(unit, "replace")?;
    Ok(())
}

pub fn stop(connection_fn: fn() -> zbus::Result<Connection>, unit: String) -> Result<()> {
    manager_proxy(&connection_fn()?)?.stop_unit(unit, "replace")?;
    Ok(())
}

type RawUnitInfoList<'a> = Vec<(
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
    pub substate: String,
    pub subunit: String,
}

pub fn list_units(connection: Connection) -> Result<BTreeMap<String, UnitInfo>> {
    let mut result = BTreeMap::new();
    for (name, description, loaded, active, substate, subunit, _, _, _, _) in connection
        .call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            "ListUnits",
            &(),
        )?
        .body()
        .deserialize::<RawUnitInfoList>()?
    {
        let _ = result.insert(
            name,
            UnitInfo {
                description,
                loaded,
                active,
                substate,
                subunit,
            },
        );
    }
    Ok(result)
}
