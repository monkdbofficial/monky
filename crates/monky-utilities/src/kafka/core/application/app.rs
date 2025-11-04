/*
 * Copyright (C) 2025 Movibase Platform Private Limited
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::collections::HashMap;
use once_cell::sync::Lazy;

use crate::kafka::core::schema::{topic::Topic, topic_impl::ApplicationCommunication};

// Static default config shared by all topics
static DEFAULT_COMPACT_CONFIG: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert("cleanup.policy".to_string(), "compact".to_string());
    map.insert("segment.bytes".to_string(), "10485760".to_string());
    map.insert("min.compaction.lag.ms".to_string(), "86400000".to_string());
    map
});

/// Generic struct for all application communication topics
pub struct AppTopic {
    base: ApplicationCommunication,
    dataset_name: &'static str,
    use_default_config: bool,
}

impl AppTopic {
    pub fn new(dataset_name: &'static str) -> Self {
        Self {
            base: ApplicationCommunication,
            dataset_name,
            use_default_config: true,
        }
    }

    /// If a topic needs a custom config, we can override this flag
    pub fn with_custom_config(dataset_name: &'static str) -> Self {
        Self {
            base: ApplicationCommunication,
            dataset_name,
            use_default_config: false,
        }
    }
}

impl Topic for AppTopic {
    fn kind(&self) -> &str {
        self.base.kind()
    }

    fn domain(&self) -> &str {
        self.base.domain()
    }

    fn dataset(&self) -> &str {
        self.dataset_name
    }

    fn config(&self) -> HashMap<String, String> {
        if self.use_default_config {
            DEFAULT_COMPACT_CONFIG.clone()
        } else {
            HashMap::new()
        }
    }
}


pub fn app_channels() -> AppTopic {
    AppTopic::new("channels")
}

pub fn app_contacts() -> AppTopic {
    AppTopic::new("contacts")
}

pub fn app_messages() -> AppTopic {
    AppTopic::new("messages")
}

pub fn app_metadata() -> AppTopic {
    AppTopic::new("metadata")
}

pub fn app_read_receipts() -> AppTopic {
    AppTopic::new("read-receipt")
}

pub fn app_sources() -> AppTopic {
    AppTopic::new("sources")
}

pub fn app_tags() -> AppTopic {
    AppTopic::new("tags")
}

pub fn app_templates() -> AppTopic {
    AppTopic::new("templates")
}

pub fn app_users() -> AppTopic {
    AppTopic::new("users")
}

pub fn app_webhooks() -> AppTopic {
    AppTopic::new("webhooks")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_names() {
        let topic = app_messages();
        assert_eq!(topic.name(), "application.communication.messages");
        assert_eq!(topic.config().get("cleanup.policy").unwrap(), "compact");
    }
}
