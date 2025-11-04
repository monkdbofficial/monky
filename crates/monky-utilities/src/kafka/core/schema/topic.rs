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
use std::env;

use once_cell::sync::Lazy;

static MONKY_NAMESPACE: Lazy<String> = Lazy::new(|| {
    match env::var("MONKY_CORE_NAMESPACE") {
        Ok(ns) if !ns.is_empty() => format!("{}.", ns),
        _ => String::new(),
    }
});

pub struct AbstractTopic;

impl AbstractTopic {
    fn namespace() -> &'static str {
        &MONKY_NAMESPACE
    }
}

pub trait Topic {
    fn kind(&self) -> &str;
    fn domain(&self) -> &str;
    fn dataset(&self) -> &str;

    fn config(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    fn name(&self) -> String {
        format!(
            "{}{}.{}.{}",
            AbstractTopic::namespace(),
            self.kind(),
            self.domain(),
            self.dataset()
        )
    }
}

