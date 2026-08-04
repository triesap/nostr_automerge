#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Actor([u8; 32]);

impl Actor {
    pub(crate) const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub(crate) const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Hash([u8; 32]);

impl Hash {
    pub(crate) const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub(crate) const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OpId {
    pub(crate) counter: u64,
    pub(crate) actor: Actor,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ObjectId {
    Root,
    Operation(OpId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Key {
    Map(String),
    Head,
    Element(OpId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ObjectKind {
    Map,
    List,
    Text,
    Table,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Scalar {
    Bytes(Vec<u8>),
    String(String),
    Int(i64),
    Uint(u64),
    F64Bits(u64),
    Counter(i64),
    Timestamp(i64),
    Boolean(bool),
    Null,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    Make(ObjectKind),
    Set(Scalar),
    Delete,
    Increment(i64),
    MarkBegin {
        name: String,
        value: Scalar,
        expand: bool,
    },
    MarkEnd {
        expand: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Operation {
    pub(crate) object: ObjectId,
    pub(crate) key: Key,
    pub(crate) predecessors: Vec<OpId>,
    pub(crate) insert: bool,
    pub(crate) action: Action,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecodedChange {
    pub(crate) hash: Hash,
    pub(crate) actor: Actor,
    pub(crate) sequence: u64,
    pub(crate) start_op: u64,
    pub(crate) dependencies: Vec<Hash>,
    pub(crate) operations: Vec<Operation>,
    pub(crate) time: i64,
    pub(crate) message: Option<String>,
    pub(crate) extra_bytes: Vec<u8>,
}
