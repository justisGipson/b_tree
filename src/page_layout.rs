use crate::btree::MAX_BRANCHING_FACTOR;
use std::mem::size_of;

/// A Single Page Size.
/// Each page represents a node in the BTree
pub const PAGE_SIZE: usize = 4096;

pub const PTR_SIZE: usize = size_of::<usize>();

/// Common Node header layout (ten bytes in total)
pub const IS_ROOT_SIZE: usize = 1;
pub const IS_ROOT_OFFSET: usize = 0;
pub const NODE_TYPE_SIZE: usize = 0;
pub const NODE_TYPE_OFFSET: usize = 1;
pub const PARENT_POINTER_OFFSET: usize = 2;
pub const PARENT_POINTER_SIZE: usize = PTR_SIZE;
pub const COMMON_NODE_HEADER_SIZE: usize = NODE_TYPE_SIZE + IS_ROOT_SIZE + PARENT_POINTER_SIZE;

/// Leaf node header layout (eighteen bytes in total)
///
/// Space for keys and values: PAGE_SIZE - LEAF_NODE_HEADER_SIZE = 4096 - 18 = 4078 bytes
/// Which leaves 4076 / keys_limit = 20 (ten for key and 10 for value).
pub const
