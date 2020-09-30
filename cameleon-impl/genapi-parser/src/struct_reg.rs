use super::{elem_type::*, node_base::*, register_base::*, xml, Parse};

#[derive(Debug, Clone)]
pub struct StructRegNode {
    comment: String,

    register_base: RegisterBase,

    endianness: register_node_elem::Endianness,

    entries: Vec<StructEntry>,
}

impl StructRegNode {
    pub fn comment(&self) -> &str {
        &self.comment
    }

    pub fn register_base(&self) -> &RegisterBase {
        &self.register_base
    }

    pub fn endianness(&self) -> register_node_elem::Endianness {
        self.endianness
    }

    pub fn entries(&self) -> &[StructEntry] {
        &self.entries
    }
}

impl Parse for StructRegNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert_eq!(node.tag_name(), "StructReg");

        let comment = node.attribute_of("Comment").unwrap().into();

        let register_base = node.parse();

        let endianness = node.parse_if("Endianess").unwrap_or_default();

        let mut entries = vec![];
        while let Some(mut entry_node) = node.next() {
            let entry = entry_node.parse();
            entries.push(entry);
        }

        Self {
            comment,
            register_base,
            endianness,
            entries,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StructEntry {
    attr_base: NodeAttributeBase,
    elem_base: NodeElementBase,

    p_invalidators: Vec<String>,

    access_mode: AccessMode,

    cacheable: CachingMode,

    polling_time: Option<i64>,

    streamable: bool,

    bit_mask: register_node_elem::BitMask,

    sign: register_node_elem::Sign,

    unit: Option<String>,

    representation: IntegerRepresentation,

    p_selected: Vec<String>,
}

impl StructEntry {
    pub fn node_base(&self) -> NodeBase {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    pub fn p_invalidators(&self) -> &[String] {
        &self.p_invalidators
    }

    pub fn access_mode(&self) -> AccessMode {
        self.access_mode
    }

    pub fn cacheable(&self) -> CachingMode {
        self.cacheable
    }

    pub fn polling_time(&self) -> Option<i64> {
        self.polling_time
    }

    pub fn streamable(&self) -> bool {
        self.streamable
    }

    pub fn bit_mask(&self) -> register_node_elem::BitMask {
        self.bit_mask
    }

    pub fn sign(&self) -> register_node_elem::Sign {
        self.sign
    }

    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    pub fn representation(&self) -> IntegerRepresentation {
        self.representation
    }

    pub fn p_selected(&self) -> &[String] {
        &self.p_selected
    }
}

impl Parse for StructEntry {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert_eq!(node.tag_name(), "StructEntry");
        let attr_base = node.parse();
        let elem_base = node.parse();

        let mut p_invalidators = vec![];
        while let Some(invalidator) = node.parse_if("pInvalidator") {
            p_invalidators.push(invalidator);
        }

        let access_mode = node.parse_if("AccessMode").unwrap_or(AccessMode::RO);

        let cacheable = node.parse_if("Cachable").unwrap_or_default();

        let polling_time = node.parse_if("PollingTime");

        let streamable = node.parse_if("Streamable").unwrap_or_default();

        let bit_mask = node.parse();

        let sign = node.parse_if("Sign").unwrap_or_default();

        let unit = node.parse_if("Unit");

        let representation = node.parse_if("Representation").unwrap_or_default();

        let mut p_selected = vec![];
        while let Some(selected) = node.parse_if("pSelected") {
            p_selected.push(selected);
        }

        Self {
            attr_base,
            elem_base,
            p_invalidators,
            access_mode,
            cacheable,
            polling_time,
            streamable,
            bit_mask,
            sign,
            unit,
            representation,
            p_selected,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::elem_type::register_node_elem::*;

    use super::*;

    #[test]
    fn test_struct_reg() {
        let xml = r#"
            <StructReg Comment="Struct Entry Comment">
                <Address>0x10000</Address>
                <Length>4</Length>
                <pPort>Device</pPort>
                <Endianess>BigEndian</Endianess>

                <StructEntry Name="StructEntry0">
                    <pInvalidator>Invalidator0</pInvalidator>
                    <pInvalidator>Invalidator1</pInvalidator>
                    <AccessMode>RW</AccessMode>
                    <Cachable>WriteAround</Cachable>
                    <PollingTime>1000</PollingTime>
                    <Streamable>Yes</Streamable>
                    <LSB>10</LSB>
                    <MSB>1</MSB>
                    <Sign>Signed</Sign>
                    <Unit>Hz</Unit>
                    <Representation>Logarithmic</Representation>
                    <pSelected>Selected0</pSelected>
                    <pSelected>Selected1</pSelected>
                </StructEntry>

                <StructEntry Name="StructEntry1">
                    <Bit>24</Bit>
                </StructEntry>

            </StructReg>
            "#;

        let node: StructRegNode = xml::Document::from_str(&xml).unwrap().root_node().parse();

        assert_eq!(node.comment(), "Struct Entry Comment");
        assert_eq!(node.endianness(), Endianness::BE);

        let entries = node.entries();
        assert_eq!(entries.len(), 2);

        let first_ent = &entries[0];
        assert_eq!(first_ent.node_base().name(), "StructEntry0");
        assert_eq!(first_ent.p_invalidators().len(), 2);
        assert_eq!(first_ent.access_mode(), AccessMode::RW);
        assert_eq!(first_ent.cacheable(), CachingMode::WriteAround);
        assert_eq!(first_ent.polling_time().unwrap(), 1000);
        assert_eq!(first_ent.streamable(), true);
        assert_eq!(first_ent.bit_mask(), BitMask::Range { lsb: 10, msb: 1 });
        assert_eq!(first_ent.sign(), Sign::Signed);
        assert_eq!(first_ent.unit().unwrap(), "Hz");
        assert_eq!(
            first_ent.representation(),
            IntegerRepresentation::Logarithmic
        );
        assert_eq!(first_ent.p_selected().len(), 2);

        let second_ent = &entries[1];
        assert_eq!(second_ent.node_base().name(), "StructEntry1");
        assert_eq!(second_ent.bit_mask(), BitMask::SingleBit(24));
    }
}
