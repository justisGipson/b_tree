use crate::error::Error;
use crate::node::Node;
use crate::node_type::{Key, KeyValuePair, NodeType, Offset};
use crate::page::Page;
use crate::pager::Pager;
use std::cmp;
use std::convert::TryFrom;
use std::path::Path;

/// BTREE properties
pub const MAX_BRANCHING_FACTOR: usize = 200;
pub const NODE_KEYS_LIMIT: usize = MAX_BRANCHING_FACTOR - 1;

/// BTree struct represents an on-disk B+Tree
/// Each node is persisted in the table file, leaf nodes contain the values
pub struct BTree {
  pager: Pager,
  b: usize,
  root_offset: Offset,
}

/// BtreeBuilder is a Builder for the BTree struct
pub struct BTreeBuilder {
  /// Path to the tree file
  path: &'static Path,
  /// Btree param, an inner node contains no mor than 2*b-1 keys and no less than b-1 keys
  /// and no more than 2*b children and no less than b children
  b: usize,
}

impl BTreeBuilder {
  pub fn new() -> BTreeBuilder {
    BTreeBuilder {
      path: Path::new(""),
      b: 0,
    }
  }

  pub fn path(mut self, path: &'static Path) -> BTreeBuilder {
    self.path = path;
    self
  }

  pub fn b_parameter(mut self, b: usize) -> BTreeBuilder {
    self.b = b;
    self
  }

  pub fn build(&self) -> Result<BTree, Error> {
    if self.path.to_string_lossy() == "" {
      return Err(Error::UnexpectedError);
    }
    if self.b == 0 {
      return Err(Error::UnexpectedError);
    }

    let mut pager = Pager::new(&self.path)?;
    let root = Node::new(NodeType::Leaf(vec![]), true, None);
    let root_offset = pager.write_page(Page::try_from(&root)?)?;
    Ok(BTree {
      pager,
      b: self.b,
      root_offset,
    })
  }
}

impl Default for BTreeBuilder {
  /// A default bTreeBuilder provides a builder with:
  /// - b parameter set to 200
  /// - path set to '/tmp/db'
  fn default() -> Self {
    BTreeBuilder::new()
      .b_parameter(200)
      .path(Path::new("/tmp/db"))
  }
}

impl BTree {
  fn is_node_full(&self, node: &Node) -> Result<bool, Error> {
    match &node.node_type {
      NodeType::Leaf(pairs) => Ok(pairs.len() == (2 * self.b -1)),
      NodeType::Internal(_, keys) => Ok(keys.len() == (2 * self.b -1)),
      NodeType::Unexpected => Err(Error::UnexpectedError),
    }
  }

  fn is_node_underflow(&self, node: &Node) -> Result<bool, Error> {
    match &node.node_type {
      // A root cannot really be "underflowing" as it can contain less than b-1 keys/pointers
      NodeType::Leaf(pairs) => Ok(pairs.len() < self.b - 1 && !node.is_root),
      NodeType::Internal(_, keys) => Ok(keys.len() < self.b - 1 && !node.is_root),
      NodeType::Unexpected => Err(Error::UnexpectedError),
    }
  }

  /// insert a key value pair possibly splitting nodes along the way
  pub fn insert(&mut self, kv: KeyValuePair) -> Result<(), Error> {
    let root_page = self.pager.get_page(&self.root_offset)?;
  }
}
