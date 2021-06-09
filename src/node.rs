use crate::error::Error;
use crate::node_type::{Key, KeyValuePair, NodeType, Offset};
use crate::page::Page;
use crate::page_layout::{
  FromByte, INTERNAL_NODE_HEADER_SIZE, INTERNAL_NODE_NUM_CHILDREN_OFFSET, IS_ROOT_OFFSET, KEY_SIZE, LEAF_NODE_HEADER_SIZE, LEAF_NODE_NUM_PAIRS_OFFSET, NODE_TYPE_OFFSET, PARENT_POINTER_OFFSET, PTR_SIZE, VALUE_SIZE,
};

use std::convert::TryFrom;
use std::str;

/// Node represents a node in the BTree occupied by a single page in memory
#[derive(Clone, Debug)]
pub struct Node {
  pub node_type: NodeType,
  pub is_root: bool,
  pub parent_offset: Option<Offset>,
}

// Node represents a node in the BTree
impl Node {
  pub fn new(node_type: NodeType, is_root: bool, parent_offset: Option<Offset>) -> Node {

  }
}
