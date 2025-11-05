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

use crate::kafka::schema::topic::Topic;

pub struct SourceTopic {
    kind: &'static str,
    domain: &'static str,
    dataset: &'static str,
}

impl SourceTopic {
    pub fn new(domain: &'static str, dataset: &'static str) -> Self {
        Self {
            kind: "source",
            domain,
            dataset,
        }
    }
}

impl Topic for SourceTopic {
    fn kind(&self) -> &str {
        self.kind
    }

    fn domain(&self) -> &str {
        self.domain
    }

    fn dataset(&self) -> &str {
        self.dataset
    }

    fn config(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

pub fn source_facebook_events() -> SourceTopic {
    SourceTopic::new("facebook", "events")
}

pub fn source_whatsapp_events() -> SourceTopic {
    SourceTopic::new("whatsapp", "events")
}

pub fn source_viber_events() -> SourceTopic {
    SourceTopic::new("viber", "events")
}

pub fn source_twilio_events() -> SourceTopic {
    SourceTopic::new("twilio", "events")
}

pub fn source_google_events() -> SourceTopic {
    SourceTopic::new("google", "events")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_topic_names() {
        let fb = source_facebook_events();
        assert_eq!(fb.name(), "source.facebook.events");

        let twilio = source_twilio_events();
        assert_eq!(twilio.name(), "source.twilio.events");
    }
}
