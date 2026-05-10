mod units;

use std::{collections::BTreeMap, process::Command, time::Duration, vec};

use descape::UnescapeExt;
use eframe::egui;
use pollster::FutureExt;
use tokio::runtime::Runtime;
use zbus::{Connection, connection::Builder};

use crate::units::UnitInfo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new()?;
    let _enter = rt.enter();
    let _dbus_thread = rt.spawn(async { tokio::time::sleep(Duration::MAX).await });
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "System Control",
        native_options,
        Box::new(|cc| Ok(Box::new(SystemControlApp::new(cc)))),
    )?;
    Ok(())
}

#[derive(Default)]
struct SystemControlApp {
    user_units: BTreeMap<String, UnitInfo>,
    system_units: BTreeMap<String, UnitInfo>,
    unit_log: String,
    refresh: bool,
    show_inactive: bool,
    unit_filter: String,
}

impl SystemControlApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            refresh: true,
            user_units: BTreeMap::new(),
            system_units: BTreeMap::new(),
            unit_log: String::new(),
            show_inactive: true,
            unit_filter: String::new(),
        }
    }
}

impl eframe::App for SystemControlApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.unit_filter).hint_text("Filter..."));
                ui.checkbox(&mut self.show_inactive, "Show Inactive");
                self.refresh |= ui.button("Refresh").clicked();
            });
            if self.refresh {
                if let Ok(user_units) =
                    async { units::list_units(Connection::session().await?).await }.block_on()
                {
                    self.user_units = user_units;
                }
                if let Ok(system_units) =
                    async { units::list_units(Connection::system().await?).await }.block_on()
                {
                    self.system_units = system_units;
                }

                self.refresh = false;
            }
            egui::Panel::left("User units").show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("User units");
                });
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.unit_list(true, ui, || {
                        async { Connection::session().await }.block_on()
                    })
                })
            });
            egui::Panel::right("System units").show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("System units");
                });
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.unit_list(false, ui, || {
                        async {
                            Builder::system()?
                                .auth_mechanism(zbus::AuthMechanism::External)
                                .build()
                                .await
                        }
                        .block_on()
                    })
                })
            });
            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Unit log");
                });
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(egui::Label::new(self.unit_log.clone().as_str()).wrap());
                })
            });
        });
    }
}

impl SystemControlApp {
    fn unit_list(
        &mut self,
        user: bool,
        ui: &mut egui::Ui,
        connection: fn() -> zbus::Result<Connection>,
    ) {
        for (name, unit) in if user {
            &self.user_units
        } else {
            &self.system_units
        } {
            if !name.contains(self.unit_filter.as_str()) {
                continue;
            }
            match unit.active.clone().as_str() {
                _ if self.show_inactive => {}
                "running" | "active" | "mounted" | "plugged" => {}
                _ => continue,
            }
            let id = ui.make_persistent_id(name.clone().as_str().to_unescaped().unwrap());
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
                .show_header(ui, |ui| match unit.active.clone().as_str() {
                    "running" | "active" | "mounted" | "plugged" if self.show_inactive => ui.add(
                        egui::Label::new(
                            egui::RichText::new(name.clone().as_str().to_unescaped().unwrap())
                                .underline(),
                        )
                        .wrap(),
                    ),
                    _ => ui.add(
                        egui::Label::new(name.clone().as_str().to_unescaped().unwrap()).wrap(),
                    ),
                })
                .body(|ui| {
                    ui.label(format!(
                        "{}, {}, {}. {}",
                        unit.description.clone(),
                        unit.loaded.clone(),
                        unit.active.clone(),
                        unit.subunit.clone()
                    ));
                    ui.horizontal(|ui| {
                        if ui.button("Enable").clicked() {
                            let _ = async {
                                let connection = connection()?;
                                units::enable(connection, vec![name.clone()]).await
                            }
                            .block_on();
                            self.refresh = true;
                        }
                        if ui.button("Disable").clicked() {
                            let _ = async {
                                let connection = connection()?;
                                units::disable(connection, vec![name.clone()]).await
                            }
                            .block_on();
                            self.refresh = true;
                        }
                        if ui.button("Start").clicked() {
                            let _ = async {
                                let connection = connection()?;
                                units::start(connection, name.clone()).await
                            }
                            .block_on();
                            self.refresh = true;
                        }
                        if ui.button("Stop").clicked() {
                            let _ = async {
                                let connection = connection()?;
                                units::stop(connection, name.clone()).await
                            }
                            .block_on();
                            self.refresh = true;
                        }
                        if ui.button("Logs").clicked() {
                            if user {
                                match Command::new("journalctl")
                                    .args(["--user", "-b0", "-u", name.clone().as_str()])
                                    .output()
                                {
                                    Ok(output) => {
                                        if let Ok(output) = String::from_utf8(output.stdout) {
                                            self.unit_log = output;
                                        }
                                    }
                                    _ => {}
                                }
                            } else {
                                match Command::new("journalctl")
                                    .args(["-b0", "-u", name.clone().as_str()])
                                    .output()
                                {
                                    Ok(output) => {
                                        if let Ok(output) = String::from_utf8(output.stdout) {
                                            self.unit_log = output;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    })
                });
        }
    }
}
