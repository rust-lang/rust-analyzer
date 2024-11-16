//! Wrappers over [`make`] constructors
use itertools::Itertools;

use crate::{
    ast::{self, make, HasArgList, HasGenericArgs, HasName, HasTypeBounds},
    syntax_editor::SyntaxMappingBuilder,
    AstNode, NodeOrToken, SyntaxKind, SyntaxNode, SyntaxToken,
};

use super::SyntaxFactory;

impl SyntaxFactory {
    pub fn name(&self, name: &str) -> ast::Name {
        make::name(name).clone_for_update()
    }

    pub fn name_ref(&self, name: &str) -> ast::NameRef {
        make::name_ref(name).clone_for_update()
    }

    pub fn lifetime(&self, text: &str) -> ast::Lifetime {
        make::lifetime(text).clone_for_update()
    }

    pub fn ty(&self, text: &str) -> ast::Type {
        make::ty(text).clone_for_update()
    }

    pub fn ty_placeholder(&self) -> ast::Type {
        make::ty_placeholder().clone_for_update()
    }

    pub fn ty_path(&self, path: ast::Path) -> ast::PathType {
        let ast::Type::PathType(ast) = make::ty_path(path.clone()).clone_for_update() else {
            unreachable!()
        };

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_node(path.syntax().clone(), ast.path().unwrap().syntax().clone());
            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn type_param(
        &self,
        name: ast::Name,
        bounds: Option<ast::TypeBoundList>,
    ) -> ast::TypeParam {
        let ast = make::type_param(name.clone(), bounds.clone()).clone_for_update();

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_node(name.syntax().clone(), ast.name().unwrap().syntax().clone());
            if let Some(input) = bounds {
                builder.map_node(
                    input.syntax().clone(),
                    ast.type_bound_list().unwrap().syntax().clone(),
                );
            }
            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn path_segment(&self, name_ref: ast::NameRef) -> ast::PathSegment {
        let ast = make::path_segment(name_ref.clone()).clone_for_update();

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_node(name_ref.syntax().clone(), ast.name_ref().unwrap().syntax().clone());
            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn path_segment_generics(
        &self,
        name_ref: ast::NameRef,
        generics: ast::GenericArgList,
    ) -> ast::PathSegment {
        let ast::Type::PathType(path) = make::ty(&format!("{name_ref}{generics}")) else {
            unreachable!();
        };

        let ast = path.path().unwrap().segment().unwrap().clone_for_update();

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_node(name_ref.syntax().clone(), ast.name_ref().unwrap().syntax().clone());
            builder.map_node(
                generics.syntax().clone(),
                ast.generic_arg_list().unwrap().syntax().clone(),
            );
            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn path_unqualified(&self, segment: ast::PathSegment) -> ast::Path {
        let ast = make::path_unqualified(segment.clone()).clone_for_update();

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_node(segment.syntax().clone(), ast.segment().unwrap().syntax().clone());
            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn path_from_segments(
        &self,
        segments: impl IntoIterator<Item = ast::PathSegment>,
        is_abs: bool,
    ) -> ast::Path {
        let segments = segments.into_iter().collect_vec();
        let input = segments.iter().map(|it| it.syntax().clone()).collect_vec();
        let ast = make::path_from_segments(segments, is_abs).clone_for_update();

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_children(input.into_iter(), ast.segments().map(|it| it.syntax().clone()));
            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn ident_pat(&self, ref_: bool, mut_: bool, name: ast::Name) -> ast::IdentPat {
        let ast = make::ident_pat(ref_, mut_, name.clone()).clone_for_update();

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_node(name.syntax().clone(), ast.name().unwrap().syntax().clone());
            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn block_expr(
        &self,
        stmts: impl IntoIterator<Item = ast::Stmt>,
        tail_expr: Option<ast::Expr>,
    ) -> ast::BlockExpr {
        let stmts = stmts.into_iter().collect_vec();
        let input = stmts.iter().map(|it| it.syntax().clone()).collect_vec();

        let ast = make::block_expr(stmts, tail_expr.clone()).clone_for_update();

        if let Some((mut mapping, stmt_list)) = self.mappings().zip(ast.stmt_list()) {
            let mut builder = SyntaxMappingBuilder::new(stmt_list.syntax().clone());

            builder.map_children(
                input.into_iter(),
                stmt_list.statements().map(|it| it.syntax().clone()),
            );

            if let Some((input, output)) = tail_expr.zip(stmt_list.tail_expr()) {
                builder.map_node(input.syntax().clone(), output.syntax().clone());
            }

            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn expr_bin(&self, lhs: ast::Expr, op: ast::BinaryOp, rhs: ast::Expr) -> ast::BinExpr {
        let ast::Expr::BinExpr(ast) =
            make::expr_bin_op(lhs.clone(), op, rhs.clone()).clone_for_update()
        else {
            unreachable!()
        };

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_node(lhs.syntax().clone(), ast.lhs().unwrap().syntax().clone());
            builder.map_node(rhs.syntax().clone(), ast.rhs().unwrap().syntax().clone());
            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn expr_path(&self, path: ast::Path) -> ast::Expr {
        let ast::Expr::PathExpr(ast) = make::expr_path(path.clone()).clone_for_update() else {
            unreachable!()
        };

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_node(path.syntax().clone(), ast.path().unwrap().syntax().clone());
            builder.finish(&mut mapping);
        }

        ast.into()
    }

    pub fn expr_call(&self, expr: ast::Expr, arg_list: ast::ArgList) -> ast::CallExpr {
        // FIXME: `make::expr_call`` should return a `CallExpr`, not just an `Expr`
        let ast::Expr::CallExpr(ast) =
            make::expr_call(expr.clone(), arg_list.clone()).clone_for_update()
        else {
            unreachable!()
        };

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_node(expr.syntax().clone(), ast.expr().unwrap().syntax().clone());
            builder.map_node(arg_list.syntax().clone(), ast.arg_list().unwrap().syntax().clone());
            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn arg_list(&self, args: impl IntoIterator<Item = ast::Expr>) -> ast::ArgList {
        let args: Vec<ast::Expr> = args.into_iter().collect();
        let input: Vec<ast::SyntaxNode> = args.iter().map(|it| it.syntax().clone()).collect();
        let ast = make::arg_list(args).clone_for_update();

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax.clone());
            builder.map_children(input.into_iter(), ast.args().map(|it| it.syntax().clone()));
            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn expr_ref(&self, expr: ast::Expr, exclusive: bool) -> ast::Expr {
        let ast::Expr::RefExpr(ast) = make::expr_ref(expr.clone(), exclusive).clone_for_update()
        else {
            unreachable!()
        };

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_node(expr.syntax().clone(), ast.expr().unwrap().syntax().clone());
            builder.finish(&mut mapping);
        }

        ast.into()
    }

    pub fn let_stmt(
        &self,
        pattern: ast::Pat,
        ty: Option<ast::Type>,
        initializer: Option<ast::Expr>,
    ) -> ast::LetStmt {
        let ast =
            make::let_stmt(pattern.clone(), ty.clone(), initializer.clone()).clone_for_update();

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_node(pattern.syntax().clone(), ast.pat().unwrap().syntax().clone());
            if let Some(input) = ty {
                builder.map_node(input.syntax().clone(), ast.ty().unwrap().syntax().clone());
            }
            if let Some(input) = initializer {
                builder
                    .map_node(input.syntax().clone(), ast.initializer().unwrap().syntax().clone());
            }
            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn type_arg(&self, ty: ast::Type) -> ast::TypeArg {
        let ast = make::type_arg(ty.clone()).clone_for_update();

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_node(ty.syntax().clone(), ast.ty().unwrap().syntax().clone());
            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn lifetime_arg(&self, lifetime: ast::Lifetime) -> ast::LifetimeArg {
        let ast = make::lifetime_arg(lifetime.clone()).clone_for_update();

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_node(lifetime.syntax().clone(), ast.lifetime().unwrap().syntax().clone());
            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn generic_arg_list(
        &self,
        args: impl IntoIterator<Item = ast::GenericArg>,
        is_turbo: bool,
    ) -> ast::GenericArgList {
        let args: Vec<ast::GenericArg> = args.into_iter().collect();
        let input: Vec<ast::SyntaxNode> = args.iter().map(|it| it.syntax().clone()).collect();
        let ast = if is_turbo {
            make::turbofish_generic_arg_list(args).clone_for_update()
        } else {
            make::generic_arg_list(args).clone_for_update()
        };

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder
                .map_children(input.into_iter(), ast.generic_args().map(|it| it.syntax().clone()));
            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn token_tree(
        &self,
        delimiter: SyntaxKind,
        tt: Vec<NodeOrToken<ast::TokenTree, SyntaxToken>>,
    ) -> ast::TokenTree {
        let tt: Vec<_> = tt.into_iter().collect();
        let input: Vec<_> = tt.iter().cloned().filter_map(only_nodes).collect();

        let ast = make::token_tree(delimiter, tt).clone_for_update();

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_children(
                input.into_iter(),
                ast.token_trees_and_tokens().filter_map(only_nodes),
            );
            builder.finish(&mut mapping);
        }

        return ast;

        fn only_nodes(element: NodeOrToken<ast::TokenTree, SyntaxToken>) -> Option<SyntaxNode> {
            element.as_node().map(|it| it.syntax().clone())
        }
    }

    pub fn token(&self, kind: SyntaxKind) -> SyntaxToken {
        make::token(kind)
    }
}

// `ext` constructors
impl SyntaxFactory {
    pub fn ident_path(&self, ident: &str) -> ast::Path {
        self.path_unqualified(self.path_segment(self.name_ref(ident)))
    }

    pub fn ty_option(&self, t: ast::Type) -> ast::PathType {
        let generic_arg_list = self.generic_arg_list([self.type_arg(t).into()], false);
        let path = self.path_unqualified(
            self.path_segment_generics(self.name_ref("Option"), generic_arg_list),
        );

        self.ty_path(path)
    }

    pub fn ty_result(&self, t: ast::Type, e: ast::Type) -> ast::PathType {
        let generic_arg_list =
            self.generic_arg_list([self.type_arg(t).into(), self.type_arg(e).into()], false);
        let path = self.path_unqualified(
            self.path_segment_generics(self.name_ref("Result"), generic_arg_list),
        );

        self.ty_path(path)
    }
}
