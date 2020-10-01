use super::{elem_type::*, node_base::*, xml, Parse};

#[derive(Debug, Clone)]
pub struct RegisterBase {
    pub(super) elem_base: NodeElementBase,
    pub(super) streamable: bool,
    pub(super) address_kinds: Vec<register_node_elem::AddressKind>,
    pub(super) length: ImmOrPNode<i64>,
    pub(super) access_mode: AccessMode,
    pub(super) p_port: String,
    pub(super) cacheable: CachingMode,
    pub(super) polling_time: Option<u64>,
    pub(super) p_invalidators: Vec<String>,
}

impl RegisterBase {
    pub fn streamable(&self) -> bool {
        self.streamable
    }

    pub fn address_kinds(&self) -> &[register_node_elem::AddressKind] {
        &self.address_kinds
    }

    pub fn length(&self) -> &ImmOrPNode<i64> {
        &self.length
    }

    pub fn access_mode(&self) -> AccessMode {
        self.access_mode
    }

    pub fn p_port(&self) -> &str {
        &self.p_port
    }

    pub fn cacheable(&self) -> CachingMode {
        self.cacheable
    }

    pub fn polling_time(&self) -> Option<u64> {
        self.polling_time
    }

    pub fn p_invalidators(&self) -> &[String] {
        &self.p_invalidators
    }
}

impl Parse for RegisterBase {
    fn parse(node: &mut xml::Node) -> Self {
        let elem_base = node.parse();

        let streamable = node.parse_if("Streamable").unwrap_or_default();

        let mut address_kinds = vec![];
        while let Some(addr_kind) = node
            .parse_if("Address")
            .or_else(|| node.parse_if("IntSwissKnife"))
            .or_else(|| node.parse_if("pAddress"))
            .or_else(|| node.parse_if("pIndex"))
        {
            address_kinds.push(addr_kind);
        }

        let length = node.parse();

        let access_mode = node.parse_if("AccessMode").unwrap_or(AccessMode::RO);

        let p_port = node.parse();

        let cacheable = node.parse_if("Cachable").unwrap_or_default();

        let polling_time = node.parse_if("PollingTime");

        let mut p_invalidators = vec![];
        while let Some(invalidator) = node.parse_if("pInvalidator") {
            p_invalidators.push(invalidator);
        }

        Self {
            elem_base,
            streamable,
            address_kinds,
            length,
            access_mode,
            p_port,
            cacheable,
            polling_time,
            p_invalidators,
        }
    }
}
