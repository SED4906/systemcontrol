mod manage;
mod units;
mod auth;

use std::{
    time::Duration,
    vec,
};

use auth::authenticate;
use descape::UnescapeExt;
use eframe::egui::{self, CollapsingHeader};
use pollster::FutureExt;
use tokio::runtime::Runtime;
use units::list_units;
use zbus::{message::Body, zvariant::ObjectPath};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new()?;
    let _enter = rt.enter();
    let _dbus_thread = rt.spawn(async { tokio::time::sleep(Duration::MAX).await });
    //let auth = async{authenticate().await}.block_on()?;
    //println!("{auth:?}");
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
    units: Option<Body>,
}

impl SystemControlApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
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
            if self.units.is_none() || ui.button("Reload Unit List").clicked() {
                self.units = Some(async { list_units().await.unwrap() }.block_on());
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut units_sorted = self
                    .units
                    .as_ref()
                    .unwrap()
                    .deserialize::<UnitInfo>()
                    .unwrap();
                units_sorted.sort_by_key(|u| u.clone().0);
                for unit in units_sorted.clone().into_iter()
                {
                    CollapsingHeader::new(unit.0.clone().as_str().to_unescaped().unwrap())
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.label(format!("{}, {}, {}. {}", unit.2.clone(), unit.3.clone(), unit.4.clone(), unit.5.clone()));
                            ui.horizontal(|ui| {
                                if ui.button("Enable").clicked() {
                                    let _ = async { manage::enable(vec![unit.0.clone()]).await }
                                        .block_on();
                                }
                                if ui.button("Disable").clicked() {
                                    let _ = async { manage::disable(vec![unit.0.clone()]).await }
                                        .block_on();
                                }
                                if ui.button("Start").clicked() {
                                    let _ = async { manage::start(unit.0.clone()).await }
                                        .block_on();
                                }
                                if ui.button("Stop").clicked() {
                                    let _ = async { manage::stop(unit.0.clone()).await }
                                        .block_on();
                                }
                            })
                        });
                }
            });
        });
    }
}
