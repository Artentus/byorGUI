use crate::style::axis::*;
use crate::*;
use smallvec::SmallVec;

#[must_use]
fn scroll_along_axis(
    persistent_state: Option<&PersistentState>,
    axis: Axis,
) -> Option<Float<Pixel>> {
    let key = match axis {
        Axis::X => PersistentStateKey::HorizontalScroll,
        Axis::Y => PersistentStateKey::VerticalScroll,
    };

    persistent_state
        .and_then(|persistent_state| persistent_state.get::<Float<Pixel>>(key))
        .copied()
}

fn wrap_text(node: &mut Node, text_layout: &mut TextLayout<Color>) {
    use parley::AlignmentOptions as TextAlignmentOptions;

    let horizontal_padding = node.style.padding().left + node.style.padding().right;
    let wrap_width = (node.style.fixed_size.x - horizontal_padding).value();

    text_layout.break_all_lines(node.style.text_wrap().then_some(wrap_width));
    text_layout.align(
        Some(wrap_width),
        node.style.horizontal_text_alignment().into(),
        TextAlignmentOptions {
            align_when_overflowing: true,
        },
    );
}

// must be bottom up recursive
fn compute_node_size<Renderer: rendering::Renderer>(
    tree: TreeRef<'_, Node, Exclusive>,
    data: &mut ByorGuiData<Renderer>,
    axis: Axis,
) {
    use parley::ContentWidths as TextMeasurements;

    let TreeRef {
        parent: node,
        mut descendants,
        is_root,
    } = tree;

    iter_subtrees!(descendants => |mut subtree| {
        compute_node_size(subtree, data, axis);
    });

    let min_size = node.style.min_size.along_axis(axis);
    let max_size = node.style.max_size.along_axis(axis);

    // fixed sizing
    if node.style.size_along_axis(axis) == ComputedSizing::Fixed {
        let size = node.style.fixed_size.along_axis(axis);
        *node.style.min_size.along_axis_mut(axis) = size;
        *node.style.max_size.along_axis_mut(axis) = size;

        if let Some(text_layout_id) = node.text_layout.expand()
            && (axis == Axis::Y)
        {
            let text_layout = &mut data.text_layouts[text_layout_id];
            wrap_text(node, text_layout);
        }

        return;
    }

    // text sizing
    if let Some(text_layout_id) = node.text_layout.expand() {
        let text_layout = &mut data.text_layouts[text_layout_id];
        let padding: Float<Pixel> = node.style.padding().along_axis(axis).into_iter().sum();

        match axis {
            Axis::X => {
                let TextMeasurements {
                    min: min_width,
                    max: preferred_width,
                } = text_layout.calculate_content_widths();

                let min_width = (min_width.px().ceil() + padding).clamp(min_size, max_size);
                let width = (preferred_width.px().ceil() + padding).clamp(min_width, max_size);

                node.style.min_size.x = if node.style.text_wrap() {
                    min_width
                } else {
                    width
                };
                node.style.fixed_size.x = width;
            }
            Axis::Y => {
                wrap_text(node, text_layout);

                let height = (text_layout.height().px().ceil() + padding).clamp(min_size, max_size);
                node.style.min_size.y = height;
                node.style.fixed_size.y = height;
            }
        }

        if is_root || (node.style.size_along_axis(axis) != ComputedSizing::Grow) {
            *node.style.max_size.along_axis_mut(axis) = node.style.fixed_size.along_axis(axis);
        }

        return;
    }

    // fit content or grow sizing
    {
        let mut min_fit_size: Float<Pixel> =
            node.style.padding().along_axis(axis).into_iter().sum();
        let mut fit_size = min_fit_size;

        let (min_child_size, child_size) = if axis.is_primary(node.style.layout_direction()) {
            let mut child_count = 0u32;
            let mut total_min_child_size = 0.px();
            let mut total_child_size = 0.px();
            iter_children!(descendants => |child| {
                child_count += 1;
                total_min_child_size += child.style.min_size.along_axis(axis);
                total_child_size += child.style.fixed_size.along_axis(axis);
            });

            let total_child_spacing =
                (child_count.saturating_sub(1) as f32) * node.style.child_spacing();
            total_min_child_size += total_child_spacing;
            total_child_size += total_child_spacing;

            (total_min_child_size, total_child_size)
        } else {
            let mut max_min_child_size = 0.px();
            let mut max_child_size = 0.px();
            iter_children!(descendants => |child| {
                max_min_child_size =
                    max_min_child_size.max(child.style.min_size.along_axis(axis));
                max_child_size = max_child_size.max(child.style.fixed_size.along_axis(axis));
            });

            (max_min_child_size, max_child_size)
        };
        min_fit_size += min_child_size;
        fit_size += child_size;

        let min_fit_size = min_fit_size.clamp(min_size, max_size);
        let fit_size = fit_size.clamp(min_size, max_size);

        let scroll = node
            .uid
            .and_then(|uid| scroll_along_axis(data.persistent_state.get(uid), axis));

        *node.style.min_size.along_axis_mut(axis) = if scroll.is_some() {
            min_size
        } else {
            min_fit_size
        };
        *node.style.fixed_size.along_axis_mut(axis) = fit_size;
        if is_root || (node.style.size_along_axis(axis) != ComputedSizing::Grow) {
            *node.style.max_size.along_axis_mut(axis) = fit_size;
        }
    }
}

// must be top down recursive
fn grow_or_shrink_children<Renderer: rendering::Renderer>(
    tree: TreeRef<'_, Node, Exclusive>,
    data: &mut ByorGuiData<Renderer>,
    axis: Axis,
) {
    let TreeRef {
        parent,
        mut descendants,
        ..
    } = tree;

    let parent_size = parent.style.fixed_size.along_axis(axis);
    let parent_padding: Float<Pixel> = parent.style.padding().along_axis(axis).into_iter().sum();

    if axis.is_primary(parent.style.layout_direction()) {
        let node_count = descendants.child_count();
        let total_spacing = (node_count.saturating_sub(1) as f32) * parent.style.child_spacing();

        let mut total_target_size = parent_size - parent_padding - total_spacing;
        let mut available_space = total_target_size;
        let mut nodes_to_resize = SmallVec::<[u32; 10]>::new();
        let mut flex_ratio_sum = 0.0;
        iter_child_indices!(descendants => |node, node_index| {
            available_space -= node.style.fixed_size.along_axis(axis);

            // filter out nodes that cannot be resized
            if node.style.min_size.along_axis(axis) == node.style.max_size.along_axis(axis) {
                total_target_size -= node.style.fixed_size.along_axis(axis);
            } else {
                nodes_to_resize.push(node_index);
                flex_ratio_sum += node.style.flex_ratio();
            }
        });

        'grow_or_shrink: {
            // if the parent supports scrolling, do not shrink nodes
            let parent_scroll = parent
                .uid
                .and_then(|uid| scroll_along_axis(data.persistent_state.get(uid), axis));
            if parent_scroll.is_some() && (available_space <= 0.px()) {
                break 'grow_or_shrink;
            }

            loop {
                let mut collection_changed = false;
                nodes_to_resize.retain(|&mut node_index| {
                    let node = &mut descendants[node_index];

                    let min_size = node.style.min_size.along_axis(axis);
                    let max_size = node.style.max_size.along_axis(axis);

                    let flex_ratio = node.style.flex_ratio();
                    let flex_factor = if flex_ratio_sum > 0.0 {
                        flex_ratio / flex_ratio_sum
                    } else {
                        0.0
                    };
                    let target_size = total_target_size * flex_factor;

                    if (target_size <= min_size) || (target_size >= max_size) {
                        let new_size = target_size.clamp(min_size, max_size);
                        *node.style.fixed_size.along_axis_mut(axis) = new_size;

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
                let node = &mut descendants[node_id];

                let flex_ratio = node.style.flex_ratio();
                let flex_factor = if flex_ratio_sum > 0.0 {
                    flex_ratio / flex_ratio_sum
                } else {
                    0.0
                };
                let target_size = total_target_size * flex_factor;
                *node.style.fixed_size.along_axis_mut(axis) = target_size;
            }
        }
    } else {
        let available_space = parent_size - parent_padding;
        iter_children!(descendants => |mut node| {
            let node_min_size = node.style.min_size.along_axis(axis);
            let node_max_size = node.style.max_size.along_axis(axis);
            *node.style.fixed_size.along_axis_mut(axis) =
                available_space.clamp(node_min_size, node_max_size);
        });
    }

    iter_subtrees!(descendants => |mut subtree| {
        grow_or_shrink_children(subtree, data, axis);
    });
}

fn position_children<Renderer: rendering::Renderer>(
    tree: TreeRef<'_, Node, Exclusive>,
    data: &mut ByorGuiData<Renderer>,
) {
    let TreeRef {
        parent,
        mut descendants,
        ..
    } = tree;

    if let Some(text_layout_id) = parent.text_layout.expand() {
        let text_layout = &data.text_layouts[text_layout_id];

        parent.vertical_text_offset = match parent.style.vertical_text_alignment() {
            VerticalTextAlignment::Top => 0.px(),
            VerticalTextAlignment::Center => {
                (parent.style.fixed_size.y
                    - text_layout.height().px().ceil()
                    - parent.style.padding().top
                    - parent.style.padding().bottom)
                    / 2.0
            }
            VerticalTextAlignment::Bottom => {
                parent.style.fixed_size.y
                    - text_layout.height().px().ceil()
                    - parent.style.padding().top
                    - parent.style.padding().bottom
            }
        };
    }

    let primary_axis = parent.style.layout_direction().primary_axis();
    let cross_axis = parent.style.layout_direction().cross_axis();
    let parent_persistent_state = parent.uid.and_then(|uid| data.persistent_state.get(uid));

    let parent_primary_position = parent.position.along_axis(primary_axis);
    let parent_cross_position = parent.position.along_axis(cross_axis);
    let parent_primary_size = parent.style.fixed_size.along_axis(primary_axis);
    let parent_cross_size = parent.style.fixed_size.along_axis(cross_axis);
    let parent_primary_padding = parent.style.padding().along_axis(primary_axis);
    let parent_cross_padding = parent.style.padding().along_axis(cross_axis);
    let parent_primary_scroll =
        scroll_along_axis(parent_persistent_state, primary_axis).unwrap_or_default();
    let parent_cross_scroll =
        scroll_along_axis(parent_persistent_state, cross_axis).unwrap_or_default();

    let mut total_primary_node_size = 0.px();
    iter_children!(descendants => |node| {
        total_primary_node_size += node.style.fixed_size.along_axis(primary_axis);
        total_primary_node_size += parent.style.child_spacing();
    });
    total_primary_node_size = (total_primary_node_size - parent.style.child_spacing()).max(0.px());

    let mut primary_offset = match parent.style.child_alignment() {
        Alignment::Start => 0.px(),
        Alignment::Center => {
            (parent_primary_size - total_primary_node_size) / 2.0 - parent_primary_padding[0]
        }
        Alignment::End => {
            parent_primary_size
                - total_primary_node_size
                - parent_primary_padding[0]
                - parent_primary_padding[1]
        }
    };
    primary_offset = primary_offset.max(0.px());

    iter_subtrees!(descendants => |mut subtree| {
        let TreeRef { parent: node, is_root, .. } = subtree.reborrow_mut();

        if is_root {
            node.position = if let Some(uid) = node.uid
                && let Some(&float_pos) = data.float_positions.get(uid)
            {
                // FIXME: if the position has the node end up outside the window bounds, fall back to a different position
                match float_pos {
                    PersistentFloatPosition::Cursor { x, y, .. }
                    | PersistentFloatPosition::CursorFixed { x, y, .. }
                    | PersistentFloatPosition::Fixed { x, y, .. } => {
                        Vec2 { x, y }
                    }
                    PersistentFloatPosition::Popup { x, y, .. } => {
                        Vec2 {
                            x: match x {
                                PopupPosition::BeforeParent => parent.position.x - node.style.fixed_size.x,
                                PopupPosition::ParentStart => parent.position.x,
                                PopupPosition::ParentEnd => parent.position.x + parent.style.fixed_size.x - node.style.fixed_size.x,
                                PopupPosition::AfterParent => parent.position.x + parent.style.fixed_size.x,
                            },
                            y: match y {
                                PopupPosition::BeforeParent => parent.position.y - node.style.fixed_size.y,
                                PopupPosition::ParentStart => parent.position.y,
                                PopupPosition::ParentEnd => parent.position.y + parent.style.fixed_size.y - node.style.fixed_size.y,
                                PopupPosition::AfterParent => parent.position.y + parent.style.fixed_size.y,
                            },
                        }
                    }
                }
            } else {
                Vec2::ZERO
            };
        } else {
            // primary axis
            *node.position.along_axis_mut(primary_axis) =
                parent_primary_position + parent_primary_padding[0] + primary_offset - parent_primary_scroll;

            primary_offset += node.style.fixed_size.along_axis(primary_axis);
            primary_offset += parent.style.child_spacing();

            // cross axis
            *node.position.along_axis_mut(cross_axis) = match node.style.cross_axis_alignment() {
                Alignment::Start => parent_cross_position + parent_cross_padding[0],
                Alignment::Center => {
                    parent_cross_position
                        + (parent_cross_size - node.style.fixed_size.along_axis(cross_axis)) / 2.0
                }
                Alignment::End => {
                    parent_cross_position + parent_cross_size
                        - node.style.fixed_size.along_axis(cross_axis)
                        - parent_cross_padding[1]
                }
            } - parent_cross_scroll;
        }

        position_children(subtree, data);
    });
}

impl<Renderer: rendering::Renderer> ByorGui<Renderer> {
    pub(crate) fn layout(&mut self) {
        if let Some(mut tree) = self.forest.primary_mut() {
            compute_node_size(tree.reborrow_mut(), &mut self.data, Axis::X);
            grow_or_shrink_children(tree.reborrow_mut(), &mut self.data, Axis::X);
            compute_node_size(tree.reborrow_mut(), &mut self.data, Axis::Y);
            grow_or_shrink_children(tree.reborrow_mut(), &mut self.data, Axis::Y);
            position_children(tree.reborrow_mut(), &mut self.data);
        }
    }
}
