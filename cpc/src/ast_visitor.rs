use crate::ast::*;
use std::collections::HashSet;

pub fn find_referenced_vars_in_expr(expr: &Expr, vars: &mut HashSet<String>) {
    match expr {
        Expr::Ident(i) => {
            vars.insert(i.sym.clone());
        }
        Expr::Bin(b) => {
            find_referenced_vars_in_expr(&b.left, vars);
            find_referenced_vars_in_expr(&b.right, vars);
        }
        Expr::Call(c) => {
            find_referenced_vars_in_expr(&c.callee, vars);
            for a in &c.args {
                find_referenced_vars_in_expr(a, vars);
            }
        }
        Expr::New(c) => {
            find_referenced_vars_in_expr(&c.callee, vars);
            for a in &c.args {
                find_referenced_vars_in_expr(a, vars);
            }
        }
        Expr::Member(m) | Expr::OptMember(m) => {
            find_referenced_vars_in_expr(&m.obj, vars);
        } // prop is just name
        Expr::OptCall(c) => {
            find_referenced_vars_in_expr(&c.callee, vars);
            for a in &c.args {
                find_referenced_vars_in_expr(a, vars);
            }
        }
        Expr::Slice(s) => {
            find_referenced_vars_in_expr(&s.obj, vars);
            if let Some(st) = &s.start {
                find_referenced_vars_in_expr(st, vars);
            }
            if let Some(e) = &s.end {
                find_referenced_vars_in_expr(e, vars);
            }
        }
        Expr::Match(m) => {
            find_referenced_vars_in_expr(&m.disc, vars);
            for arm in &m.arms {
                if let Some(g) = &arm.guard {
                    find_referenced_vars_in_expr(g, vars);
                }
                find_referenced_vars_in_stmt(&arm.body, vars);
            }
        }
        Expr::Spawn(s) => {
            find_referenced_vars_in_expr(&s.callee, vars);
            for a in &s.args {
                find_referenced_vars_in_expr(a, vars);
            }
        }
        Expr::Question(q) | Expr::ErrWrap(q) | Expr::BoxNew(q) => {
            find_referenced_vars_in_expr(q, vars);
        }
        Expr::QuestionFallback { expr, fallback, .. } => {
            find_referenced_vars_in_expr(expr, vars);
            find_referenced_vars_in_expr(fallback, vars);
        }
        Expr::NullCoalesce { left, right } => {
            find_referenced_vars_in_expr(left, vars);
            find_referenced_vars_in_expr(right, vars);
        }
        Expr::Catch { expr, body, .. } => {
            find_referenced_vars_in_expr(expr, vars);
            find_referenced_vars_in_stmt(body, vars);
        }
        Expr::Downcast { obj, .. } => {
            find_referenced_vars_in_expr(obj, vars);
        }
        Expr::Array(arr) => {
            for a in arr {
                find_referenced_vars_in_expr(a, vars);
            }
        }
        Expr::Object(obj) | Expr::Dict(obj) => {
            for f in obj {
                find_referenced_vars_in_expr(&f.val, vars);
            }
        }
        Expr::Index { obj, index } => {
            find_referenced_vars_in_expr(obj, vars);
            find_referenced_vars_in_expr(index, vars);
        }
        Expr::EnumInit(e) => {
            if let Some(f) = &e.fields {
                for a in f {
                    find_referenced_vars_in_expr(&a.val, vars);
                }
            }
        }
        Expr::Template(t) => {
            for e in &t.expressions {
                find_referenced_vars_in_expr(e, vars);
            }
        }
        Expr::Await(a) => {
            find_referenced_vars_in_expr(a, vars);
        }
        Expr::Move(m) => {
            find_referenced_vars_in_expr(m, vars);
        }
        Expr::Borrow(b, _) => {
            find_referenced_vars_in_expr(b, vars);
        }
        Expr::Builtin(b) => {
            for a in &b.args {
                find_referenced_vars_in_expr(a, vars);
            }
        }
        Expr::Arrow(a) => match &*a.body {
            ArrowBody::Expr(e) => find_referenced_vars_in_expr(e, vars),
            ArrowBody::Block(b) => find_referenced_vars_in_block(b, vars),
        },
        Expr::ReflectType { .. } => {}
        Expr::ReflectOf { expr, .. } => {
            find_referenced_vars_in_expr(expr, vars);
        }
        Expr::Range { start, end, .. } => {
            find_referenced_vars_in_expr(start, vars);
            find_referenced_vars_in_expr(end, vars);
        }
        _ => {}
    }
}

pub fn find_referenced_vars_in_stmt(stmt: &Stmt, vars: &mut HashSet<String>) {
    match stmt {
        Stmt::Expr(e) => find_referenced_vars_in_expr(e, vars),
        Stmt::Var(v) => {
            if let Some(init) = &v.init {
                find_referenced_vars_in_expr(init, vars);
            }
        }
        Stmt::If(i) => {
            find_referenced_vars_in_expr(&i.test, vars);
            find_referenced_vars_in_stmt(&i.cons, vars);
            if let Some(alt) = &i.alt {
                find_referenced_vars_in_stmt(alt, vars);
            }
        }
        Stmt::While(w) => {
            find_referenced_vars_in_expr(&w.test, vars);
            find_referenced_vars_in_stmt(&w.body, vars);
        }
        Stmt::For(f) => {
            find_referenced_vars_in_expr(&f.expr, vars);
            find_referenced_vars_in_stmt(&f.body, vars);
        }
        Stmt::Return(r) => {
            if let Some(arg) = &r.arg {
                find_referenced_vars_in_expr(arg, vars);
            }
        }
        Stmt::Match(m) => {
            find_referenced_vars_in_expr(&m.disc, vars);
            for arm in &m.arms {
                if let Some(g) = &arm.guard {
                    find_referenced_vars_in_expr(g, vars);
                }
                find_referenced_vars_in_stmt(&arm.body, vars);
            }
        }
        Stmt::Spawn(s) => {
            find_referenced_vars_in_expr(&s.callee, vars);
            for a in &s.args {
                find_referenced_vars_in_expr(a, vars);
            }
        }
        Stmt::Defer(d) => find_referenced_vars_in_stmt(d, vars),
        Stmt::ErrDefer(d) => find_referenced_vars_in_stmt(d, vars),
        Stmt::ComptimeBlock(b) => find_referenced_vars_in_block(b, vars),
        Stmt::Block(b) => find_referenced_vars_in_block(b, vars),
        Stmt::TryCatch {
            try_block,
            catch_body,
            ..
        } => {
            find_referenced_vars_in_block(try_block, vars);
            if let Some(cb) = catch_body {
                find_referenced_vars_in_stmt(cb, vars);
            }
        }
    }
}

pub fn find_referenced_vars_in_block(block: &BlockStmt, vars: &mut HashSet<String>) {
    for stmt in &block.body {
        find_referenced_vars_in_stmt(stmt, vars);
    }
}

pub fn get_declared_vars_in_block(block: &BlockStmt) -> HashSet<String> {
    let mut vars = HashSet::new();
    for stmt in &block.body {
        if let Stmt::Var(v) = stmt {
            if let Pattern::Ident(i) = &v.pat {
                vars.insert(i.sym.clone());
            }
        } else if let Stmt::Block(b) = stmt {
            let inner = get_declared_vars_in_block(b);
            vars.extend(inner);
        } else if let Stmt::TryCatch {
            try_block,
            catch_body,
            ..
        } = stmt
        {
            vars.extend(get_declared_vars_in_block(try_block));
            if let Some(cb) = catch_body {
                if let Stmt::Block(b) = &**cb {
                    vars.extend(get_declared_vars_in_block(b));
                }
            }
        } else if let Stmt::If(i) = stmt {
            if let Stmt::Block(b) = &*i.cons {
                vars.extend(get_declared_vars_in_block(b));
            }
            if let Some(alt) = &i.alt {
                if let Stmt::Block(b) = &**alt {
                    vars.extend(get_declared_vars_in_block(b));
                }
            }
        } else if let Stmt::For(f) = stmt {
            extract_idents_from_pat(&f.pat, &mut vars);
            if let Stmt::Block(b) = &*f.body {
                vars.extend(get_declared_vars_in_block(b));
            }
        }
    }
    vars
}

pub fn extract_idents_from_pat(pat: &Pattern, vars: &mut HashSet<String>) {
    match pat {
        Pattern::Ident(i) => {
            vars.insert(i.sym.clone());
        }
        Pattern::Array(pats) | Pattern::Tuple(pats) | Pattern::Or(pats) => {
            for p in pats {
                extract_idents_from_pat(p, vars);
            }
        }
        Pattern::Object(props) => {
            for p in props {
                vars.insert(p.key.sym.clone());
            }
        }
        Pattern::Rest(inner) => {
            extract_idents_from_pat(inner, vars);
        }
        _ => {}
    }
}
