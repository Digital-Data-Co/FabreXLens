mod dashboard;
mod theme;

pub use dashboard::{render as render_dashboard, DashboardSnapshot, DashboardState};
pub use theme::apply_theme;
