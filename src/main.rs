mod units;

use std::{collections::BTreeMap, error::Error, process::Command, time::Duration, vec};

use descape::UnescapeExt;
use eframe::egui;
use pollster::FutureExt;
use tokio::runtime::Runtime;
use zbus::Connection;

use crate::units::UnitInfo;

fn main() -> Result<(), Box<dyn Error>> {
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
    hide_success: bool,
    unit_filter: String,
    unit_type_filters: [bool; 11],
}

impl SystemControlApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            refresh: true,
            user_units: BTreeMap::new(),
            system_units: BTreeMap::new(),
            unit_log: String::new(),
            show_inactive: true,
            hide_success: false,
            unit_filter: String::new(),
            unit_type_filters: [true; 11],
        }
    }
}

impl eframe::App for SystemControlApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("Menu bar").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_inactive, "Inactive");
                    ui.checkbox(&mut self.hide_success, "Failed");
                    ui.separator();
                    ui.radio_value(&mut self.unit_type_filters, [true; 11], "All Types");
                    ui.radio_value(
                        &mut self.unit_type_filters,
                        [
                            true, false, false, false, false, false, false, false, false, false,
                            false,
                        ],
                        "Services",
                    );
                    ui.radio_value(
                        &mut self.unit_type_filters,
                        [
                            false, true, false, false, false, false, false, false, false, false,
                            false,
                        ],
                        "Sockets",
                    );
                    ui.radio_value(
                        &mut self.unit_type_filters,
                        [
                            false, false, true, false, false, false, false, false, false, false,
                            false,
                        ],
                        "Devices",
                    );
                    ui.radio_value(
                        &mut self.unit_type_filters,
                        [
                            false, false, false, true, false, false, false, false, false, false,
                            false,
                        ],
                        "Mounts",
                    );
                    ui.radio_value(
                        &mut self.unit_type_filters,
                        [
                            false, false, false, false, true, false, false, false, false, false,
                            false,
                        ],
                        "Automounts",
                    );
                    ui.radio_value(
                        &mut self.unit_type_filters,
                        [
                            false, false, false, false, false, true, false, false, false, false,
                            false,
                        ],
                        "Swaps",
                    );
                    ui.radio_value(
                        &mut self.unit_type_filters,
                        [
                            false, false, false, false, false, false, true, false, false, false,
                            false,
                        ],
                        "Targets",
                    );
                    ui.radio_value(
                        &mut self.unit_type_filters,
                        [
                            false, false, false, false, false, false, false, true, false, false,
                            false,
                        ],
                        "Paths",
                    );
                    ui.radio_value(
                        &mut self.unit_type_filters,
                        [
                            false, false, false, false, false, false, false, false, true, false,
                            false,
                        ],
                        "Timers",
                    );
                    ui.radio_value(
                        &mut self.unit_type_filters,
                        [
                            false, false, false, false, false, false, false, false, false, true,
                            false,
                        ],
                        "Slices",
                    );
                    ui.radio_value(
                        &mut self.unit_type_filters,
                        [
                            false, false, false, false, false, false, false, false, false, false,
                            true,
                        ],
                        "Scopes",
                    );
                });
                ui.add(egui::TextEdit::singleline(&mut self.unit_filter).hint_text("Filter..."));
                self.refresh |= ui.button("Refresh").clicked();
            });
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
                    async { Connection::system().await }.block_on()
                })
            })
        });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Unit log");
            });
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(egui::Label::new(&self.unit_log).wrap());
            })
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
            if !name.contains(&self.unit_filter)
                || (!self.unit_type_filters[0] && name.ends_with(".service"))
                || (!self.unit_type_filters[1] && name.ends_with(".socket"))
                || (!self.unit_type_filters[2] && name.ends_with(".device"))
                || (!self.unit_type_filters[3] && name.ends_with(".mount"))
                || (!self.unit_type_filters[4] && name.ends_with(".automount"))
                || (!self.unit_type_filters[5] && name.ends_with(".swap"))
                || (!self.unit_type_filters[6] && name.ends_with(".target"))
                || (!self.unit_type_filters[7] && name.ends_with(".path"))
                || (!self.unit_type_filters[8] && name.ends_with(".timer"))
                || (!self.unit_type_filters[9] && name.ends_with(".slice"))
                || (!self.unit_type_filters[10] && name.ends_with(".scope"))
            {
                continue;
            }
            match unit.active.as_str() {
                "failed" => {}
                _ if self.hide_success => continue,
                _ if self.show_inactive => {}
                "active" => {}
                _ => continue,
            }
            let id = ui.make_persistent_id(name.to_unescaped().unwrap());
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
                .show_header(ui, |ui| match unit.active.as_str() {
                    "active" if self.show_inactive => ui.add(
                        egui::Label::new(
                            egui::RichText::new(name.to_unescaped().unwrap()).strong(),
                        )
                        .wrap(),
                    ),
                    "failed" if !self.hide_success => ui.add(
                        egui::Label::new(
                            egui::RichText::new(name.to_unescaped().unwrap()).underline(),
                        )
                        .wrap(),
                    ),
                    _ => ui.add(egui::Label::new(name.to_unescaped().unwrap()).wrap()),
                })
                .body(|ui| {
                    ui.add(egui::Label::new(&unit.description).wrap());
                    ui.add(
                        egui::Label::new(format!(
                            "{}, {}, {}. {}",
                            unit.loaded, unit.active, unit.substate, unit.subunit,
                        ))
                        .wrap(),
                    );
                    ui.horizontal_wrapped(|ui| {
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
                                    .args(["--user", "-b0", "-u", &name])
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
                                    .args(["-b0", "-u", &name])
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
