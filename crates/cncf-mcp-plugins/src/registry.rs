use crate::manifest::PluginManifest;
use std::collections::HashMap;

/// In-memory registry of loaded plugins.
#[derive(Debug, Default)]
pub struct PluginRegistry {
    plugins: HashMap<String, PluginManifest>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, manifest: PluginManifest) {
        self.plugins.insert(manifest.name.clone(), manifest);
    }

    pub fn get(&self, name: &str) -> Option<&PluginManifest> {
        self.plugins.get(name)
    }

    pub fn list(&self) -> Vec<&PluginManifest> {
        self.plugins.values().collect()
    }

    pub fn count(&self) -> usize {
        self.plugins.len()
    }
}
