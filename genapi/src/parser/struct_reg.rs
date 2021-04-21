use crate::{
    elem_type::{AccessMode, BitMask, CachingMode, Endianness, IntegerRepresentation, Sign},
    node_base::{NodeAttributeBase, NodeElementBase},
    register_base::RegisterBase,
    store::{NodeId, NodeStore},
    MaskedIntRegNode,
};

use super::{
    elem_name::{
        ACCESS_MODE, CACHEABLE, COMMENT, ENDIANNESS, POLLING_TIME, P_INVALIDATOR, P_SELECTED,
        REPRESENTATION, SIGN, STREAMABLE, STRUCT_ENTRY, STRUCT_REG, UNIT,
    },
    xml, Parse,
};

#[derive(Debug, Clone)]
pub(super) struct StructRegNode {
    comment: String,
    register_base: RegisterBase,

    endianness: Endianness,
    entries: Vec<StructEntryNode>,
}

impl StructRegNode {
    #[must_use]
    pub(super) fn into_masked_int_regs(self) -> Vec<MaskedIntRegNode> {
        let register_base = self.register_base;
        let endianness = self.endianness;
        self.entries
            .into_iter()
            .map(|ent| ent.into_masked_int_reg(register_base.clone(), endianness))
            .collect()
    }
}

impl Parse for StructRegNode {
    fn parse<T>(node: &mut xml::Node, store: &mut T) -> Self
    where
        T: NodeStore,
    {
        debug_assert_eq!(node.tag_name(), STRUCT_REG);

        let comment = node.attribute_of(COMMENT).unwrap().into();
        let register_base = node.parse(store);

        let endianness = node.parse_if(ENDIANNESS, store).unwrap_or_default();
        let mut entries = vec![];
        while let Some(mut entry_node) = node.next() {
            let entry = entry_node.parse(store);
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
struct StructEntryNode {
    attr_base: NodeAttributeBase,
    elem_base: NodeElementBase,

    p_invalidators: Vec<NodeId>,
    access_mode: AccessMode,
    cacheable: CachingMode,
    polling_time: Option<u64>,
    streamable: bool,
    bit_mask: BitMask,
    sign: Sign,
    unit: Option<String>,
    representation: IntegerRepresentation,
    p_selected: Vec<NodeId>,
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
    fn into_masked_int_reg(
        self,
        mut register_base: RegisterBase,
        endianness: Endianness,
    ) -> MaskedIntRegNode {
        let attr_base = self.attr_base;

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
            endianness,
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
    fn parse<T>(node: &mut xml::Node, store: &mut T) -> Self
    where
        T: NodeStore,
    {
        debug_assert_eq!(node.tag_name(), STRUCT_ENTRY);

        let attr_base = node.parse(store);
        let elem_base = node.parse(store);

        let p_invalidators = node.parse_while(P_INVALIDATOR, store);
        let access_mode = node.parse_if(ACCESS_MODE, store).unwrap_or(AccessMode::RO);
        let cacheable = node.parse_if(CACHEABLE, store).unwrap_or_default();
        let polling_time = node.parse_if(POLLING_TIME, store);
        let streamable = node.parse_if(STREAMABLE, store).unwrap_or_default();
        let bit_mask = node.parse(store);
        let sign = node.parse_if(SIGN, store).unwrap_or_default();
        let unit = node.parse_if(UNIT, store);
        let representation = node.parse_if(REPRESENTATION, store).unwrap_or_default();
        let p_selected = node.parse_while(P_SELECTED, store);

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
    use crate::store::DefaultNodeStore;

    use super::*;

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
        let mut store = DefaultNodeStore::new();
        let node: StructRegNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut store);
        let masked_int_regs: Vec<_> = node.into_masked_int_regs();

        assert_eq!(masked_int_regs.len(), 2);

        let masked_int_reg0 = &masked_int_regs[0];
        assert_eq!(
            masked_int_reg0.node_base().id(),
            store.id_by_name("StructEntry0")
        );
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
        assert_eq!(
            masked_int_reg1.node_base().id(),
            store.id_by_name("StructEntry1")
        );
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
