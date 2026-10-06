//! Maintenance plugin: short-circuits requests while enabled.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Returns Stop on PreRequest while the flag is set.
pub struct MaintenancePlugin {
    info: toxi_plugin::PluginInfo,
    active: Arc<AtomicBool>,
}

impl MaintenancePlugin {
    /// Create the plugin with a shared flag handle.
    pub fn new(active: Arc<AtomicBool>) -> Self {
        Self {
            info: toxi_plugin::PluginInfo::new(
                "maintenance",
                "Maintenance",
                "0.1.0",
                "Short-circuits requests while active",
                "taskboard",
            ),
            active,
        }
    }
}

#[async_trait::async_trait]
impl toxi_plugin::Plugin for MaintenancePlugin {
    fn info(&self) -> toxi_plugin::PluginInfo {
        let mut info = self.info.clone();
        info.enabled = true;
        info
    }

    async fn hook(&self, hook: toxi_plugin::PluginHook) -> toxi_plugin::HookResult {
        if matches!(hook, toxi_plugin::PluginHook::PreRequest { .. }) && self.active.load(Ordering::Relaxed) {
            return toxi_plugin::HookResult::Stop;
        }
        toxi_plugin::HookResult::Continue
    }
}
