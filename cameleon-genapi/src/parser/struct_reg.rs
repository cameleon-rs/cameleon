use super::{
    elem_name::{
        ACCESS_MODE, CACHEABLE, COMMENT, ENDIANNESS, POLLING_TIME, P_INVALIDATOR, P_SELECTED,
        REPRESENTATION, SIGN, STREAMABLE, STRUCT_ENTRY, STRUCT_REG, UNIT,
    },
    elem_type::{register_node_elem, AccessMode, CachingMode, IntegerRepresentation},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    register_base::RegisterBase,
    xml, MaskedIntRegNode, Parse,
};

#[derive(Debug, Clone)]
pub struct StructRegNode {
    comment: String,
    register_base: RegisterBase,

    endianness: register_node_elem::Endianness,
    entries: Vec<StructEntryNode>,
}

impl StructRegNode {
    #[must_use]
    pub fn comment(&self) -> &str {
        &self.comment
    }

    #[must_use]
    pub fn register_base(&self) -> &RegisterBase {
        &self.register_base
    }

    #[must_use]
    pub fn endianness(&self) -> register_node_elem::Endianness {
        self.endianness
    }

    #[must_use]
    pub fn entries(&self) -> &[StructEntryNode] {
        &self.entries
    }

    #[must_use]
    pub fn to_masked_int_regs<T>(&self) -> T
    where
        T: std::iter::FromIterator<MaskedIntRegNode>,
    {
        self.entries
            .iter()
            .map(|ent| ent.to_masked_int_reg(&self))
            .collect()
    }
}

impl Parse for StructRegNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert_eq!(node.tag_name(), STRUCT_REG);

        let comment = node.attribute_of(COMMENT).unwrap().into();
        let register_base = node.parse();

        let endianness = node.parse_if(ENDIANNESS).unwrap_or_default();
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
pub struct StructEntryNode {
    attr_base: NodeAttributeBase,
    elem_base: NodeElementBase,

    p_invalidators: Vec<String>,
    access_mode: AccessMode,
    cacheable: CachingMode,
    polling_time: Option<u64>,
    streamable: bool,
    bit_mask: register_node_elem::BitMask,
    sign: register_node_elem::Sign,
    unit: Option<String>,
    representation: IntegerRepresentation,
    p_selected: Vec<String>,
}

/// See "2.8.7 StructReg" in GenICam Standard v2.1.1.
macro_rules! merge_impl {
    ($lhs:ident, $rhs:ident, $name:ident) => {
        if $rhs.$name.is_some() {
            $lhs.$name = $rhs.$name;
        }
    };

    ($lhs:ident, $rhs:ident, $name:ident, default) => {
        #[allow(clippy::default_trait_access)]
        if $rhs.$name != Default::default() {
            $lhs.$name = $rhs.$name;
        }
    };

    ($lhs:ident, $rhs:ident, $name:ident, vec) => {
        if $rhs.$name.is_empty() {
            $lhs.$name = $rhs.$name.clone();
        }
    };
}

impl StructEntryNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn p_invalidators(&self) -> &[String] {
        &self.p_invalidators
    }

    #[must_use]
    pub fn access_mode(&self) -> AccessMode {
        self.access_mode
    }

    #[must_use]
    pub fn cacheable(&self) -> CachingMode {
        self.cacheable
    }

    #[must_use]
    pub fn polling_time(&self) -> Option<u64> {
        self.polling_time
    }

    #[must_use]
    pub fn streamable(&self) -> bool {
        self.streamable
    }

    #[must_use]
    pub fn bit_mask(&self) -> register_node_elem::BitMask {
        self.bit_mask
    }

    #[must_use]
    pub fn sign(&self) -> register_node_elem::Sign {
        self.sign
    }

    #[must_use]
    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    #[must_use]
    pub fn representation(&self) -> IntegerRepresentation {
        self.representation
    }

    #[must_use]
    pub fn p_selected(&self) -> &[String] {
        &self.p_selected
    }

    fn to_masked_int_reg(&self, struct_reg: &StructRegNode) -> MaskedIntRegNode {
        self.clone().into_masked_int_reg(struct_reg)
    }

    fn into_masked_int_reg(self, struct_reg: &StructRegNode) -> MaskedIntRegNode {
        let attr_base = self.attr_base;

        let mut register_base = struct_reg.register_base().clone();
        let elem_base = &mut register_base.elem_base;
        elem_base.merge(self.elem_base);

        merge_impl!(register_base, self, streamable, default);
        // `AccessMode::RO` is the default value of AccessMode.
        if self.access_mode != AccessMode::RO {
            register_base.access_mode = self.access_mode;
        }
        merge_impl!(register_base, self, cacheable, default);
        merge_impl!(register_base, self, polling_time);
        merge_impl!(register_base, self, p_invalidators, vec);

        MaskedIntRegNode {
            attr_base,
            register_base,
            bit_mask: self.bit_mask,
            sign: self.sign,
            endianness: struct_reg.endianness,
            unit: self.unit,
            representation: self.representation,
            p_selected: self.p_selected,
        }
    }
}

impl NodeElementBase {
    fn merge(&mut self, rhs: Self) {
        merge_impl!(self, rhs, tool_tip);
        merge_impl!(self, rhs, description);
        merge_impl!(self, rhs, display_name);
        merge_impl!(self, rhs, visibility, default);
        merge_impl!(self, rhs, docu_url);
        merge_impl!(self, rhs, is_deprecated, default);
        merge_impl!(self, rhs, event_id);
        merge_impl!(self, rhs, p_is_implemented);
        merge_impl!(self, rhs, p_is_available);
        merge_impl!(self, rhs, p_is_locked);
        merge_impl!(self, rhs, p_block_polling);
        // `AccessMode::RW` is the default value of ImposedAccessMode.
        if rhs.imposed_access_mode != AccessMode::RW {
            self.imposed_access_mode = rhs.imposed_access_mode;
        }

        merge_impl!(self, rhs, p_errors, vec);
        merge_impl!(self, rhs, p_alias);
        merge_impl!(self, rhs, p_cast_alias);
    }
}

impl Parse for StructEntryNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert_eq!(node.tag_name(), STRUCT_ENTRY);

        let attr_base = node.parse();
        let elem_base = node.parse();

        let p_invalidators = node.parse_while(P_INVALIDATOR);
        let access_mode = node.parse_if(ACCESS_MODE).unwrap_or(AccessMode::RO);
        let cacheable = node.parse_if(CACHEABLE).unwrap_or_default();
        let polling_time = node.parse_if(POLLING_TIME);
        let streamable = node.parse_if(STREAMABLE).unwrap_or_default();
        let bit_mask = node.parse();
        let sign = node.parse_if(SIGN).unwrap_or_default();
        let unit = node.parse_if(UNIT);
        let representation = node.parse_if(REPRESENTATION).unwrap_or_default();
        let p_selected = node.parse_while(P_SELECTED);

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
    use super::super::elem_type::register_node_elem::{BitMask, Endianness, Sign};

    use super::*;

    #[test]
    fn test_struct_reg() {
        let xml = r#"
            <StructReg Comment="Struct Reg Comment">
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

        assert_eq!(node.comment(), "Struct Reg Comment");
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

    #[test]
    fn test_to_masked_int_regs() {
        let xml = r#"
            <StructReg Comment="Struct Reg Comment">
                <ToolTip>Struct Reg ToolTip</ToolTip>
                <Address>0x10000</Address>
                <Length>4</Length>
                <pPort>Device</pPort>
                <Endianess>BigEndian</Endianess>

                <StructEntry Name="StructEntry0">
                    <ToolTip>StructEntry0 ToolTip</ToolTip>
                    <ImposedAccessMode>RO</ImposedAccessMode>
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
        let masked_int_regs: Vec<_> = node.to_masked_int_regs();

        assert_eq!(masked_int_regs.len(), 2);

        let masked_int_reg0 = &masked_int_regs[0];
        assert_eq!(masked_int_reg0.node_base().name(), "StructEntry0");
        assert_eq!(
            masked_int_reg0.node_base().imposed_access_mode(),
            AccessMode::RO
        );
        assert_eq!(
            masked_int_reg0.node_base().tool_tip().unwrap(),
            "StructEntry0 ToolTip"
        );
        assert_eq!(
            masked_int_reg0.register_base().access_mode(),
            AccessMode::RW,
        );

        let masked_int_reg1 = &masked_int_regs[1];
        assert_eq!(masked_int_reg1.node_base().name(), "StructEntry1");
        assert_eq!(
            masked_int_reg1.node_base().imposed_access_mode(),
            AccessMode::RW
        );
        assert_eq!(
            masked_int_reg1.node_base().tool_tip().unwrap(),
            "Struct Reg ToolTip"
        );
        assert_eq!(
            masked_int_reg1.register_base().access_mode(),
            AccessMode::RO,
        );
    }
}