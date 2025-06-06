use std::sync::Arc;

use cairo_lang_filesystem::ids::FileId;
use cairo_lang_filesystem::span::TextWidth;
use cairo_lang_utils::{LookupIntern, define_short_id};

use super::SyntaxNode;
use super::db::SyntaxGroup;
use super::green::GreenNode;
use super::kind::SyntaxKind;
use crate::node::stable_ptr::SyntaxStablePtr;

define_short_id!(GreenId, Arc::<GreenNode>, SyntaxGroup, lookup_intern_green, intern_green);
impl GreenId {
    /// Returns the width of the node of this green id.
    pub fn width(&self, db: &dyn SyntaxGroup) -> TextWidth {
        match &self.lookup_intern(db).details {
            super::green::GreenNodeDetails::Token(text) => TextWidth::from_str(text),
            super::green::GreenNodeDetails::Node { width, .. } => *width,
        }
    }
}

define_short_id!(
    SyntaxStablePtrId,
    SyntaxStablePtr,
    SyntaxGroup,
    lookup_intern_stable_ptr,
    intern_stable_ptr
);
impl SyntaxStablePtrId {
    /// Lookups a syntax node using a stable syntax pointer.
    /// Should only be called on the root from which the stable pointer was generated.
    pub fn lookup(&self, db: &dyn SyntaxGroup) -> SyntaxNode {
        let ptr = self.lookup_intern(db);
        match ptr {
            SyntaxStablePtr::Root(file_id, green) => SyntaxNode::new_root(db, file_id, green),
            SyntaxStablePtr::Child { parent, .. } => {
                let parent = parent.lookup(db);
                for child in parent.get_children(db).iter() {
                    if child.stable_ptr(db) == *self {
                        return *child;
                    }
                }
                unreachable!();
            }
        }
    }
    pub fn file_id(&self, db: &dyn SyntaxGroup) -> FileId {
        let ptr = self.lookup_intern(db);
        match ptr {
            SyntaxStablePtr::Root(file_id, _) => file_id,
            SyntaxStablePtr::Child { parent, .. } => parent.file_id(db),
        }
    }
    /// Returns the stable pointer of the parent of this stable pointer.
    /// Assumes that the parent exists (that is, `self` is not the root). Panics otherwise.
    pub fn parent(&self, db: &dyn SyntaxGroup) -> SyntaxStablePtrId {
        let SyntaxStablePtr::Child { parent, .. } = self.lookup_intern(db) else { panic!() };
        parent
    }
    /// Returns the stable pointer of the `n`th parent of this stable pointer.
    /// n = 0: returns itself.
    /// n = 1: return the parent.
    /// n = 2: return the grand parent.
    /// And so on...
    /// Assumes that the `n`th parent exists. Panics otherwise.
    pub fn nth_parent(&self, db: &dyn SyntaxGroup, n: usize) -> SyntaxStablePtrId {
        let mut ptr = *self;
        for _ in 0..n {
            ptr = ptr.parent(db);
        }
        ptr
    }
    /// Returns the kind of this stable pointer.
    /// Assumes that `self` is not the root. Panics otherwise.
    pub fn kind(&self, db: &dyn SyntaxGroup) -> SyntaxKind {
        let SyntaxStablePtr::Child { kind, .. } = self.lookup_intern(db) else { panic!() };
        kind
    }
}
