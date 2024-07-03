/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(clippy::upper_case_acronyms)]
use std::marker::PhantomData;

use super::{
    ivalue::IValue,
    store::{CacheStore, NodeId, NodeStore, ValueStore},
    Device, GenApiResult, ValueCtxt,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameSpace {
    Standard,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Beginner,
    Expert,
    Guru,
    Invisible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergePriority {
    High,
    Mid,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    RO,
    WO,
    RW,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImmOrPNode<T> {
    Imm(T),
    PNode(NodeId),
}

impl<T> ImmOrPNode<T> {
    pub fn imm(self) -> Option<T> {
        match self {
            Self::Imm(value) => Some(value),
            _ => None,
        }
    }

    pub fn pnode(self) -> Option<NodeId> {
        match self {
            Self::PNode(node) => Some(node),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerRepresentation {
    Linear,
    Logarithmic,
    Boolean,
    PureNumber,
    HexNumber,
    IpV4Address,
    MacAddress,
}

impl IntegerRepresentation {
    /// Deduce defalut value of min element.
    pub(super) fn deduce_min(self) -> i64 {
        use IntegerRepresentation::{
            Boolean, HexNumber, IpV4Address, Linear, Logarithmic, MacAddress, PureNumber,
        };
        match self {
            Linear | Logarithmic | Boolean | PureNumber | HexNumber => i64::MIN,
            IpV4Address | MacAddress => 0,
        }
    }

    /// Deduce defalut value of max element.
    pub(super) fn deduce_max(self) -> i64 {
        use IntegerRepresentation::{
            Boolean, HexNumber, IpV4Address, Linear, Logarithmic, MacAddress, PureNumber,
        };
        match self {
            Linear | Logarithmic | Boolean | PureNumber | HexNumber => i64::MAX,
            IpV4Address => 0xffff_ffff,
            MacAddress => 0xffff_ffff_ffff,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatRepresentation {
    Linear,
    Logarithmic,
    PureNumber,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slope {
    Increasing,
    Decreasing,
    Varying,
    Automatic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayNotation {
    Automatic,
    Fixed,
    Scientific,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardNameSpace {
    None,
    IIDC,
    GEV,
    CL,
    USB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachingMode {
    /// Allow to caching on read/write.
    WriteThrough,
    /// Allow to caching on read.
    WriteAround,
    /// Caching is not allowed.
    NoCache,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedValue<T> {
    pub(crate) name: String,
    pub(crate) value: T,
}

impl<T> NamedValue<T> {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> T
    where
        T: Copy,
    {
        self.value
    }

    pub fn value_ref(&self) -> &T {
        &self.value
    }
}

#[derive(Debug, Clone)]
pub enum ValueKind<T> {
    Value(T),
    PValue(PValue<T>),
    PIndex(PIndex<T>),
}

impl<T> ValueKind<T> {
    pub fn imm(&self) -> Option<T>
    where
        T: Copy,
    {
        if let Self::Value(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn p_value(&self) -> Option<&PValue<T>> {
        if let Self::PValue(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn p_index(&self) -> Option<&PIndex<T>> {
        if let Self::PIndex(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct PValue<T> {
    pub(crate) p_value: NodeId,
    pub(crate) p_value_copies: Vec<NodeId>,
    pub(crate) phantom: PhantomData<T>,
}

impl<T> PValue<T> {
    #[must_use]
    pub fn p_value(&self) -> NodeId {
        self.p_value
    }

    #[must_use]
    pub fn p_value_copies(&self) -> &[NodeId] {
        &self.p_value_copies
    }
}

#[derive(Debug, Clone)]
pub struct PIndex<T> {
    pub(crate) p_index: NodeId,
    pub(crate) value_indexed: Vec<ValueIndexed<T>>,
    pub(crate) value_default: ImmOrPNode<T>,
}

impl<T> PIndex<T> {
    #[must_use]
    pub fn p_index(&self) -> NodeId {
        self.p_index
    }

    #[must_use]
    pub fn value_indexed(&self) -> &[ValueIndexed<T>] {
        &self.value_indexed
    }

    #[must_use]
    pub fn value_default(&self) -> ImmOrPNode<T>
    where
        T: Copy,
    {
        self.value_default
    }
}

#[derive(Debug, Clone)]
pub struct ValueIndexed<T> {
    pub(crate) index: i64,
    pub(crate) indexed: ImmOrPNode<T>,
}

impl<T> ValueIndexed<T> {
    #[must_use]
    pub fn index(&self) -> i64 {
        self.index
    }

    #[must_use]
    pub fn indexed(&self) -> ImmOrPNode<T>
    where
        T: Copy,
    {
        self.indexed
    }
}

#[derive(Debug, Clone)]
pub enum AddressKind {
    Address(ImmOrPNode<i64>),
    IntSwissKnife(NodeId),
    PIndex(RegPIndex),
}

impl AddressKind {
    pub(super) fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        match self {
            Self::Address(i) => i.value(device, store, cx),
            Self::IntSwissKnife(nid) => nid.value(device, store, cx),
            Self::PIndex(p_index) => p_index.value(device, store, cx),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegPIndex {
    pub(crate) offset: Option<ImmOrPNode<i64>>,
    pub(crate) p_index: NodeId,
}

impl RegPIndex {
    #[must_use]
    pub fn offset(&self) -> Option<ImmOrPNode<i64>> {
        self.offset
    }

    #[must_use]
    pub fn p_index(&self) -> NodeId {
        self.p_index
    }

    pub(super) fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        let base = self.p_index.value(device, store, cx)?;
        if let Some(offset) = &self.offset {
            let offset: i64 = offset.value(device, store, cx)?;
            Ok(base * offset)
        } else {
            Ok(base)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    LE,
    BE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Signed,
    Unsigned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitMask {
    SingleBit(u64),
    Range { lsb: u64, msb: u64 },
}
