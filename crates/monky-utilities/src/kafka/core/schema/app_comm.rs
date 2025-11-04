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

use crate::kafka::core::schema::topic::Topic;

/// A Kafka topic representing the **Application Communication** domain.
///
/// This struct acts as a marker type implementing the [`Topic`] trait,
/// providing static information about the topic's kind and domain.
///
/// # Overview
///
/// The `ApplicationCommunication` topic is used to handle messages related to
/// communication events between applications, such as:
/// - Notification dispatches
/// - Application-to-application messages
/// - Event-driven communication flows
/// 
/// # See also
///
/// - [`Topic`] — the trait defining the structure for Kafka topic identifiers.
///
/// # Notes
///
/// This struct contains no data and serves only as a type-level identifier.
pub struct ApplicationCommunication;

impl Topic for ApplicationCommunication {

    /// Returns the high-level category of this topic.
    /// For `ApplicationCommunication`, this always returns `"application"`.
    fn kind(&self) -> &str {
        "application"
    }

    /// Returns the specific domain within the kind of this topic.
    /// For `ApplicationCommunication`, this always returns `"communication"`.
    fn domain(&self) -> &str {
        "communication"
    }
}