//! Audit plugin: counts every PreRequest hook it sees.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Counts requests passing through the hook pipeline.
pub struct AuditPlugin {
    info: toxi_plugin::PluginInfo,
    count: Arc<AtomicU64>,
}

impl AuditPlugin {
    /// Create the plugin with a shared counter handle.
    pub fn new(count: Arc<AtomicU64>) -> Self {
        Self {
            info: toxi_plugin::PluginInfo::new(
                "audit",
                "Audit",
                "0.1.0",
                "Counts PreRequest hooks",
                "taskboard",
            ),
            count,
        }
    }
}

#[async_trait::async_trait]
impl toxi_plugin::Plugin for AuditPlugin {
    fn info(&self) -> toxi_plugin::PluginInfo {
        let mut info = self.info.clone();
        info.enabled = true;
        info
    }

    async fn hook(&self, hook: toxi_plugin::PluginHook) -> toxi_plugin::HookResult {
        if matches!(hook, toxi_plugin::PluginHook::PreRequest { .. }) {
            self.count.fetch_add(1, Ordering::Relaxed);
        }
        toxi_plugin::HookResult::Continue
    }
}
