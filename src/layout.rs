use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Axis {
    X,
    Y,
}

impl Axis {
    #[inline]
    fn is_primary(self, direction: Direction) -> bool {
        match (self, direction) {
            (Axis::X, Direction::LeftToRight) => true,
            (Axis::X, Direction::TopToBottom) => false,
            (Axis::Y, Direction::LeftToRight) => false,
            (Axis::Y, Direction::TopToBottom) => true,
        }
    }
}

impl Position {
    #[inline]
    fn along_axis(self, axis: Axis) -> Pixel {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
        }
    }

    #[inline]
    fn along_axis_mut(&mut self, axis: Axis) -> &mut Pixel {
        match axis {
            Axis::X => &mut self.x,
            Axis::Y => &mut self.y,
        }
    }
}

impl Size {
    #[inline]
    fn along_axis(self, axis: Axis) -> Pixel {
        match axis {
            Axis::X => self.width,
            Axis::Y => self.height,
        }
    }

    #[inline]
    fn along_axis_mut(&mut self, axis: Axis) -> &mut Pixel {
        match axis {
            Axis::X => &mut self.width,
            Axis::Y => &mut self.height,
        }
    }
}

impl Padding {
    #[inline]
    fn along_axis(self, axis: Axis) -> [Pixel; 2] {
        match axis {
            Axis::X => [self.left, self.right],
            Axis::Y => [self.top, self.bottom],
        }
    }
}

impl ComputedStyle {
    #[inline]
    fn size_along_axis(&self, axis: Axis) -> Sizing {
        match axis {
            Axis::X => self.width,
            Axis::Y => self.height,
        }
    }
}

impl ByorGui {
    // must be bottom up recursive
    fn compute_node_size(&mut self, node_id: NodeId, axis: Axis) {
        // we have to use index-based iteration because of borrowing
        let child_count = self.child_count(node_id);
        for child_index in 0..child_count {
            let child_id = self.children[node_id][child_index];
            self.compute_node_size(child_id, axis);
        }

        let node = &mut self.nodes[node_id];

        if let Sizing::Fixed(fixed_size) = node.style.size_along_axis(axis) {
            *node.size.along_axis_mut(axis) = fixed_size;
            return;
        }

        let padding = node.style.padding.along_axis(axis);
        *node.size.along_axis_mut(axis) = padding[0] + padding[1];

        let child_size = if axis.is_primary(node.style.layout_direction) {
            let total_child_spacing =
                (child_count.saturating_sub(1) as Pixel) * node.style.child_spacing;
            let total_child_size: Pixel = self
                .iter_children(node_id)
                .map(|child| child.size.along_axis(axis))
                .sum();

            total_child_spacing + total_child_size
        } else {
            let max_child_size = self
                .iter_children(node_id)
                .map(|child| child.size.along_axis(axis))
                .max_by(Pixel::total_cmp)
                .unwrap_or_default();

            max_child_size
        };
        *self.nodes[node_id].size.along_axis_mut(axis) += child_size;
    }

    // must be top down recursive
    fn grow_or_shrink_children(&mut self, parent_id: NodeId, axis: Axis) {
        let parent = &self.nodes[parent_id];
        let parent_size = parent.size.along_axis(axis);
        let parent_padding: Pixel = parent.style.padding.along_axis(axis).into_iter().sum();

        let child_count = self.child_count(parent_id);
        if axis.is_primary(parent.style.layout_direction) {
            let total_spacing =
                (child_count.saturating_sub(1) as Pixel) * parent.style.child_spacing;
            let mut available_space = parent_size
                - parent_padding
                - total_spacing
                - self
                    .iter_children(parent_id)
                    .filter(|&node| !matches!(node.style.size_along_axis(axis), Sizing::Grow))
                    .map(|node| node.size.along_axis(axis))
                    .sum::<Pixel>();

            if available_space > Pixel::EPSILON {
                // grow
                let mut children_to_grow: NodeIdVec = self
                    .child_ids(parent_id)
                    .iter()
                    .copied()
                    .filter(|&child_id| {
                        matches!(
                            self.nodes[child_id].style.size_along_axis(axis),
                            Sizing::Grow,
                        )
                    })
                    .collect();

                loop {
                    let target_size = available_space / (children_to_grow.len() as Pixel);

                    let mut collection_altered = false;
                    children_to_grow.retain(|&mut child_id| {
                        let child_size = self.nodes[child_id].size.along_axis(axis);
                        if child_size > target_size {
                            available_space -= child_size;
                            collection_altered = true;
                            false
                        } else {
                            true
                        }
                    });

                    if !collection_altered {
                        for child_id in children_to_grow {
                            *self.nodes[child_id].size.along_axis_mut(axis) = target_size;
                        }
                        break;
                    }
                }
            } else if available_space < Pixel::EPSILON {
                // shrink
                // TODO
            }
        } else {
            let available_space = parent_size - parent_padding;
            let mut children = self.iter_children_mut(parent_id);
            while let Some(child) = children.next() {
                if matches!(child.style.size_along_axis(axis), Sizing::Grow) {
                    *child.size.along_axis_mut(axis) = available_space;
                }
            }
        }

        // we have to use index-based iteration because of borrowing
        for child_index in 0..child_count {
            let child_id = self.children[parent_id][child_index];
            self.grow_or_shrink_children(child_id, axis);
        }
    }

    fn position_children(&mut self, parent_id: NodeId, axis: Axis) {
        let parent = &self.nodes[parent_id];
        let parent_position = parent.position.along_axis(axis);
        let parent_size = parent.size.along_axis(axis);
        let parent_padding = parent.style.padding.along_axis(axis);
        let parent_layout_direction = parent.style.layout_direction;
        let parent_child_spacing = parent.style.child_spacing;
        let parent_child_alignment = parent.style.child_alignment;

        let mut children = self.iter_children_mut(parent_id);
        if axis.is_primary(parent_layout_direction) {
            let mut offset = 0.0;
            while let Some(child) = children.next() {
                *child.position.along_axis_mut(axis) = parent_position + parent_padding[0] + offset;

                offset += child.size.along_axis(axis);
                offset += parent_child_spacing;
            }

            let total_child_size = (offset - parent_child_spacing).max(0.0);
            let alignment_offset = match parent_child_alignment {
                Alignment::Start => 0.0,
                Alignment::Center => (parent_size - total_child_size) / 2.0 - parent_padding[0],
                Alignment::End => {
                    parent_size - total_child_size - parent_padding[0] - parent_padding[1]
                }
            };

            children.reset();
            while let Some(child) = children.next() {
                *child.position.along_axis_mut(axis) += alignment_offset;
            }
        } else {
            while let Some(child) = children.next() {
                *child.position.along_axis_mut(axis) = match child.style.cross_axis_alignment {
                    Alignment::Start => parent_position + parent_padding[0],
                    Alignment::Center => {
                        parent_position + (parent_size - child.size.along_axis(axis)) / 2.0
                    }
                    Alignment::End => {
                        parent_position + parent_size
                            - child.size.along_axis(axis)
                            - parent_padding[1]
                    }
                };
            }
        }

        // we have to use index-based iteration because of borrowing
        let child_count = self.child_count(parent_id);
        for child_index in 0..child_count {
            let child_id = self.children[parent_id][child_index];
            self.position_children(child_id, axis);
        }
    }

    pub(crate) fn layout(&mut self, root_id: NodeId) {
        self.compute_node_size(root_id, Axis::X);
        self.grow_or_shrink_children(root_id, Axis::X);
        // TODO: text wrapping
        self.compute_node_size(root_id, Axis::Y);
        self.grow_or_shrink_children(root_id, Axis::Y);
        self.position_children(root_id, Axis::X);
        self.position_children(root_id, Axis::Y);
    }
}
