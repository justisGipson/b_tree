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
    let mut root = Node::try_from(root_page)?;
    if self.is_node_full(&root)? {
      let mut old_root = &mut root;
      let old_root_offset = self.root_offset.clone();
      let mut new_root = Node::new(NodeType::Internal(vec![], vec![]), true, None);
      // write the new root to disk
      let new_root_offset = self.pager.write_page(Page::try_from(&new_root)?)?;
      // Set the current roots parent to the new root
      old_root.parent_offset = Some(new_root_offset.clone());
      old_root.is_root = false;
      // update root offset.
      self.root_offset = new_root_offset;
      // split old_root
      let (median, sibling) = old_root.split(self.b)?;
      // write olf root with tts new data to disks
      self.pager.write_page_at_offset(Page::try_from(&*old_root)?, &old_root_offset)?;
      // write the newly created sibling with its children and key
      let sibling_offset = self.pager.write_page(Page::try_from(&sibling)?)?;
      // update new root with its children and key
      new_root.node_type = NodeType::Internal(vec![old_root_offset, sibling_offset], vec![median]);
      // write the new_root to disk
      self.pager.write_page_at_offset(Page::try_from(&new_root), &self.root_offset)?;
      // assign new root
      root = new_root;
    }
    self.insert_non_full(&mut root, self.root_offset.clone(), kv)
  }

  /// insert_non_full (recursively) finds a node rooted at a fiven non-full node
  /// to insert a given kv pair
  fn insert_non_full(
    &mut self,
    node: &mut Node,
    node_offset: Offset,
    kv: KeyValuePair,
  ) -> Result<(), Error> {
    match &mut node.node_type {
      NodeType::Leaf(ref mut pairs) => {
        let idx = pairs.binary_search(&kv).unwrap_or_else(|x| x);
        pairs.insert(idx, kv);
        self.pager.write_page_at_offset(Page::try_from(&*node)?,&node_offset)
      }
      NodeType::Internal(ref mut children, ref mut keys) => {
        let idx = keys.binary_search(&Key(kv.key.clone())).unwrap_or_else(|x| x);
        let child_offset = children.get(idx).ok_or(Error::UnexpectedError)?.clone();
        let child_page = self.pager.get_page(&child_offset)?;
        let mut child = Node::try_from(child_page)?;
        if self.is_node_full(&child)? {
          // split will split the child at b leaving the [0, b-1] keys
          // while moving the set of [b, 2b-1] keys to the sibling
          let (median, mut sibling) = child.split(self.b)?;
          self.pager.write_page_at_offset(Page::try_from(&child)?, &child_offset)?;
          // write newly created sibling to disk
          let sibling_offset = self.pager.write_page(Page::try_from(&sibling)?)?;
          // siblings keys are larger than the split child, thus needs to be inserted
          // at next index
          children.insert(idx + 1, sibling_offset.clone());
          keys.insert(idx, median.clone());

          // write parent page to disk
          self.pager.write_page_at_offset(Page::try_from(&*node)?, &node_offset)?;
          // continue recursively
          if kv.key <= median.0 {
            self.insert_non_full(&mut child, child_offset, kv)
          } else {
            self.insert_non_full(&mut sibling, sibling_offset, kv)
          }
        } else {
          self.insert_non_full(&mut child, child_offset, kv)
        }
      }
      NodeType::Unexpected => Err(Error::UnexpectedError),
    }
  }

  pub fn search(&mut self, key: String) -> Result<KeyValuePair, Error> {
    let root_page = self.pager.get_page(&self.root_offset)?;
    let root = Node::try_from(root_page)?;
    self.search_node(root, &key)
  }

  fn search_node(&mut self, node: Node, search: &str) -> Result<KeyValuePair, Error> {
    match node.node_type {
      NodeType::Internal(children, keys) => {
        let idx = keys.binary_search(&Key(search.to_string()))
        .unwrap_or_else(|x| x);
        // retrieve child page from disk and deserialize
        let child_offset = children.get(idx).ok_or(Error::UnexpectedError)?;
        let page = self.pager.get_page(child_offset)?;
        let child_node = Node::try_from(page)?;
        self.search_node(child_node, search)
      }
      NodeType::Leaf(pairs) => {
        if let Ok(idx) = pairs.binary_search_by_key(&search.to_string(), |pair| pair.key.clone())
        {
          return Ok(pairs[idx].clone());
        }
        Err(Error::KeyNotFound)
      }
      NodeType::Unexpected => Err(Error::UnexpectedError),
    }
  }

  /// delete deletes a given key from the tree
  pub fn delete(&mut self, key: Key) -> Result<(), Error> {
    self.delete_key_from_subtree(key, &self.root_offset.clone())
  }

  /// delete key from subtree recursively traverses a tree rooted at a node in certain offset
  /// until it finds the given key and deletes
  fn delete_key_from_subtree(&mut self, key: Key, offset: &Offset) -> Result<(), Error> {
    let page = self.pager.get_page(offset)?;
    let mut node = Node::try_from(page)?;

  }
}
