mod units;

use std::{collections::BTreeMap, process::Command, vec};

use descape::UnescapeExt;
use eframe::egui;
use zbus::blocking::Connection;

use crate::units::UnitInfo;

fn main() -> eframe::Result {
    eframe::run_native(
        "System Control",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(SystemControlApp::new(cc)))),
    )
}

#[derive(Default)]
struct SystemControlApp {
    user_units: BTreeMap<String, UnitInfo>,
    system_units: BTreeMap<String, UnitInfo>,
    unit_log: String,
    refresh: bool,
    show_inactive: bool,
    show_failed_only: bool,
    name_filter: String,
    type_filter: TypeFilter,
}

impl SystemControlApp {
    fn new(_: &eframe::CreationContext<'_>) -> Self {
        Self {
            refresh: true,
            show_inactive: true,
            ..Default::default()
        }
    }
}

#[derive(Default, PartialEq)]
pub enum TypeFilter {
    #[default]
    All,
    Services,
    Sockets,
    Devices,
    Mounts,
    Automounts,
    Swaps,
    Targets,
    Paths,
    Timers,
    Slices,
    Scopes,
}

impl eframe::App for SystemControlApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        egui::Panel::top("Menu bar").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_inactive, "Inactive");
                    ui.checkbox(&mut self.show_failed_only, "Failed");
                    ui.separator();
                    ui.radio_value(&mut self.type_filter, TypeFilter::All, "All Types");
                    ui.radio_value(&mut self.type_filter, TypeFilter::Services, "Services");
                    ui.radio_value(&mut self.type_filter, TypeFilter::Sockets, "Sockets");
                    ui.radio_value(&mut self.type_filter, TypeFilter::Devices, "Devices");
                    ui.radio_value(&mut self.type_filter, TypeFilter::Mounts, "Mounts");
                    ui.radio_value(&mut self.type_filter, TypeFilter::Automounts, "Automounts");
                    ui.radio_value(&mut self.type_filter, TypeFilter::Swaps, "Swaps");
                    ui.radio_value(&mut self.type_filter, TypeFilter::Targets, "Targets");
                    ui.radio_value(&mut self.type_filter, TypeFilter::Paths, "Paths");
                    ui.radio_value(&mut self.type_filter, TypeFilter::Timers, "Timers");
                    ui.radio_value(&mut self.type_filter, TypeFilter::Slices, "Slices");
                    ui.radio_value(&mut self.type_filter, TypeFilter::Scopes, "Scopes");
                });
                ui.add(egui::TextEdit::singleline(&mut self.name_filter).hint_text("Filter..."));
                self.refresh |= ui.button("Refresh").clicked();
            });
        });
        if self.refresh {
            if let Ok(connection) = Connection::session()
                && let Ok(user_units) = units::list_units(connection)
            {
                self.user_units = user_units;
            }
            if let Ok(connection) = Connection::system()
                && let Ok(system_units) = units::list_units(connection)
            {
                self.system_units = system_units;
            }

            self.refresh = false;
        }
        egui::Panel::left("User units").show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("User units");
            });
            egui::ScrollArea::vertical()
                .show(ui, |ui| self.unit_list(true, ui, Connection::session))
        });
        egui::Panel::right("System units").show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("System units");
            });
            egui::ScrollArea::vertical()
                .show(ui, |ui| self.unit_list(false, ui, Connection::system))
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
        connection_fn: fn() -> zbus::Result<Connection>,
    ) {
        for (name, unit) in if user {
            &self.user_units
        } else {
            &self.system_units
        } {
            if !name.contains(&self.name_filter)
                || (self.type_filter == TypeFilter::Services && !name.ends_with(".service"))
                || (self.type_filter == TypeFilter::Sockets && !name.ends_with(".socket"))
                || (self.type_filter == TypeFilter::Devices && !name.ends_with(".device"))
                || (self.type_filter == TypeFilter::Mounts && !name.ends_with(".mount"))
                || (self.type_filter == TypeFilter::Automounts && !name.ends_with(".automount"))
                || (self.type_filter == TypeFilter::Swaps && !name.ends_with(".swap"))
                || (self.type_filter == TypeFilter::Targets && !name.ends_with(".target"))
                || (self.type_filter == TypeFilter::Paths && !name.ends_with(".path"))
                || (self.type_filter == TypeFilter::Timers && !name.ends_with(".timer"))
                || (self.type_filter == TypeFilter::Slices && !name.ends_with(".slice"))
                || (self.type_filter == TypeFilter::Scopes && !name.ends_with(".scope"))
            {
                continue;
            }
            match unit.active.as_str() {
                "failed" => {}
                _ if self.show_failed_only => continue,
                _ if self.show_inactive => {}
                "active" => {}
                _ => continue,
            }
            let name_u = name.to_unescaped().unwrap();
            let id = ui.make_persistent_id(name_u.clone());
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
                .show_header(ui, |ui| match unit.active.as_str() {
                    "active" if self.show_inactive => {
                        ui.add(egui::Label::new(egui::RichText::new(name_u).strong()).wrap())
                    }
                    "failed" if !self.show_failed_only => {
                        ui.add(egui::Label::new(egui::RichText::new(name_u).underline()).wrap())
                    }
                    _ => ui.add(egui::Label::new(name_u).wrap()),
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
                            let _ = units::enable(connection_fn, vec![name.clone()]);
                            self.refresh = true;
                        }
                        if ui.button("Disable").clicked() {
                            let _ = units::disable(connection_fn, vec![name.clone()]);
                            self.refresh = true;
                        }
                        if ui.button("Start").clicked() {
                            let _ = units::start(connection_fn, name.clone());
                            self.refresh = true;
                        }
                        if ui.button("Stop").clicked() {
                            let _ = units::stop(connection_fn, name.clone());
                            self.refresh = true;
                        }
                        if ui.button("Logs").clicked() {
                            let _ = Command::new("journalctl")
                                .args(if user {
                                    vec!["--user", "-b0", "-u", &name]
                                } else {
                                    vec!["-b0", "-u", &name]
                                })
                                .output()
                                .inspect(|output| {
                                    if let Ok(output) = String::from_utf8(output.stdout.clone()) {
                                        self.unit_log = output;
                                    }
                                });
                        }
                    })
                });
        }
    }
}
