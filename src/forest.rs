use crate::multi_vec::MultiVec;
use modular_bitfield::prelude::*;
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, Index, IndexMut};

pub trait Mutability: 'static {
    type Ref<'a, T: ?Sized + 'a>: Deref<Target = T>;
}

pub enum Shared {}

impl Mutability for Shared {
    type Ref<'a, T: ?Sized + 'a> = &'a T;
}

pub enum Exclusive {}

impl Mutability for Exclusive {
    type Ref<'a, T: ?Sized + 'a> = &'a mut T;
}

#[derive(Clone, Copy)]
#[bitfield]
struct TreeProperties {
    is_root: bool,
    size: B31,
}

impl fmt::Debug for TreeProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TreeProperties")
            .field("is_root", &self.is_root())
            .field("size", &self.size())
            .finish()
    }
}

pub struct Descendants<'a, T: 'a, M: Mutability> {
    nodes: *mut T,
    tree_properties: *const TreeProperties,
    len: u32,
    _nodes: PhantomData<M::Ref<'a, [T]>>,
    _tree_properties: PhantomData<&'a TreeProperties>,
}

impl<T> Copy for Descendants<'_, T, Shared> {}

impl<T> Clone for Descendants<'_, T, Shared> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

pub struct TreeRef<'a, T: 'a, M: Mutability> {
    pub parent: M::Ref<'a, T>,
    pub descendants: Descendants<'a, T, M>,
    pub is_root: bool,
}

impl<T> Copy for TreeRef<'_, T, Shared> {}

impl<T> Clone for TreeRef<'_, T, Shared> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Descendants<'a, T, Shared> {
    #[must_use]
    #[inline]
    fn new(nodes: &[T], tree_properties: &'a [TreeProperties]) -> Self {
        assert_eq!(nodes.len(), tree_properties.len());
        let len = nodes.len() as u32;

        Self {
            nodes: nodes.as_ptr().cast_mut(),
            tree_properties: tree_properties.as_ptr(),
            len,
            _nodes: PhantomData,
            _tree_properties: PhantomData,
        }
    }
}

impl<'a, T> Descendants<'a, T, Exclusive> {
    #[must_use]
    #[inline]
    fn new_mut(nodes: &mut [T], tree_properties: &'a [TreeProperties]) -> Self {
        assert_eq!(nodes.len(), tree_properties.len());
        let len = nodes.len() as u32;

        Self {
            nodes: nodes.as_mut_ptr(),
            tree_properties: tree_properties.as_ptr(),
            len,
            _nodes: PhantomData,
            _tree_properties: PhantomData,
        }
    }
}

impl<'a, T, M: Mutability> Descendants<'a, T, M> {
    #[must_use]
    #[inline]
    pub fn len(&self) -> u32 {
        self.len
    }

    #[must_use]
    pub fn split_first(self) -> Option<(TreeRef<'a, T, Shared>, Descendants<'a, T, Shared>)> {
        let nodes = unsafe { std::slice::from_raw_parts(self.nodes, self.len as usize) };
        let tree_properties =
            unsafe { std::slice::from_raw_parts(self.tree_properties, self.len as usize) };

        let (child, nodes) = nodes.split_first()?;
        let (child_tree_properties, tree_properties) = tree_properties.split_first()?;

        let child_is_root = child_tree_properties.is_root();
        let child_tree_size = child_tree_properties.size();
        assert!(child_tree_size < self.len);

        let (child_nodes, nodes) = nodes.split_at(child_tree_size as usize);
        let (child_tree_properties, tree_properties) =
            tree_properties.split_at(child_tree_size as usize);

        Some((
            TreeRef {
                parent: child,
                descendants: Descendants::new(child_nodes, child_tree_properties),
                is_root: child_is_root,
            },
            Descendants::new(nodes, tree_properties),
        ))
    }

    #[must_use]
    #[inline]
    pub fn reborrow(&self) -> Descendants<'_, T, Shared> {
        Descendants {
            nodes: self.nodes,
            tree_properties: self.tree_properties,
            len: self.len,
            _nodes: PhantomData,
            _tree_properties: PhantomData,
        }
    }
}

impl<'a, T> Descendants<'a, T, Exclusive> {
    #[must_use]
    pub fn split_first_mut(
        self,
    ) -> Option<(TreeRef<'a, T, Exclusive>, Descendants<'a, T, Exclusive>)> {
        let nodes = unsafe { std::slice::from_raw_parts_mut(self.nodes, self.len as usize) };
        let tree_properties =
            unsafe { std::slice::from_raw_parts(self.tree_properties, self.len as usize) };

        let (child, nodes) = nodes.split_first_mut()?;
        let (child_tree_properties, tree_properties) = tree_properties.split_first()?;

        let child_is_root = child_tree_properties.is_root();
        let child_tree_size = child_tree_properties.size();
        assert!(child_tree_size < self.len);

        let (child_nodes, nodes) = nodes.split_at_mut(child_tree_size as usize);
        let (child_tree_properties, tree_properties) =
            tree_properties.split_at(child_tree_size as usize);

        Some((
            TreeRef {
                parent: child,
                descendants: Descendants::new_mut(child_nodes, child_tree_properties),
                is_root: child_is_root,
            },
            Descendants::new_mut(nodes, tree_properties),
        ))
    }

    #[must_use]
    #[inline]
    pub fn reborrow_mut(&mut self) -> Descendants<'_, T, Exclusive> {
        Descendants {
            nodes: self.nodes,
            tree_properties: self.tree_properties,
            len: self.len,
            _nodes: PhantomData,
            _tree_properties: PhantomData,
        }
    }
}

impl<'a, T, M: Mutability> TreeRef<'a, T, M> {
    #[must_use]
    #[inline]
    pub fn len(&self) -> u32 {
        self.descendants.len() + 1
    }

    #[must_use]
    #[inline]
    pub fn reborrow(&self) -> TreeRef<'_, T, Shared> {
        TreeRef {
            parent: &self.parent,
            descendants: self.descendants.reborrow(),
            is_root: self.is_root,
        }
    }
}

impl<'a, T> TreeRef<'a, T, Exclusive> {
    #[must_use]
    #[inline]
    pub fn reborrow_mut(&mut self) -> TreeRef<'_, T, Exclusive> {
        TreeRef {
            parent: &mut self.parent,
            descendants: self.descendants.reborrow_mut(),
            is_root: self.is_root,
        }
    }
}

macro_rules! iter_subtrees {
    ($tree:expr => |$subtree:ident| $body:stmt) => {
        let mut tree = $tree.reborrow();
        while let Some(($subtree, remaining)) = tree.split_first() {
            tree = remaining;
            $body
        }
    };
    ($tree:expr => |mut $subtree:ident| $body:stmt) => {
        let mut tree = $tree.reborrow_mut();
        #[allow(unused_mut)]
        while let Some((mut $subtree, remaining)) = tree.split_first_mut() {
            tree = remaining;
            #[warn(unused_mut)]
            $body
        }
    };
}
pub(crate) use iter_subtrees;

macro_rules! iter_children {
    ($tree:expr => |$child:ident| $body:stmt) => {
        let mut tree = $tree.reborrow();
        while let Some((
            TreeRef {
                parent: $child,
                is_root,
                ..
            },
            remaining,
        )) = tree.split_first()
        {
            tree = remaining;
            if is_root {
                continue;
            }
            $body
        }
    };
    ($tree:expr => |mut $child:ident| $body:stmt) => {
        let mut tree = $tree.reborrow_mut();
        while let Some((
            TreeRef {
                parent: $child,
                is_root,
                ..
            },
            remaining,
        )) = tree.split_first_mut()
        {
            tree = remaining;
            if is_root {
                continue;
            }
            $body
        }
    };
}
pub(crate) use iter_children;

macro_rules! iter_child_indices {
    ($tree:expr => |$child:ident, $index:ident| $body:stmt) => {
        let mut tree = $tree.reborrow();
        let mut offset = 0u32;
        while let Some((subtree, remaining)) = tree.split_first() {
            tree = remaining;
            let $child = subtree.parent;
            let $index = offset;
            offset += subtree.len();
            if subtree.is_root {
                continue;
            }
            $body
        }
    };
    ($tree:expr => |mut $child:ident, $index:ident| $body:stmt) => {
        let mut tree = $tree.reborrow_mut();
        let mut index = 0u32;
        while let Some((subtree, remaining)) = tree.split_first_mut() {
            tree = remaining;
            let $child = subtree.parent;
            let $index = offset;
            offset += subtree.len();
            if subtree.is_root {
                continue;
            }
            $body
        }
    };
}
pub(crate) use iter_child_indices;

impl<'a, T, M: Mutability> Descendants<'a, T, M> {
    #[must_use]
    pub fn child_count(&self) -> usize {
        let mut count = 0;
        iter_children!(self => |_child| count += 1);
        count
    }
}

impl<T, M: Mutability> Index<u32> for Descendants<'_, T, M> {
    type Output = T;

    #[inline]
    fn index(&self, index: u32) -> &Self::Output {
        let nodes = unsafe { std::slice::from_raw_parts(self.nodes, self.len as usize) };
        &nodes[index as usize]
    }
}

impl<T> IndexMut<u32> for Descendants<'_, T, Exclusive> {
    #[inline]
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        let nodes = unsafe { std::slice::from_raw_parts_mut(self.nodes, self.len as usize) };
        &mut nodes[index as usize]
    }
}

pub struct Forest<T> {
    nodes: MultiVec<(T, TreeProperties)>,
    root_indices: Vec<u32>,
}

impl<T> Default for Forest<T> {
    #[inline]
    fn default() -> Self {
        Self {
            nodes: MultiVec::new(),
            root_indices: Vec::new(),
        }
    }
}

impl<T> Forest<T> {
    #[must_use]
    pub fn primary(&self) -> Option<TreeRef<'_, T, Shared>> {
        let (nodes, tree_properties) = self.nodes.as_slices();
        let (root, nodes) = nodes.split_first()?;
        let tree_properties = &tree_properties[1..];

        Some(TreeRef {
            parent: root,
            descendants: Descendants::new(nodes, tree_properties),
            is_root: true,
        })
    }

    #[must_use]
    pub fn primary_mut(&mut self) -> Option<TreeRef<'_, T, Exclusive>> {
        let (nodes, tree_properties) = self.nodes.as_mut_slices();
        let (root, nodes) = nodes.split_first_mut()?;
        let tree_properties = &tree_properties[1..];

        Some(TreeRef {
            parent: root,
            descendants: Descendants::new_mut(nodes, tree_properties),
            is_root: true,
        })
    }
}

pub struct TreeIter<'a, T: 'a, M: Mutability> {
    forest: M::Ref<'a, Forest<T>>,
    tree_index: usize,
}

impl<T> TreeIter<'_, T, Shared> {
    pub fn next(&mut self) -> Option<TreeRef<'_, T, Shared>> {
        let root_index = *self.forest.root_indices.get(self.tree_index)? as usize;
        self.tree_index += 1;

        let (nodes, tree_properties) = self.forest.nodes.as_slices();
        let nodes = &nodes[root_index..];
        let tree_properties = &tree_properties[root_index..];

        let (parent, nodes) = nodes.split_first().unwrap();
        let (parent_tree_properties, tree_properties) = tree_properties.split_first().unwrap();

        assert!(parent_tree_properties.is_root());
        let tree_size = parent_tree_properties.size() as usize;
        assert!(tree_size <= nodes.len());

        let nodes = &nodes[..tree_size];
        let tree_properties = &tree_properties[..tree_size];

        Some(TreeRef {
            parent,
            descendants: Descendants::new(nodes, tree_properties),
            is_root: true,
        })
    }
}

impl<T> TreeIter<'_, T, Exclusive> {
    pub fn next(&mut self) -> Option<TreeRef<'_, T, Exclusive>> {
        let root_index = *self.forest.root_indices.get(self.tree_index)? as usize;
        self.tree_index += 1;

        let (nodes, tree_properties) = self.forest.nodes.as_mut_slices();
        let nodes = &mut nodes[root_index..];
        let tree_properties = &tree_properties[root_index..];

        let (parent, nodes) = nodes.split_first_mut().unwrap();
        let (parent_tree_properties, tree_properties) = tree_properties.split_first().unwrap();

        assert!(parent_tree_properties.is_root());
        let tree_size = parent_tree_properties.size() as usize;
        assert!(tree_size <= nodes.len());

        let nodes = &mut nodes[..tree_size];
        let tree_properties = &tree_properties[..tree_size];

        Some(TreeRef {
            parent,
            descendants: Descendants::new_mut(nodes, tree_properties),
            is_root: true,
        })
    }
}

impl<T> Forest<T> {
    #[inline]
    pub fn trees(&self) -> TreeIter<'_, T, Shared> {
        TreeIter {
            forest: self,
            tree_index: 0,
        }
    }

    #[inline]
    pub fn trees_mut(&mut self) -> TreeIter<'_, T, Exclusive> {
        TreeIter {
            forest: self,
            tree_index: 0,
        }
    }
}

pub struct ForestBuilder<'a, T> {
    forest: &'a mut Forest<T>,
    parent_index: usize,
}

impl<T> Forest<T> {
    pub fn insert_primary(&mut self, root: T) -> ForestBuilder<'_, T> {
        self.nodes.clear();
        self.root_indices.clear();

        let tree_properties = TreeProperties::new().with_is_root(true);
        self.nodes.push((root, tree_properties));
        self.root_indices.push(0);

        ForestBuilder {
            forest: self,
            parent_index: 0,
        }
    }
}

impl<T> ForestBuilder<'_, T> {
    pub fn insert(&mut self, node: T, is_root: bool) -> ForestBuilder<'_, T> {
        let index = self.forest.nodes.len();
        let tree_properties = TreeProperties::new().with_is_root(is_root);
        self.forest.nodes.push((node, tree_properties));

        if is_root {
            self.forest.root_indices.push(index as u32);
        }

        ForestBuilder {
            forest: self.forest,
            parent_index: index,
        }
    }
}

impl<T> Drop for ForestBuilder<'_, T> {
    fn drop(&mut self) {
        let tree_properties = self.forest.nodes.as_mut_slices().1;
        tree_properties[self.parent_index]
            .set_size((tree_properties.len() - self.parent_index - 1) as u32);
    }
}

impl<T> ForestBuilder<'_, T> {
    #[inline]
    pub fn parent_node(&self) -> &T {
        let nodes = self.forest.nodes.as_slices().0;
        &nodes[self.parent_index]
    }

    #[inline]
    pub fn parent_node_mut(&mut self) -> &mut T {
        let nodes = self.forest.nodes.as_mut_slices().0;
        &mut nodes[self.parent_index]
    }
}
