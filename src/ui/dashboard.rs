use crate::services::api::{
    FabrexEndpoint,
    FabrexFabric,
    FabrexUsage,
    GryfWorkload,
    SupernodeNode,
};
use eframe::egui;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct DashboardSnapshot {
    pub fabrics: Vec<FabrexFabric>,
    pub fabric_usage: Vec<FabrexUsage>,
    pub workloads: Vec<GryfWorkload>,
    pub supernodes: Vec<SupernodeNode>,
    pub endpoints: Vec<FabrexEndpoint>,
    pub alerts: Vec<String>,
}

impl Default for DashboardSnapshot {
    fn default() -> Self {
        Self {
            fabrics: Vec::new(),
            fabric_usage: Vec::new(),
            workloads: Vec::new(),
            supernodes: Vec::new(),
            endpoints: Vec::new(),
            alerts: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct DashboardState {
    snapshot: DashboardSnapshot,
    last_updated: Option<Instant>,
    loading: bool,
    error: Option<String>,
}

impl DashboardState {
    pub fn new() -> Self {
        Self {
            snapshot: DashboardSnapshot::default(),
            last_updated: None,
            loading: true,
            error: None,
        }
    }

    pub fn set_loading(&mut self) {
        self.loading = true;
        self.error = None;
    }

    pub fn update(&mut self, snapshot: DashboardSnapshot) {
        self.snapshot = snapshot;
        self.last_updated = Some(Instant::now());
        self.loading = false;
        self.error = None;
    }

    pub fn set_error(&mut self, error: String) {
        self.loading = false;
        self.error = Some(error);
    }

    pub fn snapshot(&self) -> &DashboardSnapshot {
        &self.snapshot
    }

    pub fn last_updated(&self) -> Option<Instant> {
        self.last_updated
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

pub fn render(ui: &mut egui::Ui, state: &DashboardState) {
    if state.is_loading() {
        ui.spinner();
        ui.label("Fetching latest telemetry...");
        return;
    }

    if let Some(error) = state.error() {
        ui.colored_label(egui::Color32::LIGHT_RED, error);
        ui.separator();
    }

    let snapshot = state.snapshot();

    if let Some(updated) = state.last_updated() {
        ui.label(format!(
            "Last updated {}s ago",
            updated.elapsed().as_secs()
        ));
    }

    ui.label(format!(
        "Fabrics: {} • Workloads: {} • Supernodes: {}",
        snapshot.fabrics.len(),
        snapshot.workloads.len(),
        snapshot.supernodes.len()
    ));

    if !snapshot.fabric_usage.is_empty() {
        let average_utilization: f64 = snapshot
            .fabric_usage
            .iter()
            .map(|usage| usage.utilization_percent)
            .sum::<f64>()
            / snapshot.fabric_usage.len() as f64;
        ui.label(format!("Average fabric utilization: {:.1}%", average_utilization));
    }

    ui.separator();

    ui.heading("Fabric Overview");
    egui::Grid::new("fabrex_overview").striped(true).show(ui, |ui| {
        ui.label("Fabric");
        ui.label("Status");
        ui.label("Description");
        ui.end_row();

        for fabric in &snapshot.fabrics {
            ui.label(&fabric.name);
            ui.label(&fabric.status);
            ui.label(fabric.description.as_deref().unwrap_or("—"));
            ui.end_row();
        }

        if snapshot.fabrics.is_empty() {
            ui.colored_label(egui::Color32::GRAY, "No fabrics available");
            ui.end_row();
        }
    });

    ui.separator();
    ui.heading("Utilization");
    for usage in &snapshot.fabric_usage {
        let label = format!(
            "{} • {:.1}% utilized ({}/{})",
            usage.fabric_id,
            usage.utilization_percent,
            usage.assigned_endpoints,
            usage.total_endpoints
        );
        ui.label(label);
        if !usage.alerts.is_empty() {
            egui::CollapsingHeader::new("Alerts").show(ui, |ui| {
                for alert in &usage.alerts {
                    ui.colored_label(egui::Color32::LIGHT_RED, format!(
                        "{}: {}",
                        alert.severity.to_uppercase(),
                        alert.message
                    ));
                }
            });
        }
    }

    ui.separator();
    ui.heading("Active Workloads");
    egui::Grid::new("gryf_workloads").striped(true).show(ui, |ui| {
        ui.label("Workload");
        ui.label("State");
        ui.label("Owner");
        ui.end_row();

        for workload in &snapshot.workloads {
            ui.label(&workload.name);
            ui.label(&workload.state);
            ui.label(workload.owner.as_deref().unwrap_or("—"));
            ui.end_row();
        }

        if snapshot.workloads.is_empty() {
            ui.colored_label(egui::Color32::GRAY, "No workloads reported");
            ui.end_row();
        }
    });

    ui.separator();
    ui.heading("Supernodes");
    egui::Grid::new("supernodes").striped(true).show(ui, |ui| {
        ui.label("Node");
        ui.label("Role");
        ui.label("Status");
        ui.end_row();

        for node in &snapshot.supernodes {
            ui.label(&node.name);
            ui.label(&node.role);
            ui.label(&node.status);
            ui.end_row();
        }

        if snapshot.supernodes.is_empty() {
            ui.colored_label(egui::Color32::GRAY, "No supernodes discovered");
            ui.end_row();
        }
    });
}
