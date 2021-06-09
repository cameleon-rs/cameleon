/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::elem_type::StandardNameSpace;

#[derive(Clone, Debug)]
pub struct RegisterDescription {
    pub(crate) model_name: String,
    pub(crate) vendor_name: String,
    pub(crate) tooltip: Option<String>,
    pub(crate) standard_name_space: StandardNameSpace,
    pub(crate) schema_major_version: u64,
    pub(crate) schema_minor_version: u64,
    pub(crate) schema_subminor_version: u64,
    pub(crate) major_version: u64,
    pub(crate) minor_version: u64,
    pub(crate) subminor_version: u64,
    pub(crate) product_guid: String,
    pub(crate) version_guid: String,
}

impl RegisterDescription {
    #[must_use]
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    #[must_use]
    pub fn vendor_name(&self) -> &str {
        &self.vendor_name
    }

    #[must_use]
    pub fn tooltip(&self) -> Option<&str> {
        self.tooltip.as_deref()
    }

    #[must_use]
    pub fn standard_name_space(&self) -> StandardNameSpace {
        self.standard_name_space
    }

    #[must_use]
    pub fn schema_major_version(&self) -> u64 {
        self.schema_major_version
    }

    #[must_use]
    pub fn schema_subminor_version(&self) -> u64 {
        self.schema_subminor_version
    }

    #[must_use]
    pub fn schema_minor_version(&self) -> u64 {
        self.schema_minor_version
    }

    #[must_use]
    pub fn major_version(&self) -> u64 {
        self.major_version
    }

    #[must_use]
    pub fn minor_version(&self) -> u64 {
        self.minor_version
    }

    #[must_use]
    pub fn subminor_version(&self) -> u64 {
        self.subminor_version
    }

    #[must_use]
    pub fn product_guid(&self) -> &str {
        &self.product_guid
    }

    #[must_use]
    pub fn version_guid(&self) -> &str {
        &self.version_guid
    }
}
