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

/// Key name used to enable specific Avro reader mode
pub const SPECIFIC_AVRO_READER_CONFIG: &str = "specific.avro.reader";

/// Merge or create a Kafka configuration map with Specific Avro reader enabled.
///
/// If `serializer_config` is `Some(map)`, it clones and augments it.
/// If `None`, it creates a new config map.
pub fn with_specific_avro_enabled(
    serializer_config: Option<&HashMap<String, String>>,
) -> HashMap<String, String> {
    // clone existing config if provided
    let mut config = match serializer_config {
        Some(map) => map.clone(),
        None => HashMap::new(),
    };

    // enable specific Avro reader flag
    config.insert(SPECIFIC_AVRO_READER_CONFIG.to_string(), "true".to_string());

    config
}