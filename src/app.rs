use crate::config::AppConfig;
use crate::services::api::{
    ApiClientConfig, AuthContext, FabrexClient, FabrexEndpoint, FabrexReassignmentResult,
    FabrexUsage, GryfClient, SupernodeClient,
};
use crate::services::auth::{CredentialDomain, CredentialKey, CredentialManager, CredentialSecret};
use crate::ui::{apply_theme, render_dashboard, DashboardSnapshot, DashboardState};
use anyhow::{anyhow, Context, Result};
use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError};
use eframe::{egui, App, CreationContext, NativeOptions};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use tokio::time;
use tokio::try_join;

pub fn run(config: AppConfig) -> Result<()> {
    let shared_config = Arc::new(config);
    let credential_manager = Arc::new(CredentialManager::with_default_keyring());
    let app_name = shared_config.application_name.clone();
    let native_options = NativeOptions::default();

    eframe::run_native(
        &app_name,
        native_options,
        Box::new(move |cc| {
            Ok(FabreXLensApp::new(
                cc,
                shared_config.clone(),
                credential_manager.clone(),
            ))
        }),
    )
    .map_err(|err| anyhow!(err.to_string()))
}

struct FabreXLensApp {
    config: Arc<AppConfig>,
    credential_manager: Arc<CredentialManager>,
    dashboard_state: DashboardState,
    command_tx: Sender<AppCommand>,
    event_rx: Receiver<AppEvent>,
    missing_credentials: Vec<CredentialDomain>,
    reassignment_form: ReassignmentForm,
    provision_form: Option<ProvisionForm>,
    status_message: Option<String>,
    worker_failed: bool,
    polling_enabled: bool,
    poller_active: bool,
    poll_interval_secs: u64,
    dark_mode: bool,
    telemetry_log: Vec<LogEntry>,
}

impl FabreXLensApp {
    #[allow(clippy::new_ret_no_self)]
    fn new(
        cc: &CreationContext<'_>,
        config: Arc<AppConfig>,
        credential_manager: Arc<CredentialManager>,
    ) -> Box<dyn App> {
        let (command_tx, command_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();

        spawn_background_worker(
            config.clone(),
            credential_manager.clone(),
            command_rx,
            event_tx,
        );

        let dark_mode = false;
        apply_theme(&cc.egui_ctx, dark_mode);

        let poll_interval_secs = config.poll_interval_secs;
        let mut app = Self {
            config,
            credential_manager,
            dashboard_state: DashboardState::new(),
            command_tx,
            event_rx,
            missing_credentials: Vec::new(),
            reassignment_form: ReassignmentForm::default(),
            provision_form: None,
            status_message: None,
            worker_failed: false,
            polling_enabled: true,
            poller_active: false,
            poll_interval_secs,
            dark_mode,
            telemetry_log: Vec::new(),
        };

        app.refresh_missing_credentials();

        Box::new(app)
    }

    fn refresh_missing_credentials(&mut self) {
        let domains = [
            CredentialDomain::FabreX,
            CredentialDomain::Gryf,
            CredentialDomain::Supernode,
            CredentialDomain::Redfish,
        ];

        let mut missing = Vec::new();
        for domain in domains {
            let key = CredentialKey::default(domain.clone());
            match self.credential_manager.has_credentials(&key) {
                Ok(true) => {}
                Ok(false) => missing.push(domain.clone()),
                Err(err) => {
                    self.status_message =
                        Some(format!("Failed to check {domain} credentials: {err}"));
                    missing.push(domain.clone());
                }
            }
        }
        self.missing_credentials = missing;

        if self.missing_credentials.is_empty() {
            if self.polling_enabled {
                self.start_polling();
            }
            self.request_refresh();
        } else if self.poller_active {
            self.push_log(
                LogLevel::Warn,
                "Auto-refresh paused while credentials are missing.",
            );
            self.stop_polling();
        }
    }

    fn request_refresh(&mut self) {
        if !self.missing_credentials.is_empty() {
            let message = "Cannot refresh until required credentials are stored.";
            self.status_message = Some(message.into());
            self.push_log(LogLevel::Warn, message);
            return;
        }

        self.dashboard_state.set_loading();
        self.push_log(LogLevel::Info, "Manual refresh requested.");
        if let Err(err) = self.command_tx.send(AppCommand::RefreshDashboard) {
            self.worker_failed = true;
            self.status_message = Some(format!("Unable to schedule refresh: {err}"));
            self.push_log(
                LogLevel::Error,
                format!("Unable to schedule refresh: {err}"),
            );
        }
    }

    fn start_polling(&mut self) {
        if self.worker_failed
            || !self.polling_enabled
            || self.poller_active
            || !self.missing_credentials.is_empty()
        {
            return;
        }

        let interval_secs = self.poll_interval_secs.max(5);
        match self
            .command_tx
            .send(AppCommand::StartPolling { interval_secs })
        {
            Ok(_) => {
                self.poller_active = true;
                self.push_log(
                    LogLevel::Info,
                    format!("Auto-refresh started (every {interval_secs}s)"),
                );
            }
            Err(err) => {
                self.worker_failed = true;
                self.status_message = Some(format!("Unable to start auto-refresh: {err}"));
                self.push_log(
                    LogLevel::Error,
                    format!("Unable to start auto-refresh: {err}"),
                );
            }
        }
    }

    fn stop_polling(&mut self) {
        if !self.poller_active {
            return;
        }

        match self.command_tx.send(AppCommand::StopPolling) {
            Ok(_) => {
                self.poller_active = false;
                self.push_log(LogLevel::Info, "Auto-refresh stopped.".to_string());
            }
            Err(err) => {
                self.worker_failed = true;
                self.poller_active = false;
                self.status_message = Some(format!("Unable to stop auto-refresh: {err}"));
                self.push_log(
                    LogLevel::Error,
                    format!("Unable to stop auto-refresh: {err}"),
                );
            }
        }
    }

    fn update_polling(&mut self) {
        if self.worker_failed {
            return;
        }

        if !self.polling_enabled {
            self.stop_polling();
            return;
        }

        let interval_secs = self.poll_interval_secs.max(5);
        match self
            .command_tx
            .send(AppCommand::UpdatePolling { interval_secs })
        {
            Ok(_) => {
                self.poller_active = true;
                self.push_log(
                    LogLevel::Info,
                    format!("Auto-refresh interval set to {interval_secs}s"),
                );
            }
            Err(err) => {
                self.worker_failed = true;
                self.poller_active = false;
                self.status_message = Some(format!("Unable to update auto-refresh: {err}"));
                self.push_log(
                    LogLevel::Error,
                    format!("Unable to update auto-refresh: {err}"),
                );
            }
        }
    }

    fn push_log(&mut self, level: LogLevel, message: impl Into<String>) {
        let entry = LogEntry::new(level, message.into());
        self.telemetry_log.push(entry);
        const MAX_LOG_ENTRIES: usize = 200;
        if self.telemetry_log.len() > MAX_LOG_ENTRIES {
            let surplus = self.telemetry_log.len() - MAX_LOG_ENTRIES;
            self.telemetry_log.drain(0..surplus);
        }
    }

    fn render_logs(&mut self, ui: &mut egui::Ui) {
        let frame = egui::Frame::group(ui.style())
            .fill(ui.visuals().extreme_bg_color)
            .stroke(egui::Stroke::new(
                1.0,
                ui.visuals().widgets.noninteractive.bg_stroke.color,
            ))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(14, 12));

        frame.show(ui, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(10.0, 8.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Event log")
                        .text_style(egui::TextStyle::Name("Title".into())),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(format!("{} entries", self.telemetry_log.len()))
                        .text_style(egui::TextStyle::Small)
                        .color(egui::Color32::from_rgb(120, 130, 150)),
                );
            });

            if self.telemetry_log.is_empty() {
                ui.colored_label(
                    egui::Color32::from_rgb(120, 130, 150),
                    "No events captured yet.",
                );
                return;
            }

            for entry in self.telemetry_log.iter().rev() {
                let (text_color, fill_color) = log_colors(entry.level);
                let card = egui::Frame::new()
                    .fill(fill_color)
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(10, 6));
                card.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(entry.age_display())
                                .text_style(egui::TextStyle::Small)
                                .color(text_color),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new(&entry.message).text_style(egui::TextStyle::Body),
                        );
                    });
                });
            }
        });
    }

    fn consume_events(&mut self) {
        loop {
            match self.event_rx.try_recv() {
                Ok(event) => self.handle_event(event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.worker_failed = true;
                    self.poller_active = false;
                    self.status_message = Some(
                        "Background worker disconnected. Restart application after checking logs."
                            .into(),
                    );
                    self.push_log(
                        LogLevel::Error,
                        "Background worker disconnected. Restart application after checking logs.",
                    );
                    break;
                }
            }
        }
    }

    fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::DashboardUpdated(snapshot) => {
                self.dashboard_state.update(snapshot);
                self.reassignment_form
                    .on_snapshot(self.dashboard_state.snapshot());
                self.status_message = Some("Telemetry updated successfully.".into());
                self.push_log(LogLevel::Info, "Telemetry updated successfully.");
            }
            AppEvent::DashboardFailed(error) => {
                self.dashboard_state.set_error(error.clone());
                self.status_message = Some(format!("Dashboard refresh failed: {error}"));
                self.push_log(
                    LogLevel::Error,
                    format!("Dashboard refresh failed: {error}"),
                );
            }
            AppEvent::ReassignmentCompleted(result) => {
                self.reassignment_form.on_success(&result);
                self.status_message = Some(format!(
                    "Reassignment request {} {}",
                    result.request_id, result.status
                ));
                self.push_log(
                    LogLevel::Info,
                    format!(
                        "Reassignment request {} {}",
                        result.request_id, result.status
                    ),
                );
            }
            AppEvent::ReassignmentFailed(error) => {
                self.reassignment_form.on_failure(&error);
                self.status_message = Some(format!("Reassignment failed: {error}"));
                self.push_log(LogLevel::Error, format!("Reassignment failed: {error}"));
            }
        }
    }

    fn render_top_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(&self.config.application_name);
            ui.separator();
            ui.label(format!("FabreX: {}", self.config.fabrex_base_url));
            if ui.button("Refresh now").clicked() {
                self.request_refresh();
            }
            if ui.button("Re-check credentials").clicked() {
                self.refresh_missing_credentials();
                if self.missing_credentials.is_empty() {
                    self.request_refresh();
                }
            }
            ui.separator();
            let mut auto_refresh = self.polling_enabled;
            if ui.checkbox(&mut auto_refresh, "Auto-refresh").changed() {
                self.polling_enabled = auto_refresh;
                if auto_refresh {
                    self.start_polling();
                    self.request_refresh();
                } else {
                    self.stop_polling();
                }
            }

            let mut interval = self.poll_interval_secs as f64;
            if ui
                .add(
                    egui::DragValue::new(&mut interval)
                        .range(5.0..=600.0)
                        .suffix(" s"),
                )
                .changed()
            {
                self.poll_interval_secs = interval.round().max(5.0) as u64;
                self.update_polling();
            }

            let mut dark_mode = self.dark_mode;
            if ui.checkbox(&mut dark_mode, "Dark mode").changed() {
                self.dark_mode = dark_mode;
                apply_theme(ctx, dark_mode);
                self.push_log(
                    LogLevel::Info,
                    if dark_mode {
                        "Switched to dark theme"
                    } else {
                        "Switched to light theme"
                    },
                );
            }
        });

        if let Some(message) = &self.status_message {
            ui.label(message);
        }

        if let Some(updated) = self.dashboard_state.last_updated() {
            ui.label(format!(
                "Last updated {:.0}s ago",
                updated.elapsed().as_secs_f32()
            ));
        }

        if !self.missing_credentials.is_empty() {
            ui.colored_label(
                egui::Color32::YELLOW,
                format!(
                    "Missing credentials: {}",
                    self.missing_credentials
                        .iter()
                        .map(|d| d.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            );
        }

        if self.worker_failed {
            ui.colored_label(
                egui::Color32::LIGHT_RED,
                "Background worker stopped. Please restart after resolving issues.",
            );
        }
    }

    fn render_credentials_help(&mut self, ui: &mut egui::Ui) {
        let frame = egui::Frame::group(ui.style())
            .fill(ui.visuals().extreme_bg_color)
            .stroke(egui::Stroke::new(
                1.0,
                ui.visuals().widgets.noninteractive.bg_stroke.color,
            ))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(14, 12));

        frame.show(ui, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 6.0);
            ui.label(
                egui::RichText::new("Credentials required")
                    .text_style(egui::TextStyle::Name("Title".into())),
            );
            ui.label("Provide credentials for each domain to unlock live telemetry.");

            for domain in &self.missing_credentials {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{domain} credentials missing"))
                            .color(egui::Color32::from_rgb(220, 105, 39)),
                    );
                    if ui
                        .add(
                            egui::Button::new(format!("Provision {domain}"))
                                .fill(ui.visuals().selection.bg_fill),
                        )
                        .clicked()
                    {
                        self.provision_form = Some(ProvisionForm::new(domain.clone()));
                    }
                });
            }

            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(
                    "Secrets are stored in your operating system keychain and will be reused on future launches.",
                )
                .text_style(egui::TextStyle::Small)
                .color(egui::Color32::from_rgb(86, 104, 120)),
            );
        });
    }

    fn render_reassignment_panel(&mut self, ui: &mut egui::Ui) -> Option<AppCommand> {
        let snapshot = self.dashboard_state.snapshot();
        let command = self.reassignment_form.render(ui, snapshot);
        if let Some(AppCommand::SubmitReassignment {
            endpoint_id,
            target_supernode,
            ..
        }) = &command
        {
            self.push_log(
                LogLevel::Info,
                format!(
                    "Submitting reassignment for endpoint {endpoint_id} toward supernode {target_supernode}"
                ),
            );
        }
        command
    }

    fn render_provision_window(&mut self, ctx: &egui::Context) {
        let mut outcome = ProvisionOutcome::None;
        {
            if let Some(form) = self.provision_form.as_mut() {
                let domain = form.domain.clone();
                let mut open = true;
                egui::Window::new(format!("Provision {domain} credentials"))
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .resizable(false)
                    .collapsible(false)
                    .default_width(360.0)
                    .open(&mut open)
                    .show(ctx, |ui| match form.ui(ui) {
                        ProvisionUiEvent::Submit(secret) => {
                            outcome = ProvisionOutcome::Submit(domain.clone(), secret);
                        }
                        ProvisionUiEvent::Cancel => {
                            outcome = ProvisionOutcome::Cancel;
                        }
                        ProvisionUiEvent::None => {}
                    });

                if !open {
                    outcome = ProvisionOutcome::Cancel;
                }
            }
        }

        match outcome {
            ProvisionOutcome::Submit(domain, secret) => {
                self.provision_form = None;
                let key = CredentialKey::default(domain.clone());
                match self.credential_manager.set_credentials(&key, &secret) {
                    Ok(()) => {
                        self.push_log(LogLevel::Info, format!("Stored credentials for {domain}"));
                        self.status_message = Some(format!("Stored credentials for {domain}"));
                        self.refresh_missing_credentials();
                    }
                    Err(err) => {
                        self.push_log(
                            LogLevel::Error,
                            format!("Failed to store {domain} credentials: {err}"),
                        );
                        let mut retry = ProvisionForm::new(domain.clone());
                        retry.username = secret.username.clone();
                        retry.password = secret.password.clone();
                        retry.api_token = secret.api_token.clone().unwrap_or_default();
                        retry.error = Some(format!("Unable to store credentials: {err}"));
                        self.provision_form = Some(retry);
                    }
                }
            }
            ProvisionOutcome::Cancel => {
                self.provision_form = None;
            }
            ProvisionOutcome::None => {}
        }
    }
}

impl App for FabreXLensApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.consume_events();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.render_top_bar(ctx, ui);
        });

        let mut pending_command: Option<AppCommand> = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("main_scroll")
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    if !self.missing_credentials.is_empty() {
                        self.render_credentials_help(ui);
                        ui.add_space(16.0);
                    }

                    render_dashboard(ui, &self.dashboard_state);
                    ui.add_space(20.0);

                    if let Some(command) = self.render_reassignment_panel(ui) {
                        pending_command = Some(command);
                    }

                    ui.add_space(20.0);
                    self.render_logs(ui);
                });
        });

        if let Some(command) = pending_command {
            if let Err(err) = self.command_tx.send(command) {
                self.worker_failed = true;
                self.status_message = Some(format!("Failed to schedule reassignment: {err}"));
            }
        }

        self.render_provision_window(ctx);
    }
}

#[derive(Debug, Clone, Default)]
struct ReassignmentForm {
    selected_fabric: Option<String>,
    selected_endpoint: Option<String>,
    target_supernode: Option<String>,
    status: Option<String>,
    busy: bool,
}

impl ReassignmentForm {
    fn on_snapshot(&mut self, snapshot: &DashboardSnapshot) {
        if let Some(fabric_id) = self.selected_fabric.clone() {
            if !snapshot.fabrics.iter().any(|fabric| fabric.id == fabric_id) {
                self.selected_fabric = None;
                self.selected_endpoint = None;
            }
        }
        if let Some(endpoint_id) = self.selected_endpoint.clone() {
            if !snapshot.endpoints.iter().any(|ep| ep.id == endpoint_id) {
                self.selected_endpoint = None;
            }
        }
        if let Some(target_id) = self.target_supernode.clone() {
            if !snapshot.supernodes.iter().any(|node| node.id == target_id) {
                self.target_supernode = None;
            }
        }

        self.ensure_defaults(snapshot);
    }

    fn on_success(&mut self, result: &FabrexReassignmentResult) {
        self.busy = false;
        self.status = Some(
            result
                .message
                .clone()
                .unwrap_or_else(|| format!("Reassignment status: {}", result.status)),
        );
    }

    fn on_failure(&mut self, error: &str) {
        self.busy = false;
        self.status = Some(format!("Reassignment failed: {error}"));
    }

    fn ensure_defaults(&mut self, snapshot: &DashboardSnapshot) {
        if self.selected_fabric.is_none() {
            self.selected_fabric = snapshot.fabrics.first().map(|fabric| fabric.id.clone());
        }
        if self.target_supernode.is_none() {
            self.target_supernode = snapshot.supernodes.first().map(|node| node.id.clone());
        }
        if self.selected_endpoint.is_none() {
            if let Some(fabric_id) = &self.selected_fabric {
                self.selected_endpoint = snapshot
                    .endpoints
                    .iter()
                    .find(|ep| ep.fabric_id.as_deref() == Some(fabric_id.as_str()))
                    .map(|ep| ep.id.clone());
            }
        }
    }

    fn render(&mut self, ui: &mut egui::Ui, snapshot: &DashboardSnapshot) -> Option<AppCommand> {
        ui.heading("Endpoint reassignment");
        self.ensure_defaults(snapshot);

        if snapshot.fabrics.is_empty() {
            ui.label("No fabrics available.");
            return None;
        }

        let mut command = None;

        egui::ComboBox::from_label("Fabric")
            .selected_text(self.fabric_label(snapshot))
            .show_ui(ui, |ui| {
                for fabric in &snapshot.fabrics {
                    let selected = Some(fabric.id.clone()) == self.selected_fabric;
                    if ui.selectable_label(selected, &fabric.name).clicked() {
                        self.selected_fabric = Some(fabric.id.clone());
                        self.selected_endpoint = None;
                    }
                }
            });

        let endpoints_for_fabric = self.endpoints_for_selected(snapshot);
        egui::ComboBox::from_label("Endpoint")
            .selected_text(self.endpoint_label(&endpoints_for_fabric))
            .show_ui(ui, |ui| {
                for endpoint in &endpoints_for_fabric {
                    let selected = Some(endpoint.id.clone()) == self.selected_endpoint;
                    if ui.selectable_label(selected, &endpoint.name).clicked() {
                        self.selected_endpoint = Some(endpoint.id.clone());
                    }
                }
            });

        egui::ComboBox::from_label("Target supernode")
            .selected_text(self.supernode_label(snapshot))
            .show_ui(ui, |ui| {
                for node in &snapshot.supernodes {
                    let selected = Some(node.id.clone()) == self.target_supernode;
                    if ui.selectable_label(selected, &node.name).clicked() {
                        self.target_supernode = Some(node.id.clone());
                    }
                }
            });

        let can_submit = !self.busy
            && self.selected_fabric.is_some()
            && self.selected_endpoint.is_some()
            && self.target_supernode.is_some();

        if ui
            .add_enabled(can_submit, egui::Button::new("Submit reassignment"))
            .clicked()
        {
            self.busy = true;
            self.status = Some("Submitting reassignment request...".into());
            command = Some(AppCommand::SubmitReassignment {
                fabric_id: self.selected_fabric.clone().unwrap(),
                endpoint_id: self.selected_endpoint.clone().unwrap(),
                target_supernode: self.target_supernode.clone().unwrap(),
            });
        }

        if let Some(status) = &self.status {
            ui.label(status);
        }
        command
    }

    fn fabric_label(&self, snapshot: &DashboardSnapshot) -> String {
        self.selected_fabric
            .as_ref()
            .and_then(|id| snapshot.fabrics.iter().find(|f| f.id == *id))
            .map(|fabric| fabric.name.clone())
            .unwrap_or_else(|| "Select a fabric".into())
    }

    fn endpoint_label(&self, endpoints: &[&FabrexEndpoint]) -> String {
        self.selected_endpoint
            .as_ref()
            .and_then(|id| endpoints.iter().find(|ep| ep.id == *id))
            .map(|ep| ep.name.clone())
            .unwrap_or_else(|| "Select an endpoint".into())
    }

    fn supernode_label(&self, snapshot: &DashboardSnapshot) -> String {
        self.target_supernode
            .as_ref()
            .and_then(|id| snapshot.supernodes.iter().find(|node| node.id == *id))
            .map(|node| node.name.clone())
            .unwrap_or_else(|| "Select a supernode".into())
    }

    fn endpoints_for_selected<'a>(
        &self,
        snapshot: &'a DashboardSnapshot,
    ) -> Vec<&'a FabrexEndpoint> {
        self.selected_fabric
            .as_ref()
            .map(|fabric_id| {
                snapshot
                    .endpoints
                    .iter()
                    .filter(|ep| ep.fabric_id.as_deref() == Some(fabric_id.as_str()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[derive(Clone, Copy)]
enum LogLevel {
    Info,
    Warn,
    Error,
}

struct LogEntry {
    timestamp: SystemTime,
    level: LogLevel,
    message: String,
}

impl LogEntry {
    fn new(level: LogLevel, message: String) -> Self {
        Self {
            timestamp: SystemTime::now(),
            level,
            message,
        }
    }

    fn age_display(&self) -> String {
        match SystemTime::now().duration_since(self.timestamp) {
            Ok(duration) => {
                if duration < Duration::from_secs(60) {
                    format!("{:.0}s", duration.as_secs_f32())
                } else if duration < Duration::from_secs(3600) {
                    format!("{:.1}m", duration.as_secs_f64() / 60.0)
                } else {
                    format!("{:.1}h", duration.as_secs_f64() / 3600.0)
                }
            }
            Err(_) => "now".into(),
        }
    }
}

fn log_colors(level: LogLevel) -> (egui::Color32, egui::Color32) {
    match level {
        LogLevel::Info => (
            egui::Color32::from_rgb(55, 125, 230),
            egui::Color32::from_rgb(55, 125, 230).linear_multiply(0.10),
        ),
        LogLevel::Warn => (
            egui::Color32::from_rgb(236, 146, 36),
            egui::Color32::from_rgb(236, 146, 36).linear_multiply(0.12),
        ),
        LogLevel::Error => (
            egui::Color32::from_rgb(225, 85, 73),
            egui::Color32::from_rgb(225, 85, 73).linear_multiply(0.12),
        ),
    }
}

enum AppCommand {
    RefreshDashboard,
    SubmitReassignment {
        fabric_id: String,
        endpoint_id: String,
        target_supernode: String,
    },
    StartPolling {
        interval_secs: u64,
    },
    StopPolling,
    UpdatePolling {
        interval_secs: u64,
    },
}

enum AppEvent {
    DashboardUpdated(DashboardSnapshot),
    DashboardFailed(String),
    ReassignmentCompleted(FabrexReassignmentResult),
    ReassignmentFailed(String),
}

fn spawn_background_worker(
    config: Arc<AppConfig>,
    credential_manager: Arc<CredentialManager>,
    command_rx: Receiver<AppCommand>,
    event_tx: Sender<AppEvent>,
) {
    thread::spawn(move || {
        let runtime = Runtime::new().expect("tokio runtime");
        let services = ServiceContext::new(config, credential_manager);
        let mut poller: Option<PollingHandle> = None;

        while let Ok(command) = command_rx.recv() {
            match command {
                AppCommand::RefreshDashboard => {
                    let result = runtime.block_on(fetch_dashboard_snapshot(&services));
                    match result {
                        Ok(snapshot) => {
                            let _ = event_tx.send(AppEvent::DashboardUpdated(snapshot));
                        }
                        Err(err) => {
                            let _ = event_tx.send(AppEvent::DashboardFailed(err.to_string()));
                        }
                    }
                }
                AppCommand::SubmitReassignment {
                    fabric_id,
                    endpoint_id,
                    target_supernode,
                } => {
                    let result = runtime.block_on(perform_reassignment(
                        &services,
                        fabric_id,
                        endpoint_id,
                        target_supernode,
                    ));
                    match result {
                        Ok(res) => {
                            let _ = event_tx.send(AppEvent::ReassignmentCompleted(res));
                        }
                        Err(err) => {
                            let _ = event_tx.send(AppEvent::ReassignmentFailed(err.to_string()));
                        }
                    }
                }
                AppCommand::StartPolling { interval_secs } => {
                    if let Some(handle) = poller.take() {
                        handle.stop();
                    }
                    poller = Some(start_polling(
                        &runtime,
                        services.clone(),
                        event_tx.clone(),
                        Duration::from_secs(interval_secs.max(5)),
                    ));
                }
                AppCommand::UpdatePolling { interval_secs } => {
                    if let Some(handle) = poller.take() {
                        handle.stop();
                    }
                    poller = Some(start_polling(
                        &runtime,
                        services.clone(),
                        event_tx.clone(),
                        Duration::from_secs(interval_secs.max(5)),
                    ));
                }
                AppCommand::StopPolling => {
                    if let Some(handle) = poller.take() {
                        handle.stop();
                    }
                }
            }
        }
    });
}

struct PollingHandle {
    stop: oneshot::Sender<()>,
}

impl PollingHandle {
    fn stop(self) {
        let _ = self.stop.send(());
    }
}

fn start_polling(
    runtime: &Runtime,
    services: ServiceContext,
    event_tx: Sender<AppEvent>,
    interval: Duration,
) -> PollingHandle {
    let (stop_tx, mut stop_rx) = oneshot::channel();
    let services = services.clone();
    let event_tx = event_tx.clone();
    let interval = interval.max(Duration::from_secs(5));

    runtime.spawn(async move {
        let mut ticker = time::interval(interval);

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    match fetch_dashboard_snapshot(&services).await {
                        Ok(snapshot) => { let _ = event_tx.send(AppEvent::DashboardUpdated(snapshot)); }
                        Err(err) => { let _ = event_tx.send(AppEvent::DashboardFailed(err.to_string())); }
                    }
                }
                _ = &mut stop_rx => break,
            }
        }
    });

    PollingHandle { stop: stop_tx }
}

#[derive(Clone)]
struct ServiceContext {
    config: Arc<AppConfig>,
    credentials: Arc<CredentialManager>,
}

impl ServiceContext {
    fn new(config: Arc<AppConfig>, credentials: Arc<CredentialManager>) -> Self {
        Self {
            config,
            credentials,
        }
    }

    fn auth_context(&self, domain: CredentialDomain) -> Result<AuthContext> {
        let key = CredentialKey::default(domain.clone());
        self.credentials
            .auth_context(&key)?
            .ok_or_else(|| anyhow!("Missing credentials for {domain}"))
    }

    fn fabrex_client(&self) -> Result<FabrexClient> {
        let auth = self.auth_context(CredentialDomain::FabreX)?;
        let config = ApiClientConfig::try_from_url(&self.config.fabrex_base_url)?;
        Ok(FabrexClient::new(config)?.with_auth(auth))
    }

    fn gryf_client(&self) -> Result<GryfClient> {
        let auth = self.auth_context(CredentialDomain::Gryf)?;
        let config = ApiClientConfig::try_from_url(&self.config.gryf_base_url)?;
        Ok(GryfClient::new(config)?.with_auth(auth))
    }

    fn supernode_client(&self) -> Result<SupernodeClient> {
        let auth = self.auth_context(CredentialDomain::Supernode)?;
        let config = ApiClientConfig::try_from_url(&self.config.supernode_base_url)?;
        Ok(SupernodeClient::new(config)?.with_auth(auth))
    }
}

async fn fetch_dashboard_snapshot(services: &ServiceContext) -> Result<DashboardSnapshot> {
    let fabrex_client = services.fabrex_client()?;
    let gryf_client = services.gryf_client()?;
    let supernode_client = services.supernode_client()?;

    let fabrex_for_join = fabrex_client.clone();
    let gryf_for_join = gryf_client.clone();
    let supernode_for_join = supernode_client.clone();

    let (fabrics, workloads, supernodes) = try_join!(
        fabrex_for_join.list_fabrics(),
        gryf_for_join.list_workloads(),
        supernode_for_join.list_nodes()
    )?;

    let mut usage: Vec<FabrexUsage> = Vec::new();
    let mut endpoints: Vec<FabrexEndpoint> = Vec::new();

    for fabric in &fabrics {
        let fabric_id = fabric.id.clone();
        let fabric_usage = fabrex_client
            .clone()
            .fabric_usage(&fabric_id)
            .await
            .with_context(|| format!("Fetching usage for fabric {fabric_id}"))?;
        usage.push(fabric_usage);

        let mut endpoint_page = fabrex_client
            .clone()
            .list_endpoints(&fabric_id, None)
            .await
            .with_context(|| format!("Fetching endpoints for fabric {fabric_id}"))?
            .items;
        for endpoint in &mut endpoint_page {
            if endpoint.fabric_id.is_none() {
                endpoint.fabric_id = Some(fabric_id.clone());
            }
        }
        endpoints.extend(endpoint_page);
    }

    let alerts = usage
        .iter()
        .flat_map(|entry| entry.alerts.iter())
        .map(|alert| format!("{}: {}", alert.severity.to_uppercase(), alert.message))
        .collect();

    Ok(DashboardSnapshot {
        fabrics,
        fabric_usage: usage,
        workloads,
        supernodes,
        endpoints,
        alerts,
    })
}

async fn perform_reassignment(
    services: &ServiceContext,
    fabric_id: String,
    endpoint_id: String,
    target_supernode: String,
) -> Result<FabrexReassignmentResult> {
    let client = services.fabrex_client()?;
    let result = client
        .reassign_endpoint(&fabric_id, &endpoint_id, &target_supernode)
        .await?;
    Ok(result)
}

#[derive(Debug, Clone)]
struct ProvisionForm {
    domain: CredentialDomain,
    username: String,
    password: String,
    api_token: String,
    show_password: bool,
    show_token: bool,
    error: Option<String>,
}

impl ProvisionForm {
    fn new(domain: CredentialDomain) -> Self {
        Self {
            domain,
            username: String::new(),
            password: String::new(),
            api_token: String::new(),
            show_password: false,
            show_token: false,
            error: None,
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) -> ProvisionUiEvent {
        ui.set_min_width(340.0);
        ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 8.0);

        ui.label(
            egui::RichText::new(format!("Provide credentials for {}", self.domain))
                .text_style(egui::TextStyle::Name("Title".into())),
        );
        ui.label(
            egui::RichText::new("Secrets are stored locally in the system keychain.")
                .text_style(egui::TextStyle::Small)
                .color(egui::Color32::from_rgb(120, 130, 150)),
        );

        ui.separator();
        ui.label(egui::RichText::new("Username").strong());
        ui.add(
            egui::TextEdit::singleline(&mut self.username)
                .hint_text("service account username")
                .min_size(egui::vec2(ui.available_width(), 0.0)),
        );

        ui.label(egui::RichText::new("Password").strong());
        ui.horizontal(|ui| {
            let password_edit = egui::TextEdit::singleline(&mut self.password)
                .hint_text("password")
                .password(!self.show_password)
                .min_size(egui::vec2(ui.available_width() - 80.0, 0.0));
            ui.add(password_edit);
            ui.toggle_value(&mut self.show_password, "Show");
        });

        ui.label(
            egui::RichText::new("API token (optional)")
                .text_style(egui::TextStyle::Small)
                .color(egui::Color32::from_rgb(120, 130, 150)),
        );
        ui.horizontal(|ui| {
            let token_edit = egui::TextEdit::singleline(&mut self.api_token)
                .hint_text("token or leave blank")
                .password(!self.show_token)
                .min_size(egui::vec2(ui.available_width() - 80.0, 0.0));
            ui.add(token_edit);
            ui.toggle_value(&mut self.show_token, "Show");
        });

        if let Some(error) = &self.error {
            ui.colored_label(egui::Color32::from_rgb(225, 85, 73), error);
        }

        ui.add_space(10.0);
        let mut event = ProvisionUiEvent::None;
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let has_username = !self.username.trim().is_empty();
            let has_secret = !self.password.trim().is_empty() || !self.api_token.trim().is_empty();
            let can_submit = has_username && has_secret;
            let save_clicked = ui
                .add_enabled(
                    can_submit,
                    egui::Button::new("Save credentials")
                        .fill(ui.visuals().selection.bg_fill)
                        .min_size(egui::vec2(160.0, 0.0)),
                )
                .clicked();

            ui.add_space(8.0);
            if ui.button("Cancel").clicked() {
                event = ProvisionUiEvent::Cancel;
            }

            if save_clicked {
                if can_submit {
                    self.error = None;
                    let secret = CredentialSecret {
                        username: self.username.trim().to_owned(),
                        password: self.password.clone(),
                        api_token: if self.api_token.trim().is_empty() {
                            None
                        } else {
                            Some(self.api_token.trim().to_owned())
                        },
                    };
                    event = ProvisionUiEvent::Submit(secret);
                } else {
                    self.error =
                        Some("Username and either a password or API token are required.".into());
                }
            }
        });

        event
    }
}

enum ProvisionUiEvent {
    None,
    Submit(CredentialSecret),
    Cancel,
}

enum ProvisionOutcome {
    None,
    Submit(CredentialDomain, CredentialSecret),
    Cancel,
}
