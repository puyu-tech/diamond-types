/// This module contains the old (original) text encoding format code. This code is used by
/// some early applications written using diamond types. It will probably be moved behind a feature
/// flag once the new encoding code gets written.

mod encode_oplog;
mod decode_oplog;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod fuzzer;
pub mod encode_tools;
mod decode_tools;
pub mod save_transformed;
pub(crate) mod leb;
pub(crate) mod txn_trace;
mod encode_options;

use rle::MergableSpan;
use crate::encoding::varint::*;
use num_enum::TryFromPrimitive;
pub use encode_options::{EncodeOptions, EncodeOptionsBuilder, ENCODE_FULL, ENCODE_PATCH};
pub use crate::encoding::parseerror::ParseError;

/// Original envelope fields and the local range of all submitted operations.
///
/// Returned only after strict consumption and CRC validation. Metadata is copied from the
/// original input, not a re-encoding. Frontiers use local versions in the returned staged log.
#[derive(Debug, Clone)]
pub struct StrictDecodeInfo {
    /// Original document ID, if present.
    pub doc_id: Option<String>,
    /// Original opaque user data, including the distinction between absent and empty.
    pub user_data: Option<Vec<u8>>,
    /// Original agent-name declarations in wire order, including unused declarations.
    pub agent_names: Vec<String>,
    /// Original start frontier mapped into the staged log.
    pub start_frontier: crate::Frontier,
    /// Optional original start text. Not authenticated or checked against the base text.
    pub start_content: Option<String>,
    /// Frontier reached by this artifact.
    pub end_frontier: crate::Frontier,
    /// ALL submitted operations; none were trimmed as known.
    pub submitted: crate::DTRange,
}

const MAGIC_BYTES: [u8; 8] = *b"DMNDTYPS";

const PROTOCOL_VERSION: usize = 0;

// #[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[derive(Debug, PartialEq, Eq, Copy, Clone, TryFromPrimitive)]
#[repr(u32)]
enum ListChunkType {
    /// Packed bytes storing any data compressed in later parts of the file.
    CompressedFieldsLZ4 = 5,

    /// FileInfo contains optional UserData and AgentNames.
    FileInfo = 1,
    DocId = 2,
    AgentNames = 3,
    UserData = 4,

    /// The StartBranch chunk describes the state of the document before included patches have been
    /// applied.
    StartBranch = 10,
    ExperimentalEndBranch = 11,
    Version = 12,
    /// StartBranch content is optional.
    Content = 13,
    ContentCompressed = 14, // Might make more sense to have a generic compression tag for chunks.

    Patches = 20,
    OpVersions = 21,
    OpTypeAndPosition = 22,
    OpParents = 23,

    PatchContent = 24,
    /// ContentKnown is a RLE expressing which ranges of patches have known content
    ContentIsKnown = 25,

    /// A chunk specifying which operations are cancelled when the data is transformed
    TransformedCancelsOps = 27,
    /// A chunk specifying the position deltas for operations when transformed in the stored order
    TransformedPositions = 28,

    Crc = 100,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, TryFromPrimitive)]
#[repr(u32)]
enum DataType {
    Bool = 1,
    VarUInt = 2,
    VarInt = 3,
    PlainText = 4,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, TryFromPrimitive)]
#[repr(u32)]
enum CompressionFormat {
    // Just for future proofing, ya know?
    LZ4 = 1,
}
