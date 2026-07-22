//! Implementation of applying changes to a syntax tree.

use std::{cmp::Ordering, ops::Range};

use rowan::{TextRange, TextSize};
use rustc_hash::{FxHashMap, FxHashSet};
use stdx::format_to;

use crate::{
    NodeOrToken, SyntaxElement, SyntaxNode,
    syntax_editor::{Change, ChangeKind, PositionRepr, mapping::MissingMapping},
};

use super::{SyntaxEdit, SyntaxEditor};

pub(super) fn apply_edits(editor: SyntaxEditor) -> SyntaxEdit {
    // Algorithm overview:
    //
    // - Sort changes by (range, type)
    //   - Ensures that parent edits are before child edits
    //   - Ensures that inserts will be guaranteed to be inserted at the right range
    // - Validate changes
    //   - Checking for invalid changes is easy since the changes will be sorted by range
    // - Fixup change targets
    //   - standalone change? map to original syntax tree
    //   - dependent change?
    //     - try to map to parent change (either independent or another dependent)
    //     - note: need to keep track of a parent change stack, since a change can be a parent of multiple changes
    // - Apply changes
    //   - find changes to apply to real tree by applying nested changes first
    //   - changed nodes become part of the changed node set (useful for the formatter to only change those parts)
    // - Propagate annotations

    let SyntaxEditor { root, changes, annotations, make } = editor;
    let changes = changes.into_inner();
    let annotations = annotations.into_inner();
    let mappings = make.take();

    let mut node_depths = FxHashMap::<SyntaxNode, usize>::default();
    let mut get_node_depth = |node: SyntaxNode| {
        *node_depths.entry(node).or_insert_with_key(|node| node.ancestors().count())
    };

    // Sort changes by range, then depth, then change kind, so that we can:
    // - ensure that parent edits are ordered before child edits
    // - ensure that inserts will be guaranteed to be inserted at the right range
    // - easily check for disjoint replace ranges
    let mut indexed_changes = changes.into_iter().enumerate().collect::<Vec<_>>();
    indexed_changes.sort_by(|(_, a), (_, b)| {
        a.target_range()
            .start()
            .cmp(&b.target_range().start())
            .then_with(|| {
                let a_target = a.target_parent();
                let b_target = b.target_parent();

                if a_target == b_target {
                    return Ordering::Equal;
                }

                get_node_depth(a_target).cmp(&get_node_depth(b_target))
            })
            .then(a.change_kind().cmp(&b.change_kind()))
    });
    let original_change_indices = indexed_changes.iter().map(|(idx, _)| *idx).collect::<Vec<_>>();
    let mut changes = indexed_changes.into_iter().map(|(_, change)| change).collect::<Vec<_>>();

    let disjoint_replaces_ranges = changes
        .iter()
        .zip(changes.iter().skip(1))
        .filter(|(l, r)| {
            // We only care about checking for disjoint replace ranges
            matches!(
                (l.change_kind(), r.change_kind()),
                (
                    ChangeKind::Replace | ChangeKind::ReplaceRange,
                    ChangeKind::Replace | ChangeKind::ReplaceRange
                )
            )
        })
        .all(|(l, r)| {
            get_node_depth(l.target_parent()) != get_node_depth(r.target_parent())
                || (l.target_range().end() <= r.target_range().start())
        });

    if !disjoint_replaces_ranges {
        report_intersecting_changes(&changes, get_node_depth, &root);

        return SyntaxEdit {
            old_root: root.clone(),
            new_root: root,
            annotations: Default::default(),
            changed_elements: vec![],
        };
    }

    // Build change tree
    let mut changed_ancestors = Vec::<(TextRange, usize)>::new();
    let mut dependent_changes = vec![];
    let mut independent_changes = vec![];
    let mut outdated_changes = vec![];

    for (change_index, change) in changes.iter().enumerate() {
        // Check if this change is dependent on another change (i.e. it's contained within another range)
        if let Some(index) = changed_ancestors
            .iter()
            .rposition(|(range, _)| range.contains_range(change.target_range()))
        {
            // Pop off any ancestors that aren't applicable
            changed_ancestors.drain((index + 1)..);

            // FIXME: Resolve changes that depend on a range of elements
            let (_, ancestor_change_index) = changed_ancestors[index];

            if let Change::Replace(_, None) = changes[ancestor_change_index] {
                outdated_changes.push(change_index);
            } else {
                dependent_changes.push((ancestor_change_index, change_index));
            }
        } else {
            // This change is independent of any other change

            // Drain the changed ancestors since we're no longer in a set of dependent changes
            changed_ancestors.drain(..);

            independent_changes.push(change_index);
        }

        // Add to changed ancestors, if applicable
        match change {
            Change::Replace(SyntaxElement::Node(target), _)
            | Change::ReplaceWithMany(SyntaxElement::Node(target), _) => {
                changed_ancestors.push((target.text_range(), change_index))
            }
            Change::ReplaceAll(range, _) => changed_ancestors.push((
                TextRange::new(range.start().text_range().start(), range.end().text_range().end()),
                change_index,
            )),
            _ => (),
        }
    }

    // Collect changed elements
    let mut changed_element_sources = vec![];

    for index in independent_changes {
        let change_index = original_change_indices[index];
        let target_start = changes[index].target_range().start();
        let tree = changes[index].target_parent().tree_top();
        let mut push_changed = |element: &SyntaxElement| {
            changed_element_sources.push((
                change_index,
                tree.clone(),
                element.clone(),
                target_start,
            ));
        };
        match &changes[index] {
            Change::Insert(_, element) | Change::Replace(_, Some(element)) => {
                push_changed(element);
            }
            Change::InsertAll(_, elements)
            | Change::ReplaceWithMany(_, elements)
            | Change::ReplaceAll(_, elements) => {
                elements.iter().for_each(push_changed);
            }
            Change::Replace(_, None) => {}
        }
    }

    for (parent, child) in dependent_changes.into_iter().rev() {
        let owning_node = |element: &SyntaxElement| match element {
            SyntaxElement::Node(node) => node.clone(),
            SyntaxElement::Token(token) => token.parent().unwrap(),
        };

        let (input_ancestor, output_ancestor) = match &changes[parent] {
            // No change will depend on an insert since changes can only depend on nodes in the root tree
            Change::Insert(_, _) | Change::InsertAll(_, _) => unreachable!(),
            Change::Replace(target, Some(new_target)) => {
                (owning_node(target), owning_node(new_target))
            }
            Change::Replace(_, None) => {
                unreachable!("deletions should not generate dependent changes")
            }
            Change::ReplaceAll(_, _) | Change::ReplaceWithMany(_, _) => {
                unimplemented!("cannot resolve changes that depend on replacing many elements")
            }
        };

        let upmap_target_node = |target: &SyntaxNode| match mappings.upmap_child(
            target,
            &input_ancestor,
            &output_ancestor,
        ) {
            Ok(it) => it,
            Err(MissingMapping(current)) => unreachable!(
                "no mappings exist between {current:?} (ancestor of {input_ancestor:?}) and {output_ancestor:?}"
            ),
        };

        let upmap_target = |target: &SyntaxElement| match mappings.upmap_child_element(
            target,
            &input_ancestor,
            &output_ancestor,
        ) {
            Ok(it) => it,
            Err(MissingMapping(current)) => unreachable!(
                "no mappings exist between {current:?} (ancestor of {input_ancestor:?}) and {output_ancestor:?}"
            ),
        };

        match &mut changes[child] {
            Change::Insert(target, _) | Change::InsertAll(target, _) => match &mut target.repr {
                PositionRepr::FirstChild(parent) => {
                    *parent = upmap_target_node(parent);
                }
                PositionRepr::After(child) => {
                    *child = upmap_target(child);
                }
            },
            Change::Replace(target, _) | Change::ReplaceWithMany(target, _) => {
                *target = upmap_target(target);
            }
            Change::ReplaceAll(range, _) => {
                *range = upmap_target(range.start())..=upmap_target(range.end());
            }
        }
    }

    // We reverse here since we pushed to this in ascending order,
    // and we want to remove elements in descending order
    for idx in outdated_changes.into_iter().rev() {
        changes.remove(idx);
    }

    // Apply changes
    let original_root = root;
    let mut edited_roots = FxHashMap::<SyntaxNode, SyntaxNode>::default();
    let mut shifts = FxHashMap::<SyntaxNode, Vec<(TextSize, i64)>>::default();
    let map_node_to_root = |node: &SyntaxNode, current_root: &SyntaxNode| {
        let original_root = node.tree_top();
        if node == &original_root {
            return current_root.clone();
        }

        let mut path = Vec::new();
        let mut current = node.clone();
        while current != original_root {
            path.push(current.index());
            current = current.parent().unwrap();
        }

        path.into_iter().rev().fold(current_root.clone(), |node, index| {
            node.children_with_tokens().nth(index).and_then(SyntaxElement::into_node).unwrap()
        })
    };
    let map_element_to_root = |element: &SyntaxElement, current_root: &SyntaxNode| match element {
        SyntaxElement::Node(node) => map_node_to_root(node, current_root).into(),
        SyntaxElement::Token(token) => {
            let parent = map_node_to_root(&token.parent().unwrap(), current_root);
            parent.children_with_tokens().nth(token.index()).unwrap()
        }
    };

    let mut group_end = changes.len();
    while group_end > 0 {
        let start = changes[group_end - 1].target_range().start();
        let group_start = changes[..group_end]
            .iter()
            .rposition(|change| change.target_range().start() != start)
            .map_or(0, |idx| idx + 1);

        changes[group_start..group_end].sort_by(|a, b| {
            get_node_depth(b.target_parent())
                .cmp(&get_node_depth(a.target_parent()))
                .then(b.change_kind().cmp(&a.change_kind()))
        });

        for change in &changes[group_start..group_end] {
            let tree = change.target_parent().tree_top();
            let current = edited_roots.get(&tree).unwrap_or(&tree).clone();
            let map_to_edited_root = |element: &SyntaxElement| {
                let element_tree = element.tree_top();
                if element_tree == tree {
                    return element.clone();
                }
                let Some(current_root) = edited_roots.get(&element_tree) else {
                    return element.clone();
                };
                map_element_to_root(element, current_root)
            };
            let element_to_green = |element: &SyntaxElement| match map_to_edited_root(element) {
                SyntaxElement::Node(node) => NodeOrToken::Node(node.green().into_owned()),
                SyntaxElement::Token(token) => NodeOrToken::Token(token.green().to_owned()),
            };
            let old_len = change.target_range().len();
            let new_len = match change {
                Change::Insert(_, element) | Change::Replace(_, Some(element)) => {
                    element_to_green(element).text_len()
                }
                Change::InsertAll(_, elements)
                | Change::ReplaceWithMany(_, elements)
                | Change::ReplaceAll(_, elements) => {
                    elements.iter().fold(TextSize::new(0), |acc, element| {
                        acc + element_to_green(element).text_len()
                    })
                }
                Change::Replace(_, None) => TextSize::new(0),
            };
            let delta = (
                change.target_range().start(),
                i64::from(u32::from(new_len)) - i64::from(u32::from(old_len)),
            );
            let new_root = match change {
                Change::Insert(position, element) => {
                    let (parent, index) = match &position.repr {
                        PositionRepr::FirstChild(parent) => (map_node_to_root(parent, &current), 0),
                        PositionRepr::After(child) => {
                            let child = map_element_to_root(child, &current);
                            (child.parent().unwrap(), child.index() + 1)
                        }
                    };
                    let new_parent =
                        parent.green().splice_children(index..index, [element_to_green(element)]);
                    SyntaxNode::new_root(parent.replace_with(new_parent))
                }
                Change::InsertAll(position, elements) => {
                    let (parent, index) = match &position.repr {
                        PositionRepr::FirstChild(parent) => (map_node_to_root(parent, &current), 0),
                        PositionRepr::After(child) => {
                            let child = map_element_to_root(child, &current);
                            (child.parent().unwrap(), child.index() + 1)
                        }
                    };
                    let elements = elements.iter().map(element_to_green);
                    let new_parent = parent.green().splice_children(index..index, elements);
                    SyntaxNode::new_root(parent.replace_with(new_parent))
                }
                Change::Replace(target, None) => {
                    let target = map_element_to_root(target, &current);
                    let parent = target.parent().unwrap();
                    let new_parent =
                        parent.green().splice_children(target.index()..target.index() + 1, []);
                    SyntaxNode::new_root(parent.replace_with(new_parent))
                }
                Change::Replace(SyntaxElement::Node(target), Some(new_target))
                    if target.parent().is_none() =>
                {
                    let node = match map_to_edited_root(new_target) {
                        SyntaxElement::Node(node) => node,
                        SyntaxElement::Token(_) => panic!("root node replacement should be a node"),
                    };
                    SyntaxNode::new_root(node.green().into_owned())
                }
                Change::Replace(target, Some(new_target)) => {
                    let target = map_element_to_root(target, &current);
                    let parent = target.parent().unwrap();
                    let index = target.index();
                    let new_parent = parent
                        .green()
                        .splice_children(index..index + 1, [element_to_green(new_target)]);
                    SyntaxNode::new_root(parent.replace_with(new_parent))
                }
                Change::ReplaceWithMany(target, elements) => {
                    let target = map_element_to_root(target, &current);
                    let parent = target.parent().unwrap();
                    let index = target.index();
                    let elements = elements.iter().map(element_to_green);
                    let new_parent = parent.green().splice_children(index..index + 1, elements);
                    SyntaxNode::new_root(parent.replace_with(new_parent))
                }
                Change::ReplaceAll(range, elements) => {
                    let start = map_element_to_root(range.start(), &current);
                    let end = map_element_to_root(range.end(), &current);
                    let parent = start.parent().unwrap();
                    let index = start.index();
                    let elements = elements.iter().map(element_to_green);
                    let new_parent =
                        parent.green().splice_children(index..end.index() + 1, elements);
                    SyntaxNode::new_root(parent.replace_with(new_parent))
                }
            };
            edited_roots.insert(tree.clone(), new_root);
            if delta.1 != 0 {
                shifts.entry(tree).or_default().push(delta);
            }
        }

        group_end = group_start;
    }

    let root = if changes.is_empty() {
        original_root.clone()
    } else {
        edited_roots
            .remove(&original_root)
            .expect("a non-empty edit should update the original root")
    };
    let root_elements = root.descendants_with_tokens().collect::<Vec<_>>();

    let mut used_changed_elements = FxHashSet::default();
    let changed_element_mappings = changed_element_sources
        .into_iter()
        .filter_map(|(change, tree, element, target_start)| {
            let source = element;
            let is_match = |candidate: &SyntaxElement| {
                !used_changed_elements.contains(candidate)
                    && candidate.kind() == source.kind()
                    && candidate.text_range().len() == source.text_range().len()
                    && candidate.to_string() == source.to_string()
            };
            let shift = shifts
                .get(&tree)
                .map_or(&[][..], Vec::as_slice)
                .iter()
                .filter(|(offset, _)| *offset < target_start)
                .map(|(_, delta)| *delta)
                .sum::<i64>();
            let target_start = u32::try_from(i64::from(u32::from(target_start)) + shift)
                .map_or(TextSize::new(0), TextSize::new);
            let mapped = root_elements
                .iter()
                .filter(|candidate| is_match(candidate))
                .min_by_key(|candidate| {
                    u32::from(candidate.text_range().start()).abs_diff(u32::from(target_start))
                })
                .cloned()
                .or_else(|| root_elements.iter().find(|candidate| is_match(candidate)).cloned())?;
            used_changed_elements.insert(mapped.clone());
            Some((change, source, mapped))
        })
        .collect::<Vec<_>>();
    let changed_elements =
        changed_element_mappings.iter().map(|(_, _, element)| element.clone()).collect::<Vec<_>>();

    // Propagate annotations
    let is_inside = |element: &SyntaxElement, ancestors: &[SyntaxElement]| {
        ancestors.iter().any(|ancestor| {
            element == ancestor
                || match ancestor {
                    SyntaxElement::Node(ancestor) => {
                        element.ancestors().any(|node| &node == ancestor)
                    }
                    SyntaxElement::Token(_) => false,
                }
        })
    };
    let mut used_annotation_elements = FxHashSet::default();
    let equivalent_element_in_root =
        |element: &SyntaxElement, used: &mut FxHashSet<SyntaxElement>| {
            let is_match = |candidate: &SyntaxElement| {
                !used.contains(candidate)
                    && candidate.kind() == element.kind()
                    && candidate.text_range().len() == element.text_range().len()
                    && candidate.to_string() == element.to_string()
            };
            let element = root_elements
                .iter()
                .find(|candidate| is_match(candidate) && is_inside(candidate, &changed_elements))
                .cloned()
                .or_else(|| root_elements.iter().find(|candidate| is_match(candidate)).cloned())?;
            used.insert(element.clone());
            Some(element)
        };
    let map_descendant =
        |element: &SyntaxElement, source: &SyntaxElement, target: &SyntaxElement| {
            if element == source {
                return Some(target.clone());
            }

            let source = source.as_node()?;
            let mut path = Vec::new();
            match element {
                SyntaxElement::Node(node) => {
                    let mut current = node.clone();
                    while current != *source {
                        path.push((current.index(), true));
                        current = current.parent()?;
                    }
                }
                SyntaxElement::Token(token) => {
                    path.push((token.index(), false));
                    let mut current = token.parent()?;
                    while current != *source {
                        path.push((current.index(), true));
                        current = current.parent()?;
                    }
                }
            }

            let mut current = target.clone();
            for (index, is_node) in path.into_iter().rev() {
                let node = current.into_node()?;
                current = node.children_with_tokens().nth(index)?;
                if is_node && current.as_node().is_none() {
                    return None;
                }
            }

            Some(current)
        };
    let mut annotation_groups = FxHashMap::default();
    let mut mapped_annotations = FxHashMap::<
        (Option<usize>, SyntaxElement),
        (super::SyntaxAnnotation, SyntaxElement),
    >::default();

    for annotation in annotations {
        let element = annotation.element;
        let annotation_id = annotation.annotation;
        let annotation_change = annotation.change;
        let mapped = mapped_annotations
            .get(&(annotation_change, element.clone()))
            .filter(|(source_annotation, _)| *source_annotation != annotation_id)
            .map(|(_, mapped)| mapped.clone())
            .or_else(|| {
                changed_element_mappings
                    .iter()
                    .filter(|(change, _, _)| annotation_change.is_none_or(|it| *change == it))
                    .filter_map(|(_, source, target)| map_descendant(&element, source, target))
                    .find(|mapped| !used_annotation_elements.contains(mapped))
            })
            .or_else(|| match mappings.upmap_element(&element, &root) {
                // Needed to follow the new tree to find the resulting element
                Some(Ok(mapped)) if is_inside(&mapped, &changed_elements) => {
                    used_annotation_elements.insert(mapped.clone());
                    Some(mapped)
                }
                Some(Ok(mapped)) => {
                    equivalent_element_in_root(&element, &mut used_annotation_elements).or_else(
                        || {
                            used_annotation_elements.insert(mapped.clone());
                            Some(mapped)
                        },
                    )
                }
                // Element did not need to be mapped
                None => equivalent_element_in_root(&element, &mut used_annotation_elements),
                // Element did not make it to the final tree
                Some(Err(_)) => equivalent_element_in_root(&element, &mut used_annotation_elements),
            });
        let Some(mapped) = mapped else { continue };
        mapped_annotations.insert((annotation_change, element), (annotation_id, mapped.clone()));
        annotation_groups.entry(annotation_id).or_insert(vec![]).push(mapped);
    }

    SyntaxEdit {
        old_root: original_root,
        new_root: root,
        changed_elements,
        annotations: annotation_groups,
    }
}

fn report_intersecting_changes(
    changes: &[Change],
    mut get_node_depth: impl FnMut(rowan::SyntaxNode<crate::RustLanguage>) -> usize,
    root: &rowan::SyntaxNode<crate::RustLanguage>,
) {
    let intersecting_changes = changes
        .iter()
        .zip(changes.iter().skip(1))
        .filter(|(l, r)| {
            // We only care about checking for disjoint replace ranges.
            matches!(
                (l.change_kind(), r.change_kind()),
                (
                    ChangeKind::Replace | ChangeKind::ReplaceRange,
                    ChangeKind::Replace | ChangeKind::ReplaceRange
                )
            )
        })
        .filter(|(l, r)| {
            get_node_depth(l.target_parent()) == get_node_depth(r.target_parent())
                && (l.target_range().end() > r.target_range().start())
        });

    let mut error_msg = String::from("some replace change ranges intersect!\n");

    let parent_str = root.to_string();

    for (l, r) in intersecting_changes {
        let mut highlighted_str = parent_str.clone();
        let l_range = l.target_range();
        let r_range = r.target_range();

        let i_range = l_range.intersect(r_range).unwrap();
        let i_str = format!("\x1b[46m{}", &parent_str[i_range]);

        let pre_range: Range<usize> = l_range.start().into()..i_range.start().into();
        let pre_str = format!("\x1b[44m{}", &parent_str[pre_range]);

        let (highlight_range, highlight_str) = if l_range == r_range {
            format_to!(error_msg, "\x1b[46mleft change:\x1b[0m  {l:?} {l}\n");
            format_to!(error_msg, "\x1b[46mequals\x1b[0m\n");
            format_to!(error_msg, "\x1b[46mright change:\x1b[0m {r:?} {r}\n");
            let i_highlighted = format!("{i_str}\x1b[0m\x1b[K");
            let total_range: Range<usize> = i_range.into();
            (total_range, i_highlighted)
        } else {
            format_to!(error_msg, "\x1b[44mleft change:\x1b[0m  {l:?} {l}\n");
            let range_end = if l_range.contains_range(r_range) {
                format_to!(error_msg, "\x1b[46mcovers\x1b[0m\n");
                format_to!(error_msg, "\x1b[46mright change:\x1b[0m {r:?} {r}\n");
                l_range.end()
            } else {
                format_to!(error_msg, "\x1b[46mintersects\x1b[0m\n");
                format_to!(error_msg, "\x1b[42mright change:\x1b[0m {r:?} {r}\n");
                r_range.end()
            };

            let post_range: Range<usize> = i_range.end().into()..range_end.into();

            let post_str = format!("\x1b[42m{}", &parent_str[post_range]);
            let result = format!("{pre_str}{i_str}{post_str}\x1b[0m\x1b[K");
            let total_range: Range<usize> = l_range.start().into()..range_end.into();
            (total_range, result)
        };
        highlighted_str.replace_range(highlight_range, &highlight_str);

        format_to!(error_msg, "{highlighted_str}\n");
    }

    stdx::always!(false, "{}", error_msg);
}
