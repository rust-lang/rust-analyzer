use ra_syntax::{
    SyntaxKind, SmolStr,
    ast::{self, AstNode, NameOwner, ModuleItemOwner},
};

pub(crate) struct InModulePtr<T> {
    pub(crate) module: u8,
    pub(crate) inner: T,
}


/// Plays the same role as `LocalSyntaxPtr`, is costlier to resolve, but survies
/// most reparses.
pub(crate) struct ItemPtr {
    // Kind of enclosing item.
    pub(crate) kind: SyntaxKind,
    // Name of enclosing item, if any.
    pub(crate) name: SmolStr,
    // Nth item with the specified kind and name.
    pub(crate) index: u8,
}

pub(crate) struct ImportPtr {
    pub(crate) index: u16,
}


impl ItemPtr {
    fn resolve<'a>(&self, items: impl ast::ModuleItemOwner<'a>) -> ast::ModuleItem<'a> {
        items
            .items()
            .filter(|&it| it.syntax().kind() == self.kind && has_name(it, &self.name))
            .nth(self.index as usize)
            .unwrap()
    }
}

fn has_name(item: ast::ModuleItem, name: &SmolStr) -> bool {
    let item_name = match item {
        ast::ModuleItem::StructDef(it) => it.name(),
        ast::ModuleItem::EnumDef(it) => it.name(),
        ast::ModuleItem::FnDef(it) => it.name(),
        ast::ModuleItem::TraitDef(it) => it.name(),
        ast::ModuleItem::TypeDef(it) => it.name(),
        ast::ModuleItem::ConstDef(it) => it.name(),
        ast::ModuleItem::StaticDef(it) => it.name(),
        ast::ModuleItem::Module(it) => it.name(),
        ast::ModuleItem::UseItem(_) => return false,
        ast::ModuleItem::ImplItem(_) => return false,
        ast::ModuleItem::ExternCrateItem(_) => return false,
    };
    match item_name {
        None => false,
        Some(n) => &n.text() == name,
    }
}
