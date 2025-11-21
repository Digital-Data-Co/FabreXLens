use crate::services::api::{
    fabrex::UsageAlert, FabrexEndpoint, FabrexFabric, FabrexUsage, GryfWorkload, SupernodeNode,
};
use eframe::egui::{self, Color32, RichText, TextStyle};
use std::time::Instant;

#[derive(Debug, Clone, Default)]
pub struct DashboardSnapshot {
    pub fabrics: Vec<FabrexFabric>,
    pub fabric_usage: Vec<FabrexUsage>,
    pub workloads: Vec<GryfWorkload>,
    pub supernodes: Vec<SupernodeNode>,
    pub endpoints: Vec<FabrexEndpoint>,
    pub alerts: Vec<String>,
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
        ui.add_space(24.0);
        ui.centered_and_justified(|ui| {
            ui.spinner();
            ui.label("Fetching latest telemetry...");
        });
        return;
    }

    if let Some(error) = state.error() {
        let frame = egui::Frame::group(ui.style())
            .fill(ui.visuals().error_fg_color.linear_multiply(0.09))
            .stroke(egui::Stroke::new(1.0, ui.visuals().error_fg_color))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(12, 12));
        frame.show(ui, |ui| {
            ui.label(RichText::new("Telemetry refresh failed").heading());
            ui.label(error);
        });
        ui.add_space(16.0);
    }

    let snapshot = state.snapshot();
    render_summary_cards(ui, snapshot, state.last_updated());
    ui.add_space(18.0);

    render_fabric_section(ui, snapshot);
    ui.add_space(16.0);
    render_utilization_section(ui, snapshot);
    ui.add_space(16.0);
    render_workloads_section(ui, snapshot);
    ui.add_space(16.0);
    render_supernodes_section(ui, snapshot);

    if !snapshot.alerts.is_empty() {
        ui.add_space(16.0);
        render_global_alerts(ui, snapshot);
    }
}

fn render_summary_cards(
    ui: &mut egui::Ui,
    snapshot: &DashboardSnapshot,
    last_updated: Option<Instant>,
) {
    let avg_util = average_utilization(snapshot);
    let alerts = snapshot
        .fabric_usage
        .iter()
        .fold(0usize, |acc, usage| acc + usage.alerts.len());

    let last_refresh = last_updated.map(|t| t.elapsed().as_secs());

    let cards = vec![
        SummaryCard::new(
            "Fabrics",
            snapshot.fabrics.len().to_string(),
            "Managed fabrics",
            Color32::from_rgb(45, 110, 230),
        ),
        SummaryCard::new(
            "Workloads",
            snapshot.workloads.len().to_string(),
            "Active or pending jobs",
            Color32::from_rgb(120, 94, 210),
        ),
        SummaryCard::new(
            "Supernodes",
            snapshot.supernodes.len().to_string(),
            "Cluster control nodes",
            Color32::from_rgb(33, 150, 83),
        ),
        SummaryCard::new(
            "Avg utilization",
            if let Some(util) = avg_util {
                format!("{util:.1}%")
            } else {
                "—".into()
            },
            "Across active fabrics",
            Color32::from_rgb(236, 146, 36),
        ),
        SummaryCard::new(
            "Alerts",
            alerts.to_string(),
            "Open notices",
            Color32::from_rgb(225, 85, 73),
        ),
        SummaryCard::new(
            "Last refresh",
            last_refresh
                .map(|secs| format!("{}s ago", secs))
                .unwrap_or_else(|| "—".into()),
            "Telemetry snapshot age",
            Color32::from_rgb(86, 104, 120),
        ),
    ];

    ui.horizontal_wrapped(|ui| {
        for card in cards {
            summary_card(ui, &card);
            ui.add_space(12.0);
        }
    });
}

fn render_fabric_section(ui: &mut egui::Ui, snapshot: &DashboardSnapshot) {
    section(ui, "Fabric topology", |ui| {
        egui::Grid::new("fabric_grid")
            .striped(true)
            .spacing(egui::vec2(12.0, 8.0))
            .show(ui, |ui| {
                ui.label(RichText::new("Fabric").strong());
                ui.label(RichText::new("Status").strong());
                ui.label(RichText::new("Description").strong());
                ui.end_row();

                for fabric in &snapshot.fabrics {
                    ui.label(&fabric.name);
                    status_chip(ui, &fabric.status, status_color(&fabric.status));
                    ui.label(fabric.description.as_deref().unwrap_or("—"));
                    ui.end_row();
                }

                if snapshot.fabrics.is_empty() {
                    ui.colored_label(Color32::GRAY, "No fabrics available");
                    ui.end_row();
                }
            });
    });
}

fn render_utilization_section(ui: &mut egui::Ui, snapshot: &DashboardSnapshot) {
    section(ui, "Resource utilization", |ui| {
        if snapshot.fabric_usage.is_empty() {
            ui.colored_label(Color32::GRAY, "No usage metrics reported yet.");
            return;
        }

        for usage in &snapshot.fabric_usage {
            ui.vertical(|ui| {
                let fabric_name = snapshot
                    .fabrics
                    .iter()
                    .find(|fabric| fabric.id == usage.fabric_id)
                    .map(|fabric| fabric.name.as_str())
                    .unwrap_or(&usage.fabric_id);

                let utilization = (usage.utilization_percent / 100.0).clamp(0.0, 1.0);
                let fill_color = utilization_color(usage.utilization_percent);
                let text = format!(
                    "{fabric_name} • {:.1}% ({}/{})",
                    usage.utilization_percent, usage.assigned_endpoints, usage.total_endpoints
                );

                let progress = egui::ProgressBar::new(utilization as f32)
                    .desired_width(ui.available_width())
                    .text(text)
                    .fill(fill_color);
                ui.add(progress);

                if !usage.alerts.is_empty() {
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        for alert in &usage.alerts {
                            alert_chip(ui, alert.severity.as_str(), &alert.message);
                        }
                    });
                }

                ui.add_space(8.0);
            });
        }
    });
}

fn render_workloads_section(ui: &mut egui::Ui, snapshot: &DashboardSnapshot) {
    section(ui, "Active workloads", |ui| {
        if snapshot.workloads.is_empty() {
            ui.colored_label(Color32::GRAY, "No workloads reported.");
            return;
        }

        egui::Grid::new("workload_grid")
            .striped(true)
            .spacing(egui::vec2(12.0, 8.0))
            .show(ui, |ui| {
                ui.label(RichText::new("Workload").strong());
                ui.label(RichText::new("State").strong());
                ui.label(RichText::new("Owner").strong());
                ui.end_row();

                for workload in &snapshot.workloads {
                    ui.label(&workload.name);
                    status_chip(ui, &workload.state, status_color(&workload.state));
                    ui.label(workload.owner.as_deref().unwrap_or("—"));
                    ui.end_row();
                }
            });
    });
}

fn render_supernodes_section(ui: &mut egui::Ui, snapshot: &DashboardSnapshot) {
    section(ui, "Supernodes", |ui| {
        if snapshot.supernodes.is_empty() {
            ui.colored_label(Color32::GRAY, "No supernodes discovered.");
            return;
        }

        egui::Grid::new("supernode_grid")
            .striped(true)
            .spacing(egui::vec2(12.0, 8.0))
            .show(ui, |ui| {
                ui.label(RichText::new("Node").strong());
                ui.label(RichText::new("Role").strong());
                ui.label(RichText::new("Status").strong());
                ui.end_row();

                for node in &snapshot.supernodes {
                    ui.label(&node.name);
                    ui.label(&node.role);
                    status_chip(ui, &node.status, status_color(&node.status));
                    ui.end_row();
                }
            });
    });
}

fn render_global_alerts(ui: &mut egui::Ui, snapshot: &DashboardSnapshot) {
    section(ui, "Alerts", |ui| {
        for entry in &snapshot.fabric_usage {
            for alert in &entry.alerts {
                let severity_color = match alert.severity.to_lowercase().as_str() {
                    "critical" | "error" => Color32::from_rgb(225, 85, 73),
                    "warning" => Color32::from_rgb(236, 146, 36),
                    _ => Color32::from_rgb(86, 104, 120),
                };
                alert_row(ui, &entry.fabric_id, alert, severity_color);
            }
        }

        for alert in &snapshot.alerts {
            let severity_color = Color32::from_rgb(86, 104, 120);
            let frame = egui::Frame::group(ui.style())
                .fill(severity_color.linear_multiply(0.1))
                .corner_radius(egui::CornerRadius::same(6))
                .inner_margin(egui::Margin::symmetric(10, 6));
            frame.show(ui, |ui| {
                ui.label(alert);
            });
            ui.add_space(6.0);
        }

        if snapshot.alerts.is_empty()
            && snapshot
                .fabric_usage
                .iter()
                .all(|usage| usage.alerts.is_empty())
        {
            ui.colored_label(
                Color32::from_rgb(70, 140, 90),
                "No active alerts – all systems nominal.",
            );
        }
    });
}

fn section(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.label(RichText::new(title).text_style(TextStyle::Name("Title".into())));
    ui.add_space(6.0);
    let frame = egui::Frame::group(ui.style())
        .fill(ui.visuals().extreme_bg_color)
        .stroke(egui::Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(14, 12));
    frame.show(ui, add_contents);
}

fn summary_card(ui: &mut egui::Ui, card: &SummaryCard) {
    let frame = egui::Frame::group(ui.style())
        .fill(card.accent.linear_multiply(0.1))
        .stroke(egui::Stroke::new(1.0, card.accent.linear_multiply(0.9)))
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(14, 12));
    frame.show(ui, |ui| {
        ui.style_mut().spacing.item_spacing = egui::vec2(4.0, 4.0);
        ui.label(RichText::new(&card.title).text_style(TextStyle::Name("Title".into())));
        ui.label(
            RichText::new(&card.value)
                .text_style(TextStyle::Heading)
                .color(card.accent.linear_multiply(0.95)),
        );
        ui.label(
            RichText::new(&card.subtitle)
                .text_style(TextStyle::Small)
                .color(card.accent.linear_multiply(0.9)),
        );
    });
}

fn status_chip(ui: &mut egui::Ui, text: &str, color: Color32) {
    let frame = egui::Frame::new()
        .fill(color.linear_multiply(0.12))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(8, 4));
    frame.show(ui, |ui| {
        ui.label(
            RichText::new(text)
                .text_style(TextStyle::Small)
                .color(color.linear_multiply(1.2)),
        );
    });
}

fn alert_chip(ui: &mut egui::Ui, severity: &str, message: &str) {
    let severity_color = match severity.to_lowercase().as_str() {
        "critical" | "error" => Color32::from_rgb(225, 85, 73),
        "warning" => Color32::from_rgb(236, 146, 36),
        _ => Color32::from_rgb(86, 104, 120),
    };

    let frame = egui::Frame::new()
        .fill(severity_color.linear_multiply(0.13))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(8, 4));
    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(severity.to_uppercase())
                    .color(severity_color)
                    .text_style(TextStyle::Small),
            );
            ui.label(RichText::new(message).text_style(TextStyle::Small));
        });
    });
}

fn alert_row(ui: &mut egui::Ui, fabric_id: &str, alert: &UsageAlert, color: Color32) {
    let frame = egui::Frame::group(ui.style())
        .fill(color.linear_multiply(0.1))
        .stroke(egui::Stroke::new(1.0, color.linear_multiply(0.7)))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(12, 10));
    frame.show(ui, |ui| {
        ui.vertical(|ui| {
            ui.label(
                RichText::new(format!("{fabric_id} • {}", alert.severity.to_uppercase()))
                    .color(color)
                    .text_style(TextStyle::Button),
            );
            ui.label(&alert.message);
        });
    });
    ui.add_space(6.0);
}

fn status_color(status: &str) -> Color32 {
    match status.to_lowercase().as_str() {
        "healthy" | "online" | "running" => Color32::from_rgb(33, 150, 83),
        "warning" | "degraded" | "pending" => Color32::from_rgb(236, 146, 36),
        "error" | "critical" | "offline" => Color32::from_rgb(225, 85, 73),
        _ => Color32::from_rgb(86, 104, 120),
    }
}

fn utilization_color(util_percent: f64) -> Color32 {
    if util_percent >= 85.0 {
        Color32::from_rgb(225, 85, 73)
    } else if util_percent >= 65.0 {
        Color32::from_rgb(236, 146, 36)
    } else {
        Color32::from_rgb(45, 110, 230)
    }
}

fn average_utilization(snapshot: &DashboardSnapshot) -> Option<f64> {
    if snapshot.fabric_usage.is_empty() {
        None
    } else {
        Some(
            snapshot
                .fabric_usage
                .iter()
                .map(|usage| usage.utilization_percent)
                .sum::<f64>()
                / snapshot.fabric_usage.len() as f64,
        )
    }
}

struct SummaryCard {
    title: String,
    value: String,
    subtitle: String,
    accent: Color32,
}

impl SummaryCard {
    fn new(title: &str, value: String, subtitle: &str, accent: Color32) -> Self {
        Self {
            title: title.into(),
            value,
            subtitle: subtitle.into(),
            accent,
        }
    }
}
