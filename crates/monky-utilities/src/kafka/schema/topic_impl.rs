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

use crate::kafka::schema::topic::Topic;

pub struct OpsApplication;

impl Topic for OpsApplication {
    fn kind(&self) -> &str {
        "ops"
    }

    fn domain(&self) -> &str {
        "application"
    }
}

pub struct ApplicationCommunication;

impl Topic for ApplicationCommunication {
    fn kind(&self) -> &str {
        "application"
    }

    fn domain(&self) -> &str {
        "communication"
    }
}

pub struct SourceTwilio;

impl Topic for SourceTwilio {
    fn kind(&self) -> &str {
        "source"
    }

    fn domain(&self) -> &str {
        "twilio"
    }
}
