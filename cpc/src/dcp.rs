use crate::ast::*;
// use std::fmt::Write;

pub struct DcpEmitter {
    output: String,
}

impl DcpEmitter {
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    pub fn emit_module(&mut self, module: &Module) -> String {
        self.output.push_str("// Type definitions for CP\n\n");
        for item in &module.body {
            match item {
                ModuleItem::Decl(decl) => {
                    self.emit_decl(decl);
                }
                ModuleItem::Import(imp) => {
                    self.emit_import(imp);
                }
                _ => {}
            }
        }
        self.output.clone()
    }

    fn emit_import(&mut self, imp: &ImportStmt) {
        self.output.push_str("import { ");
        for (i, spec) in imp.specifiers.iter().enumerate() {
            self.output.push_str(&spec.sym);
            if i < imp.specifiers.len() - 1 {
                self.output.push_str(", ");
            }
        }
        self.output.push_str(&format!(" }} from \"{}\";\n", imp.source));
    }

    fn emit_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Func(f) => {
                if f.ident.sym != "main" {
                    self.output.push_str("export ");
                    self.emit_function(f);
                }
            }
            Decl::Class(c) => {
                self.output.push_str(&format!("export class {} ", c.ident.sym));
                if !c.implements.is_empty() {
                    self.output.push_str("implements ");
                    for (i, imp) in c.implements.iter().enumerate() {
                        self.output.push_str(&imp.sym);
                        if i < c.implements.len() - 1 {
                            self.output.push_str(", ");
                        }
                    }
                    self.output.push(' ');
                }
                self.output.push_str("{\n");
                for field in &c.fields {
                    self.output.push_str(&format!("    {}: {},\n", field.ident.sym, self.format_type(&field.ty)));
                }
                for method in &c.methods {
                    self.output.push_str("    ");
                    self.emit_function(method);
                }
                self.output.push_str("}\n\n");
            }
            Decl::Trait(t) => {
                self.output.push_str(&format!("export trait {} {{\n", t.ident.sym));
                for method in &t.methods {
                    self.output.push_str(&format!("    fn {}(", method.ident.sym));
                    self.emit_params(&method.params);
                    self.output.push_str(")");
                    if let Some(ret) = &method.return_type {
                        self.output.push_str(&format!(": {}", self.format_type(ret)));
                    }
                    self.output.push_str(";\n");
                }
                self.output.push_str("}\n\n");
            }
            Decl::Interface(i) => {
                self.output.push_str(&format!("export interface {} {{\n", i.ident.sym));
                for method in &i.methods {
                    self.output.push_str(&format!("    fn {}(", method.ident.sym));
                    self.emit_params(&method.params);
                    self.output.push_str(")");
                    if let Some(ret) = &method.return_type {
                        self.output.push_str(&format!(": {}", self.format_type(ret)));
                    }
                    self.output.push_str(";\n");
                }
                self.output.push_str("}\n\n");
            }
            Decl::Enum(e) => {
                self.output.push_str(&format!("export enum {} {{\n", e.ident.sym));
                for (i, variant) in e.variants.iter().enumerate() {
                    self.output.push_str(&format!("    {}", variant.ident.sym));
                    if let Some(val) = &variant.value {
                        // In .d.cp we should probably just print the raw text, but formatting expr is hard without codegen.
                        // For MVP, just assume literal numbers.
                        // We will add = 0, = 1 etc if it's a literal int.
                        if let Expr::Lit(crate::ast::Lit::Int(n)) = val {
                            self.output.push_str(&format!(" = {}", n));
                        }
                    }
                    if let Some(fields) = &variant.fields {
                        self.output.push_str(" { ");
                        for (j, field) in fields.iter().enumerate() {
                            self.output.push_str(&format!("{}: {}", field.ident.sym, self.format_type(&field.ty)));
                            if j < fields.len() - 1 {
                                self.output.push_str(", ");
                            }
                        }
                        self.output.push_str(" }");
                    }
                    if i < e.variants.len() - 1 {
                        self.output.push_str(",\n");
                    } else {
                        self.output.push('\n');
                    }
                }
                self.output.push_str("}\n\n");
            }
            Decl::ErrorSet(e) => {
                self.output.push_str(&format!("export error {} {{ ", e.ident.sym));
                for (i, variant) in e.variants.iter().enumerate() {
                    self.output.push_str(&variant.sym);
                    if i < e.variants.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str(" }\n\n");
            }
            _ => {}
        }
    }

    fn emit_function(&mut self, f: &Function) {
        if f.is_async {
            self.output.push_str("async ");
        }
        self.output.push_str(&format!("fn {}(", f.ident.sym));
        self.emit_params(&f.params);
        self.output.push_str(")");
        if let Some(ret) = &f.return_type {
            self.output.push_str(&format!(": {}", self.format_type(ret)));
        }
        self.output.push_str(";\n");
    }

    fn emit_params(&mut self, params: &[Param]) {
        for (i, p) in params.iter().enumerate() {
            if p.is_comptime {
                self.output.push_str("comptime ");
            }
            self.output.push_str(&format!("{}: {}", p.ident.sym, self.format_type(&p.ty)));
            if i < params.len() - 1 {
                self.output.push_str(", ");
            }
        }
    }

    fn format_type(&self, ty: &Type) -> String {
        match ty {
            Type::I32 => "i32".to_string(),
            Type::U32 => "u32".to_string(),
            Type::U64 => "u64".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::USize => "usize".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "string".to_string(),
            Type::Void => "void".to_string(),
            Type::Any => "any".to_string(),
            Type::PID => "PID".to_string(),
            Type::Unknown => "unknown".to_string(),
            Type::Array(inner) => format!("[]{}", self.format_type(inner)),
            Type::Map(k, v) => format!("Map<{}, {}>", self.format_type(k), self.format_type(v)),
            Type::Tuple(elements) => {
                let mut s = String::from("[");
                for (i, el) in elements.iter().enumerate() {
                    s.push_str(&self.format_type(el));
                    if i < elements.len() - 1 {
                        s.push_str(", ");
                    }
                }
                s.push(']');
                s
            }
            Type::Object(fields) => {
                let mut s = String::from("{ ");
                for (i, field) in fields.iter().enumerate() {
                    s.push_str(&format!("{}: {}", field.ident.sym, self.format_type(&field.ty)));
                    if i < fields.len() - 1 {
                        s.push_str(", ");
                    }
                }
                s.push_str(" }");
                s
            }
            Type::Ref(ident) => ident.sym.clone(),
            Type::Fn(params, ret) => {
                let mut s = String::from("(");
                for (i, p) in params.iter().enumerate() {
                    s.push_str(&self.format_type(p));
                    if i < params.len() - 1 {
                        s.push_str(", ");
                    }
                }
                s.push_str(&format!(") => {}", self.format_type(ret)));
                s
            }
            Type::ErrorUnion(inner) => format!("!{}", self.format_type(inner)),
            Type::Optional(inner) => format!("?{}", self.format_type(inner)),
        }
    }
}
