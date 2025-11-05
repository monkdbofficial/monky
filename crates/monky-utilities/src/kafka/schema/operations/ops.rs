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

use crate::kafka::schema::{topic::Topic, topic_impl::OpsApplication};

pub struct OpsTopic {
    base: OpsApplication,
    dataset_name: &'static str,
}

impl OpsTopic {
    pub fn new(dataset_name: &'static str) -> Self {
        Self {
            base: OpsApplication,
            dataset_name,
        }
    }
}

impl Topic for OpsTopic {
    fn kind(&self) -> &str {
        self.base.kind()
    }
    fn domain(&self) -> &str {
        self.base.domain()
    }
    fn dataset(&self) -> &str {
        self.dataset_name
    }
}

pub fn ops_components() -> OpsTopic {
    OpsTopic::new("components")
}

pub fn ops_logs() -> OpsTopic {
    OpsTopic::new("logs")
}
