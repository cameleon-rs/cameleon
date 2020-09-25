#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameSpace {
    Standard,
    Custom,
}

impl Default for NameSpace {
    fn default() -> Self {
        Self::Custom
    }
}

impl From<&str> for NameSpace {
    fn from(value: &str) -> Self {
        match value {
            "Standard" => NameSpace::Standard,
            "Custom" => NameSpace::Custom,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Beginner,
    Expert,
    Guru,
    Invisible,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Beginner
    }
}

impl From<&str> for Visibility {
    fn from(value: &str) -> Self {
        match value {
            "Beginner" => Visibility::Beginner,
            "Expert" => Visibility::Expert,
            "Guru" => Visibility::Guru,
            "Invisible" => Visibility::Invisible,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergePriority {
    High,
    Mid,
    Low,
}

impl From<&str> for MergePriority {
    fn from(value: &str) -> Self {
        match value {
            "1" => MergePriority::High,
            "0" => MergePriority::Mid,
            "-1" => MergePriority::Low,
            _ => unreachable!(),
        }
    }
}

impl Default for MergePriority {
    fn default() -> Self {
        MergePriority::Mid
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    RO,
    WO,
    RW,
}

impl From<&str> for AccessMode {
    fn from(value: &str) -> Self {
        match value {
            "RO" => AccessMode::RO,
            "WO" => AccessMode::WO,
            "RW" => AccessMode::RW,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ImmOrPNode<T: Clone> {
    Imm(T),
    PNode(String),
}

impl<T> ImmOrPNode<T>
where
    T: Clone,
{
    pub(super) fn imm(value: T) -> Self {
        Self::Imm(value)
    }

    pub(super) fn pnode(value: String) -> Self {
        Self::PNode(value)
    }
}

pub enum IntegerRepresentation {
    Linear,
    Logarithmic,
    Boolean,
    PureNumber,
    HexNumber,
    IpV4Address,
    MacAddress,
}

impl From<&str> for IntegerRepresentation {
    fn from(value: &str) -> Self {
        match value {
            "Linear" => IntegerRepresentation::Linear,
            "Logarithmic" => IntegerRepresentation::Logarithmic,
            "Boolean" => IntegerRepresentation::Boolean,
            "PureNumber" => IntegerRepresentation::PureNumber,
            "HexNumber" => IntegerRepresentation::HexNumber,
            "IPV4Address" => IntegerRepresentation::IpV4Address,
            "MACAddress" => IntegerRepresentation::MacAddress,
            _ => unreachable!(),
        }
    }
}

pub(super) fn convert_to_bool(value: &str) -> bool {
    match value {
        "Yes" => true,
        "No" => false,
        _ => unreachable!(),
    }
}
