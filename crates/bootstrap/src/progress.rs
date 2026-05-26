use indicatif::{ProgressBar, ProgressStyle};

pub struct ProgressReporter {
    current_phase: Option<u8>,
    total_phases: u8,
}

impl ProgressReporter {
    pub fn new(total_phases: u8) -> Self {
        Self {
            current_phase: None,
            total_phases,
        }
    }

    pub fn start_phase(&mut self, phase_id: u8, name: &str) {
        self.current_phase = Some(phase_id);
        println!("[{}/{}] {}...", phase_id, self.total_phases, name);
    }

    pub fn phase_detail(&self, message: &str) {
        println!("      +-- {}", message);
    }

    pub fn phase_done(&self, duration_ms: u64) {
        let duration = if duration_ms > 60_000 {
            format!("{}m {:02}s", duration_ms / 60_000, (duration_ms % 60_000) / 1000)
        } else {
            format!("{:.1}s", duration_ms as f64 / 1000.0)
        };
        println!("      \\-- done ({})", duration);
    }

    pub fn phase_skipped(&self, reason: &str) {
        println!("      \\-- SKIPPED: {}", reason);
    }

    pub fn phase_warning(&self, message: &str) {
        println!("      !-- WARNING: {}", message);
    }

    pub fn create_progress_bar(&self, total: u64, template: &str) -> ProgressBar {
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(&format!("      +-- {}", template))
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("=> "),
        );
        pb
    }

    pub fn summary(&self, nodes: usize, edges: usize) {
        println!();
        println!("Bootstrap complete. L4 graph: {} nodes, {} edges.", nodes, edges);
    }
}
