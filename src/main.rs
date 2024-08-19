mod manage;
mod units;

use std::{
    collections::BTreeSet, time::Duration, vec
};

use descape::UnescapeExt;
use eframe::egui::{self, CollapsingHeader};
use pollster::FutureExt;
use tokio::runtime::Runtime;
use units::list_units;
use zbus::zvariant::ObjectPath;

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
    units: BTreeSet<OwnableUnitInfo>,
    refresh: bool,
}

impl SystemControlApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self { refresh: true, units: BTreeSet::new() }
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub struct OwnableUnitInfo {
    name: String,
    description: String,
    loaded: String,
    active: String,
    subunit: String,
}

type UnitInfo<'a> = Vec<(
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

impl eframe::App for SystemControlApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.refresh || ui.button("Refresh").clicked() {
                let unsorted_units = async { list_units().await.unwrap() }.block_on();
                let unsorted_units = unsorted_units.deserialize::<UnitInfo>().unwrap();
                for unit in unsorted_units {
                    self.units.insert(OwnableUnitInfo {name: unit.0.clone(), description: unit.2.clone(), loaded: unit.3.clone(), active: unit.4.clone(), subunit: unit.5.clone()});
                }
                self.refresh = false;
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
                for unit in &self.units
                {
                    CollapsingHeader::new(unit.name.clone().as_str().to_unescaped().unwrap())
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.label(format!("{}, {}, {}. {}", unit.description.clone(), unit.loaded.clone(), unit.active.clone(), unit.subunit.clone()));
                            ui.horizontal(|ui| {
                                if ui.button("Enable").clicked() {
                                    let _ = async { manage::enable(vec![unit.name.clone()]).await }
                                        .block_on();
                                    self.refresh = true;
                                }
                                if ui.button("Disable").clicked() {
                                    let _ = async { manage::disable(vec![unit.name.clone()]).await }
                                        .block_on();
                                    self.refresh = true;
                                }
                                if ui.button("Start").clicked() {
                                    let _ = async { manage::start(unit.name.clone()).await }
                                        .block_on();
                                    self.refresh = true;
                                }
                                if ui.button("Stop").clicked() {
                                    let _ = async { manage::stop(unit.name.clone()).await }
                                        .block_on();
                                    self.refresh = true;
                                }
                            })
                        });
                }
            });
        });
    }
}
