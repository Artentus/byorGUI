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

pub struct Descendants<'a, T: 'a, M: Mutability> {
    nodes: *const T,
    sub_tree_sizes: *const u32,
    len: usize,
    _nodes: PhantomData<M::Ref<'a, [T]>>,
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
}

impl<T> Copy for TreeRef<'_, T, Shared> {}

impl<T> Clone for TreeRef<'_, T, Shared> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T, M: Mutability> Descendants<'a, T, M> {
    #[must_use]
    #[inline]
    fn new(nodes: M::Ref<'a, [T]>, sub_tree_sizes: &'a [u32]) -> Self {
        assert_eq!(nodes.len(), sub_tree_sizes.len());
        let len = nodes.len();

        Self {
            nodes: nodes.as_ptr(),
            sub_tree_sizes: sub_tree_sizes.as_ptr(),
            len,
            _nodes: PhantomData,
        }
    }

    #[must_use]
    #[inline]
    pub fn len(&self) -> u32 {
        self.len as u32
    }

    #[must_use]
    pub fn split_first(self) -> Option<(TreeRef<'a, T, Shared>, Descendants<'a, T, Shared>)> {
        let nodes = unsafe { std::slice::from_raw_parts(self.nodes, self.len) };
        let sub_tree_sizes = unsafe { std::slice::from_raw_parts(self.sub_tree_sizes, self.len) };

        let (child, nodes) = nodes.split_first()?;
        let (child_sub_tree_size, sub_tree_sizes) = sub_tree_sizes.split_first()?;

        let child_sub_tree_size = *child_sub_tree_size as usize;
        assert!(child_sub_tree_size < self.len);

        let (child_nodes, nodes) = nodes.split_at(child_sub_tree_size);
        let (child_sub_tree_sizes, sub_tree_sizes) = sub_tree_sizes.split_at(child_sub_tree_size);

        Some((
            TreeRef {
                parent: child,
                descendants: Descendants::new(child_nodes, child_sub_tree_sizes),
            },
            Descendants::new(nodes, sub_tree_sizes),
        ))
    }

    #[must_use]
    #[inline]
    pub fn reborrow(&self) -> Descendants<'_, T, Shared> {
        Descendants {
            nodes: self.nodes,
            sub_tree_sizes: self.sub_tree_sizes,
            len: self.len,
            _nodes: PhantomData,
        }
    }
}

impl<'a, T> Descendants<'a, T, Exclusive> {
    #[must_use]
    pub fn split_first_mut(
        self,
    ) -> Option<(TreeRef<'a, T, Exclusive>, Descendants<'a, T, Exclusive>)> {
        let nodes = unsafe { std::slice::from_raw_parts_mut(self.nodes.cast_mut(), self.len) };
        let sub_tree_sizes = unsafe { std::slice::from_raw_parts(self.sub_tree_sizes, self.len) };

        let (child, nodes) = nodes.split_first_mut()?;
        let (child_sub_tree_size, sub_tree_sizes) = sub_tree_sizes.split_first()?;

        let child_sub_tree_size = *child_sub_tree_size as usize;
        assert!(child_sub_tree_size < self.len);

        let (child_nodes, nodes) = nodes.split_at_mut(child_sub_tree_size);
        let (child_sub_tree_sizes, sub_tree_sizes) = sub_tree_sizes.split_at(child_sub_tree_size);

        Some((
            TreeRef {
                parent: child,
                descendants: Descendants::new(child_nodes, child_sub_tree_sizes),
            },
            Descendants::new(nodes, sub_tree_sizes),
        ))
    }

    #[must_use]
    #[inline]
    pub fn reborrow_mut(&mut self) -> Descendants<'_, T, Exclusive> {
        Descendants {
            nodes: self.nodes,
            sub_tree_sizes: self.sub_tree_sizes,
            len: self.len,
            _nodes: PhantomData,
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
        while let Some(($subtree, remaining)) = tree.split_first_mut() {
            tree = remaining;
            $body
        }
    };
}
pub(crate) use iter_subtrees;

macro_rules! iter_children {
    ($tree:expr => |$child:ident| $body:stmt) => {
        let mut tree = $tree.reborrow();
        while let Some((TreeRef { parent: $child, .. }, remaining)) = tree.split_first() {
            tree = remaining;
            $body
        }
    };
    ($tree:expr => |mut $child:ident| $body:stmt) => {
        let mut tree = $tree.reborrow_mut();
        while let Some((TreeRef { parent: $child, .. }, remaining)) = tree.split_first_mut() {
            tree = remaining;
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
            $body
            offset += subtree.len();
        }
    };
    ($tree:expr => |mut $child:ident, $index:ident| $body:stmt) => {
        let mut tree = $tree.reborrow_mut();
        let mut index = 0u32;
        while let Some((subtree, remaining)) = tree.split_first_mut() {
            tree = remaining;
            let $child = subtree.parent;
            let $index = offset;
            $body
            offset += subtree.len();
        }
    };
}
pub(crate) use iter_child_indices;

impl<'a, T, M: Mutability> Descendants<'a, T, M> {
    #[must_use]
    pub fn child_count(&self) -> usize {
        let mut count = 0;
        iter_subtrees!(self => |_child| count += 1);
        count
    }
}

impl<T, M: Mutability> Index<u32> for Descendants<'_, T, M> {
    type Output = T;

    #[inline]
    fn index(&self, index: u32) -> &Self::Output {
        let nodes = unsafe { std::slice::from_raw_parts(self.nodes, self.len) };
        &nodes[index as usize]
    }
}

impl<T> IndexMut<u32> for Descendants<'_, T, Exclusive> {
    #[inline]
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        let nodes = unsafe { std::slice::from_raw_parts_mut(self.nodes.cast_mut(), self.len) };
        &mut nodes[index as usize]
    }
}

pub struct Tree<T> {
    nodes: Vec<T>,
    sub_tree_sizes: Vec<u32>,
}

impl<T> Default for Tree<T> {
    #[inline]
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            sub_tree_sizes: Vec::new(),
        }
    }
}

impl<T> Tree<T> {
    #[must_use]
    #[inline]
    pub fn as_ref(&self) -> Option<TreeRef<'_, T, Shared>> {
        assert_eq!(self.nodes.len(), self.sub_tree_sizes.len());
        let (root, nodes) = self.nodes.split_first()?;

        Some(TreeRef {
            parent: root,
            descendants: Descendants::new(nodes, &self.sub_tree_sizes[1..]),
        })
    }

    #[must_use]
    #[inline]
    pub fn as_mut(&mut self) -> Option<TreeRef<'_, T, Exclusive>> {
        assert_eq!(self.nodes.len(), self.sub_tree_sizes.len());
        let (root, nodes) = self.nodes.split_first_mut()?;

        Some(TreeRef {
            parent: root,
            descendants: Descendants::new(nodes, &self.sub_tree_sizes[1..]),
        })
    }
}

pub struct TreeBuilder<'a, T> {
    nodes: &'a mut Vec<T>,
    sub_tree_sizes: &'a mut Vec<u32>,
    parent_index: usize,
}

impl<T> Tree<T> {
    #[inline]
    pub fn insert_root(&mut self, root: T) -> TreeBuilder<'_, T> {
        self.nodes.clear();
        self.sub_tree_sizes.clear();

        self.nodes.push(root);

        TreeBuilder {
            nodes: &mut self.nodes,
            sub_tree_sizes: &mut self.sub_tree_sizes,
            parent_index: 0,
        }
    }
}

impl<T> TreeBuilder<'_, T> {
    #[inline]
    pub fn insert(&mut self, node: T) -> TreeBuilder<'_, T> {
        let index = self.nodes.len();
        self.nodes.push(node);

        TreeBuilder {
            nodes: self.nodes,
            sub_tree_sizes: self.sub_tree_sizes,
            parent_index: index,
        }
    }
}

impl<T> Drop for TreeBuilder<'_, T> {
    #[inline]
    fn drop(&mut self) {
        if self.sub_tree_sizes.len() <= self.parent_index {
            self.sub_tree_sizes.resize(self.parent_index + 1, 0);
        }

        self.sub_tree_sizes[self.parent_index] = (self.nodes.len() - self.parent_index - 1) as u32;
    }
}

impl<T> TreeBuilder<'_, T> {
    #[inline]
    pub fn parent_node(&self) -> &T {
        &self.nodes[self.parent_index]
    }

    #[inline]
    pub fn parent_node_mut(&mut self) -> &mut T {
        &mut self.nodes[self.parent_index]
    }
}
