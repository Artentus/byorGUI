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

impl Direction {
    #[inline]
    fn primary_axis(self) -> Axis {
        match self {
            Direction::LeftToRight => Axis::X,
            Direction::TopToBottom => Axis::Y,
        }
    }

    #[inline]
    fn cross_axis(self) -> Axis {
        match self {
            Direction::LeftToRight => Axis::Y,
            Direction::TopToBottom => Axis::X,
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

impl PersistentState {
    #[inline]
    fn scroll_along_axis(&self, axis: Axis) -> Option<Pixel> {
        match axis {
            Axis::X => self.horizontal_scroll,
            Axis::Y => self.vertical_scroll,
        }
    }
}

fn wrap_text(node: &mut Node, text_layout: &mut TextLayout<Brush>) {
    use parley::AlignmentOptions as TextAlignmentOptions;

    let horizontal_padding = node.style.padding.left + node.style.padding.right;
    let wrap_width = node.size.width - horizontal_padding;

    text_layout.break_all_lines(node.style.allow_text_wrap.then_some(wrap_width));
    text_layout.align(
        Some(wrap_width),
        node.style.horizontal_text_alignment,
        TextAlignmentOptions {
            align_when_overflowing: true,
        },
    );
}

impl ByorGui {
    // must be bottom up recursive
    fn compute_node_size(&mut self, node_id: NodeId, axis: Axis) {
        use parley::ContentWidths as TextMeasurements;

        // we have to use index-based iteration because of borrowing
        let child_count = self.child_count(node_id);
        for child_index in 0..child_count {
            let child_id = self.children[node_id][child_index];
            self.compute_node_size(child_id, axis);
        }

        let node = &mut self.nodes[node_id];
        let min_size = node.min_size.along_axis(axis);
        let max_size = node.max_size.along_axis(axis);

        // fixed sizing
        if let Sizing::Fixed(fixed_size) = node.style.size_along_axis(axis) {
            let size = fixed_size.clamp(min_size, max_size);
            *node.min_size.along_axis_mut(axis) = size;
            *node.max_size.along_axis_mut(axis) = size;
            *node.size.along_axis_mut(axis) = size;

            if let Some(&text_layout_id) = node.text_layout.as_ref()
                && (axis == Axis::Y)
            {
                let text_layout = &mut self.text_layouts[text_layout_id];
                wrap_text(node, text_layout);
            }

            return;
        }

        // text sizing
        if let Some(&text_layout_id) = node.text_layout.as_ref() {
            let text_layout = &mut self.text_layouts[text_layout_id];
            let padding: Pixel = node
                .style
                .padding
                .along_axis(axis)
                .into_iter()
                .sum::<Pixel>();

            match axis {
                Axis::X => {
                    let TextMeasurements {
                        min: min_width,
                        max: preferred_width,
                    } = text_layout.calculate_content_widths();

                    let min_width = (min_width + padding).ceil().clamp(min_size, max_size);
                    let width = (preferred_width + padding)
                        .ceil()
                        .clamp(min_width, max_size);

                    node.min_size.width = if node.style.allow_text_wrap {
                        min_width
                    } else {
                        width
                    };
                    node.size.width = width;
                }
                Axis::Y => {
                    wrap_text(node, text_layout);

                    let height = (text_layout.height() + padding)
                        .ceil()
                        .clamp(min_size, max_size);
                    node.min_size.height = height;
                    node.size.height = height;
                }
            }

            if !matches!(node.style.size_along_axis(axis), Sizing::Grow) {
                *node.max_size.along_axis_mut(axis) = node.size.along_axis(axis);
            }

            return;
        }

        // fit content or grow sizing
        {
            let padding = node.style.padding.along_axis(axis);
            let mut min_fit_size: Pixel = padding[0] + padding[1];
            let mut fit_size: Pixel = min_fit_size;

            let (min_child_size, child_size) = if axis.is_primary(node.style.layout_direction) {
                let total_child_spacing =
                    (child_count.saturating_sub(1) as Pixel) * node.style.child_spacing;

                let mut total_min_child_size = total_child_spacing;
                let mut total_child_size = total_child_spacing;
                for (_, child) in self.iter_children(node_id) {
                    total_min_child_size += child.min_size.along_axis(axis);
                    total_child_size += child.size.along_axis(axis);
                }

                (total_min_child_size, total_child_size)
            } else {
                let mut max_min_child_size: Pixel = 0.0;
                let mut max_child_size: Pixel = 0.0;
                for (_, child) in self.iter_children(node_id) {
                    max_min_child_size = max_min_child_size.max(child.min_size.along_axis(axis));
                    max_child_size = max_child_size.max(child.size.along_axis(axis));
                }

                (max_min_child_size, max_child_size)
            };
            min_fit_size += min_child_size;
            fit_size += child_size;

            let min_fit_size = min_fit_size.clamp(min_size, max_size);
            let fit_size = fit_size.clamp(min_size, max_size);

            let node = &mut self.nodes[node_id];
            let scroll = node
                .uid
                .and_then(|uid| self.persistent_state.get(uid))
                .and_then(|persistent_state| persistent_state.scroll_along_axis(axis));
            *node.min_size.along_axis_mut(axis) = if scroll.is_some() {
                min_size
            } else {
                min_fit_size
            };
            *node.size.along_axis_mut(axis) = fit_size;
            if !matches!(node.style.size_along_axis(axis), Sizing::Grow) {
                *node.max_size.along_axis_mut(axis) = fit_size;
            }
        }
    }

    // must be top down recursive
    fn grow_or_shrink_children(&mut self, parent_id: NodeId, axis: Axis) {
        let parent = &self.nodes[parent_id];
        let parent_size = parent.size.along_axis(axis);
        let parent_padding: Pixel = parent.style.padding.along_axis(axis).into_iter().sum();

        let node_count = self.child_count(parent_id);
        if axis.is_primary(parent.style.layout_direction) {
            let total_spacing =
                (node_count.saturating_sub(1) as Pixel) * parent.style.child_spacing;

            let mut total_target_size = parent_size - parent_padding - total_spacing;

            let mut available_space = total_target_size;
            let mut nodes_to_resize = NodeIdVec::new();
            let mut flex_ratio_sum = 0.0;
            for (node_id, node) in self.iter_children(parent_id) {
                available_space -= node.size.along_axis(axis);

                if node.min_size != node.max_size {
                    nodes_to_resize.push(node_id);
                    flex_ratio_sum += node.style.flex_ratio;
                }
            }

            'grow_or_shrink: {
                // if the parent supports scrolling, do not shrink nodes
                let parent_scroll = parent
                    .uid
                    .and_then(|uid| self.persistent_state.get(uid))
                    .and_then(|persistent_state| persistent_state.scroll_along_axis(axis));
                if parent_scroll.is_some() && (available_space <= 0.0) {
                    break 'grow_or_shrink;
                }

                loop {
                    let mut collection_changed = false;
                    nodes_to_resize.retain(|&mut node_id| {
                        let node = &mut self.nodes[node_id];

                        let min_size = node.min_size.along_axis(axis);
                        let max_size = node.max_size.along_axis(axis);

                        let flex_ratio = node.style.flex_ratio;
                        let flex_factor = flex_ratio / flex_ratio_sum;
                        let target_size = total_target_size * flex_factor;

                        if (target_size <= min_size) || (target_size >= max_size) {
                            let new_size = target_size.clamp(min_size, max_size);
                            *node.size.along_axis_mut(axis) = new_size;

                            total_target_size -= new_size;
                            flex_ratio_sum -= flex_ratio;
                            collection_changed = true;
                            false
                        } else {
                            true
                        }
                    });

                    if !collection_changed {
                        break;
                    }
                }

                for node_id in nodes_to_resize.drain(..) {
                    let node = &mut self.nodes[node_id];

                    let flex_ratio = node.style.flex_ratio;
                    let flex_factor = flex_ratio / flex_ratio_sum;
                    let target_size = total_target_size * flex_factor;
                    *node.size.along_axis_mut(axis) = target_size;
                }
            }
        } else {
            let available_space = parent_size - parent_padding;
            let mut nodes = self.iter_children_mut(parent_id);
            while let Some(node) = nodes.next() {
                let node_min_size = node.min_size.along_axis(axis);
                let node_max_size = node.max_size.along_axis(axis);
                *node.size.along_axis_mut(axis) =
                    available_space.clamp(node_min_size, node_max_size);
            }
        }

        // we have to use index-based iteration because of borrowing
        for node_index in 0..node_count {
            let node_id = self.children[parent_id][node_index];
            self.grow_or_shrink_children(node_id, axis);
        }
    }

    fn position_children(&mut self, parent_id: NodeId) {
        let parent = &mut self.nodes[parent_id];
        let parent_layout_direction = parent.style.layout_direction;
        let parent_child_spacing = parent.style.child_spacing;
        let parent_child_alignment = parent.style.child_alignment;

        if let Some(&text_layout_id) = parent.text_layout.as_ref() {
            let text_layout = &self.text_layouts[text_layout_id];

            parent.vertical_text_offset = match parent.style.vertical_text_alignment {
                VerticalTextAlignment::Top => 0.0,
                VerticalTextAlignment::Center => {
                    (parent.size.height
                        - text_layout.height()
                        - parent.style.padding.top
                        - parent.style.padding.bottom)
                        / 2.0
                }
                VerticalTextAlignment::Bottom => {
                    parent.size.height
                        - text_layout.height()
                        - parent.style.padding.top
                        - parent.style.padding.bottom
                }
            };
        }

        // primary axis
        {
            let axis = parent_layout_direction.primary_axis();

            let parent = &self.nodes[parent_id];
            let parent_position = parent.position.along_axis(axis);
            let parent_size = parent.size.along_axis(axis);
            let parent_padding = parent.style.padding.along_axis(axis);
            let parent_scroll = parent
                .uid
                .and_then(|uid| self.persistent_state.get(uid))
                .and_then(|persistent_state| persistent_state.scroll_along_axis(axis))
                .unwrap_or_default();

            let mut nodes = self.iter_children_mut(parent_id);

            let mut offset = 0.0;
            while let Some(node) = nodes.next() {
                *node.position.along_axis_mut(axis) =
                    parent_position + parent_padding[0] + offset - parent_scroll;

                offset += node.size.along_axis(axis);
                offset += parent_child_spacing;
            }

            let total_node_size = (offset - parent_child_spacing).max(0.0);
            let alignment_offset = match parent_child_alignment {
                Alignment::Start => 0.0,
                Alignment::Center => (parent_size - total_node_size) / 2.0 - parent_padding[0],
                Alignment::End => {
                    parent_size - total_node_size - parent_padding[0] - parent_padding[1]
                }
            };

            nodes.reset();
            while let Some(node) = nodes.next() {
                *node.position.along_axis_mut(axis) += alignment_offset.max(0.0);
            }
        }

        // cross axis
        {
            let axis = parent_layout_direction.cross_axis();

            let parent = &self.nodes[parent_id];
            let parent_position = parent.position.along_axis(axis);
            let parent_size = parent.size.along_axis(axis);
            let parent_padding = parent.style.padding.along_axis(axis);
            let parent_scroll = parent
                .uid
                .and_then(|uid| self.persistent_state.get(uid))
                .and_then(|persistent_state| persistent_state.scroll_along_axis(axis))
                .unwrap_or_default();

            let mut nodes = self.iter_children_mut(parent_id);

            while let Some(node) = nodes.next() {
                *node.position.along_axis_mut(axis) = match node.style.cross_axis_alignment {
                    Alignment::Start => parent_position + parent_padding[0],
                    Alignment::Center => {
                        parent_position + (parent_size - node.size.along_axis(axis)) / 2.0
                    }
                    Alignment::End => {
                        parent_position + parent_size
                            - node.size.along_axis(axis)
                            - parent_padding[1]
                    }
                } - parent_scroll;
            }
        }

        // we have to use index-based iteration because of borrowing
        let node_count = self.child_count(parent_id);
        for node_index in 0..node_count {
            let node_id = self.children[parent_id][node_index];
            self.position_children(node_id);
        }
    }

    pub(crate) fn layout(&mut self, root_id: NodeId) {
        self.compute_node_size(root_id, Axis::X);
        self.grow_or_shrink_children(root_id, Axis::X);
        self.compute_node_size(root_id, Axis::Y);
        self.grow_or_shrink_children(root_id, Axis::Y);
        self.position_children(root_id);
    }
}
