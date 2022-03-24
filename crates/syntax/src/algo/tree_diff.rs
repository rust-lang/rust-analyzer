//! AST diff with the GumTree algorithm.
//! See also: https://dl.acm.org/doi/10.1145/2642937.2642982

use crate::algo::{FxIndexMap, TreeDiff, TreeDiffInsertPos};
use crate::SyntaxElement;
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHasher};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::hash::{Hash, Hasher};

const MIN_HEIGHT: u32 = 2;
const MIN_DICE: f64 = 0.5;

pub(super) fn tree_diff(lhs: SyntaxElement, rhs: SyntaxElement) -> TreeDiff {
    let lhs_ctx = collect_info(lhs.clone());
    let rhs_ctx = collect_info(rhs.clone());

    let lhs = ElementWithContext::new(lhs, &lhs_ctx);
    let rhs = ElementWithContext::new(rhs, &rhs_ctx);
    let map = top_down_matching(lhs.clone(), rhs.clone());

    map.into_tree_diff(&lhs_ctx, &rhs_ctx)
}

fn top_down_matching<'a>(lhs: ElementWithContext<'a>, rhs: ElementWithContext<'a>) -> EditingMap {
    let lhs_ctx = lhs.ctx;
    let mut lhs_list = PriorityQueue::default();
    lhs_list.push(lhs);

    let rhs_ctx = rhs.ctx;
    let mut rhs_list = PriorityQueue::default();
    rhs_list.push(rhs);

    let mut map = EditingMap::default();
    let mut candidates = Vec::<(SyntaxElement, SyntaxElement)>::new();

    while let Some((lhs_height, rhs_height)) = lhs_list.peek_max().zip(rhs_list.peek_max()) {
        if lhs_height.min(rhs_height) <= MIN_HEIGHT {
            break;
        }
        match lhs_height.cmp(&rhs_height) {
            Ordering::Less => {
                for item in rhs_list.pop_max() {
                    rhs_list.open(item);
                }
            }
            Ordering::Greater => {
                for item in lhs_list.pop_max() {
                    lhs_list.open(item);
                }
            }
            Ordering::Equal => {
                let lhs_max = lhs_list.pop_max();
                let mut lhs_marks = vec![true; lhs_max.len()];
                let rhs_max = rhs_list.pop_max();
                let mut rhs_marks = vec![true; rhs_max.len()];

                for (lhs_index, rhs_index) in (0..lhs_max.len()).cartesian_product(0..rhs_max.len())
                {
                    let lhs = &lhs_max[lhs_index];
                    let rhs = &rhs_max[rhs_index];
                    if lhs.is_isomorphic(rhs) {
                        if rhs.ctx.get_isomorphic_size(lhs.hash()) > 1
                            || lhs.ctx.get_isomorphic_size(rhs.hash()) > 1
                        {
                            candidates.push((lhs.elem.clone(), rhs.elem.clone()));
                        } else {
                            map.add_match_recursively(lhs.elem.clone(), rhs.elem.clone());
                        }
                        lhs_marks[lhs_index] = false;
                        rhs_marks[rhs_index] = false;
                    }
                }
                lhs_max
                    .into_iter()
                    .zip(lhs_marks.into_iter())
                    .filter(|(_, mark)| *mark)
                    .for_each(|(item, _)| lhs_list.open(item));
                rhs_max
                    .into_iter()
                    .zip(rhs_marks.into_iter())
                    .filter(|(_, mark)| *mark)
                    .for_each(|(item, _)| rhs_list.open(item));
            }
        }
    }
    let get_dice = |pair: &(SyntaxElement, SyntaxElement)| -> f64 {
        let parent0 = pair.0.parent().map(|node| SyntaxElement::Node(node));
        let parent1 = pair.1.parent().map(|node| SyntaxElement::Node(node));
        if let Some((parent0, parent1)) = parent0.zip(parent1) {
            map.dice_with_child_matched(
                &ElementWithContext::new(parent0, lhs_ctx),
                &ElementWithContext::new(pair.0.clone(), lhs_ctx),
                &ElementWithContext::new(parent1, rhs_ctx),
                &ElementWithContext::new(pair.1.clone(), rhs_ctx),
            )
        } else {
            f64::MIN
        }
    };
    candidates.sort_by(|pair0, pair1| get_dice(pair0).partial_cmp(&get_dice(pair1)).unwrap());
    while !candidates.is_empty() {
        if let Some((lhs, rhs)) = candidates.pop() {
            map.add_match_recursively(lhs.clone(), rhs.clone());
            for i in (0..candidates.len()).rev() {
                let pair = &candidates[i];
                if lhs.eq(&pair.0) || rhs.eq(&pair.1) {
                    candidates.remove(i);
                }
            }
        }
    }
    map
}

struct ElementInfo {
    hash: u64,
    height: u32,
    size: u32,
    syntax_elem: SyntaxElement,
}

#[derive(Default)]
struct ElementContext {
    // Notice: The elements should be order by post order so that
    // we can get the set of descendants of element quickly.
    infos: Vec<ElementInfo>,
    elem_to_indexs: FxHashMap<SyntaxElement, usize>,
    // In a tree, there may be two or more subtrees which hash codes are equal.
    hash_to_indexs: FxHashMap<u64, Vec<usize>>,
}

#[derive(Clone)]
struct ElementWithContext<'a> {
    elem: SyntaxElement,
    ctx: &'a ElementContext,
}

// Height-indexed priority list
#[derive(Default)]
struct PriorityQueue<'a> {
    heap: BinaryHeap<ElementWithContext<'a>>,
}

#[derive(Default)]
struct EditingMap {
    matches: FxHashMap<SyntaxElement, SyntaxElement>,
}

// Collect the tree information used later.
// The tree hashing is based on algorithm described in
// "Syntax tree fingerprinting: a foundation for source code similarity detection" section 4
fn collect_info(elem: SyntaxElement) -> ElementContext {
    let mut ctx = ElementContext::default();
    collect_info_impl(&mut ctx, elem);
    ctx
}

fn collect_info_impl(ctx: &mut ElementContext, elem: SyntaxElement) {
    let info = match &elem {
        SyntaxElement::Node(node) => {
            let mut hasher = FxHasher::default();
            node.kind().hash(&mut hasher);
            let mut size = 0u32;
            let mut height = 0u32;

            for child in node.children_with_tokens() {
                collect_info_impl(ctx, child);
                // The info is pushed by post order, so the last info is the child's info.
                if let Some(child_info) = ctx.infos.last() {
                    size += child_info.size;
                    height = height.max(child_info.height);
                    child_info.hash.hash(&mut hasher);
                }
            }
            size += 1;
            height += 1;
            ElementInfo { hash: hasher.finish(), height, size, syntax_elem: elem.clone() }
        }
        SyntaxElement::Token(token) => {
            let mut hasher = FxHasher::default();
            token.kind().hash(&mut hasher);
            token.text().hash(&mut hasher);
            ElementInfo { hash: hasher.finish(), height: 1, size: 1, syntax_elem: elem.clone() }
        }
    };
    let index = ctx.infos.len();
    ctx.elem_to_indexs.insert(elem, index);
    ctx.hash_to_indexs.entry(info.hash.clone()).or_default().push(index);
    ctx.infos.push(info);
}

impl ElementContext {
    pub fn get_isomorphic_size(&self, hash: u64) -> usize {
        self.hash_to_indexs.get(&hash).map_or(0, |indexs| indexs.len())
    }
}

impl<'a> ElementWithContext<'a> {
    pub fn new(elem: SyntaxElement, ctx: &'a ElementContext) -> Self {
        Self { elem, ctx }
    }
    #[inline]
    pub fn info(&self) -> &ElementInfo {
        let index = self.ctx.elem_to_indexs.get(&self.elem).unwrap();
        &self.ctx.infos[*index]
    }
    #[inline]
    pub fn hash(&self) -> u64 {
        self.info().hash
    }
    #[inline]
    pub fn height(&self) -> u32 {
        self.info().height
    }
    #[inline]
    pub fn size(&self) -> u32 {
        self.info().size
    }
    #[inline]
    pub fn is_isomorphic(&self, other: &Self) -> bool {
        self.hash() == other.hash()
    }
    pub fn descendants(&self) -> &[ElementInfo] {
        let index = self.ctx.elem_to_indexs.get(&self.elem).unwrap();
        let elem_info = &self.ctx.infos[*index];
        // infos are ordered by elements' post order, so we can get the descendants directly.
        &self.ctx.infos[(index + 1 - elem_info.size as usize)..*index]
    }
}

impl<'a> Eq for ElementWithContext<'a> {}

impl<'a> PartialEq<Self> for ElementWithContext<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.elem.eq(&other.elem) && std::ptr::eq(self.ctx, other.ctx)
    }
}

impl<'a> PartialOrd<Self> for ElementWithContext<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.height().partial_cmp(&other.height())
    }
}

impl<'a> Ord for ElementWithContext<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.height().cmp(&other.height())
    }
}

impl<'a> PriorityQueue<'a> {
    #[inline]
    pub fn peek_max(&self) -> Option<u32> {
        self.heap.peek().map(|item| item.height())
    }
    #[inline]
    pub fn push(&mut self, item: ElementWithContext<'a>) {
        self.heap.push(item)
    }
    pub fn pop_max(&mut self) -> Vec<ElementWithContext<'a>> {
        if let Some(max) = self.peek_max() {
            let max = Some(max);
            let mut result = Vec::new();
            while max == self.peek_max() {
                if let Some(item) = self.heap.pop() {
                    result.push(item);
                }
            }
            result
        } else {
            Vec::new()
        }
    }
    pub fn open(&mut self, item: ElementWithContext<'a>) {
        let ElementWithContext { elem, ctx } = item;
        match elem {
            SyntaxElement::Node(node) => {
                for child in node.children_with_tokens() {
                    self.push(ElementWithContext::new(child, ctx));
                }
            }
            SyntaxElement::Token(_) => {}
        }
    }
}

impl EditingMap {
    pub fn dice<'a>(&self, t1: &ElementWithContext<'a>, t2: &ElementWithContext<'a>) -> f64 {
        let (match_num, descendants) = self.get_match_and_descendants(t1, t2);
        (match_num * 2) as f64 / descendants as f64
    }
    // `t1_child` is isomorphic with `t2_child`, but (t1, t2) is not contained in the map.
    // I think it's better to assume this pair is contained in the map than it actually is.
    pub fn dice_with_child_matched<'a>(
        &self,
        t1: &ElementWithContext<'a>,
        t1_child: &ElementWithContext<'a>,
        t2: &ElementWithContext<'a>,
        t2_child: &ElementWithContext<'a>,
    ) -> f64 {
        if !t1_child.is_isomorphic(&t2_child) {
            return self.dice(t1, t2);
        }
        let (mut match_num, descendants) = self.get_match_and_descendants(t1, t2);
        match_num += t1_child.size() as usize;
        (match_num * 2) as f64 / descendants as f64
    }
    fn get_match_and_descendants<'a>(
        &self,
        t1: &ElementWithContext<'a>,
        t2: &ElementWithContext<'a>,
    ) -> (usize, usize) {
        let t1_descendants = t1.descendants();
        let t2_descendants = t2.descendants();

        let mut match_num = 0;
        for info in t1_descendants {
            if self.matches.contains_key(&info.syntax_elem) {
                match_num += 1;
            }
        }
        (match_num, t1_descendants.len() + t2_descendants.len())
    }
    #[inline]
    pub fn add_match(&mut self, t1: SyntaxElement, t2: SyntaxElement) {
        self.matches.insert(t1, t2);
    }
    pub fn add_match_recursively(&mut self, t1: SyntaxElement, t2: SyntaxElement) {
        match (&t1, &t2) {
            (SyntaxElement::Node(t1), SyntaxElement::Node(t2)) => {
                for (t1, t2) in t1.children_with_tokens().zip(t2.children_with_tokens()) {
                    self.add_match_recursively(t1, t2);
                }
            }
            (_, _) => {}
        }
        self.add_match(t1, t2);
    }
    pub fn into_tree_diff(&self, lhs_ctx: &ElementContext, rhs_ctx: &ElementContext) -> TreeDiff {
        let mut diff = TreeDiff {
            replacements: Default::default(),
            deletions: vec![],
            insertions: FxIndexMap::default(),
        };
        let mut lhs_marks = vec![true; lhs_ctx.infos.len()];
        let mut rhs_marks = vec![true; rhs_ctx.infos.len()];

        for (lhs, rhs) in self.matches.iter() {
            if let Some((lhs_index, rhs_index)) =
                lhs_ctx.elem_to_indexs.get(lhs).zip(rhs_ctx.elem_to_indexs.get(rhs))
            {
                lhs_marks[*lhs_index] = false;
                rhs_marks[*rhs_index] = false;
            }
        }
        for (index, info) in lhs_ctx.infos.iter().enumerate() {
            if lhs_marks[index] {
                diff.deletions.push(info.syntax_elem.clone());
            }
        }
        // TODO: create insertion edit script
        for (index, info) in rhs_ctx.infos.iter().enumerate() {
            if rhs_marks[index] {}
        }
        diff
    }
}
