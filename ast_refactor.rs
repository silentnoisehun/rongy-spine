// ast_refactor.rs — AST-level Rust code transformation
// Rewrites function signatures and converts .unwrap() → ? using syn
// Cascade refactor: if fn A becomes Result, all callers are updated too
//
// Author: Máté Róbert (silentnoisehun)
// License: MIT

use syn::{self, parse_file, ItemFn, ReturnType, Item};
use quote::{quote, ToTokens};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct RefactorReport {
    pub functions_found: usize,
    pub functions_refactored: usize,
    pub unwraps_converted: usize,
}

pub struct AstRefactor;

impl AstRefactor {
    /// Refactor a file: convert unwrap-using functions to return Result
    pub fn refactor_file(code: &str) -> Result<(String, RefactorReport), String> {
        let ast = parse_file(code).map_err(|e| format!("Parse: {}", e))?;

        let mut report = RefactorReport {
            functions_found: 0,
            functions_refactored: 0,
            unwraps_converted: 0,
        };

        let mut targets: Vec<FnTarget> = Vec::new();

        for item in &ast.items {
            if let Item::Fn(func) = item {
                let fn_name = func.sig.ident.to_string();
                let fn_code = func.to_token_stream().to_string();
                let unwrap_count = fn_code.matches(".unwrap()").count();
                report.functions_found += 1;

                if fn_name == "main" || fn_name.starts_with("test_") { continue; }
                let return_str = match &func.sig.output {
                    ReturnType::Type(_, ty) => quote!(#ty).to_string(),
                    ReturnType::Default => "()".to_string(),
                };
                if return_str.contains("Result") { continue; }
                if unwrap_count == 0 { continue; }

                targets.push(FnTarget {
                    name: fn_name,
                    original_return: return_str,
                    unwrap_count,
                });
            }
        }

        if targets.is_empty() {
            return Ok((code.to_string(), report));
        }

        let mut result_lines: Vec<String> = code.lines().map(|l| l.to_string()).collect();

        for target in &targets {
            let mut in_target_fn = false;
            let mut brace_depth = 0i32;
            let mut fn_start = 0;
            let mut fn_end = 0;

            for (i, line) in result_lines.iter().enumerate() {
                let trimmed = line.trim();
                if !in_target_fn {
                    if (trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ")
                        || trimmed.starts_with("pub async fn ") || trimmed.starts_with("async fn "))
                        && trimmed.contains(&format!("{}(", target.name))
                    {
                        in_target_fn = true;
                        fn_start = i;
                        brace_depth = 0;
                    }
                }
                if in_target_fn {
                    brace_depth += trimmed.matches('{').count() as i32;
                    brace_depth -= trimmed.matches('}').count() as i32;
                    if brace_depth == 0 && i > fn_start {
                        fn_end = i;
                        break;
                    }
                }
            }

            if fn_end == 0 || fn_start == fn_end { continue; }

            let sig_line = &result_lines[fn_start];
            if sig_line.contains("->") && !sig_line.contains("Result") {
                let new_sig = if target.original_return == "()" {
                    sig_line.replace(
                        &format!("-> {}", target.original_return),
                        "-> Result<(), Box<dyn std::error::Error>>"
                    )
                } else {
                    sig_line.replace(
                        &format!("-> {}", target.original_return),
                        &format!("-> Result<{}, Box<dyn std::error::Error>>", target.original_return)
                    )
                };
                if new_sig != *sig_line {
                    result_lines[fn_start] = new_sig;
                    report.functions_refactored += 1;
                }
            } else if !sig_line.contains("->") {
                let new_sig = sig_line.replace("{", "-> Result<(), Box<dyn std::error::Error>> {");
                result_lines[fn_start] = new_sig;
                report.functions_refactored += 1;
            }

            for i in (fn_start + 1)..fn_end {
                let line = &result_lines[i];
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.contains("contains(\"")
                    || trimmed.contains("replace(\"") || trimmed.contains("format!(")
                    || trimmed.contains("println!(") { continue; }

                if trimmed.contains(".unwrap()") {
                    result_lines[i] = line.replace(".unwrap()", "?");
                    report.unwraps_converted += 1;
                }
            }

            if fn_end > fn_start + 1 {
                let last_expr_idx = fn_end - 1;
                let last_line = result_lines[last_expr_idx].trim().to_string();
                if !last_line.starts_with("Ok(") && !last_line.starts_with("Err(")
                    && !last_line.starts_with("}") && !last_line.starts_with("//")
                    && !last_line.is_empty() && !last_line.starts_with("return")
                {
                    let indent = result_lines[last_expr_idx].len()
                        - result_lines[last_expr_idx].trim_start().len();
                    let pad = " ".repeat(indent);
                    if last_line.ends_with(';') {
                        result_lines.insert(fn_end, format!("{}Ok(())", pad));
                    } else {
                        result_lines[last_expr_idx] = format!("{}Ok({})", pad, last_line);
                    }
                }
            }
        }

        let output = result_lines.join("\n");
        if parse_file(&output).is_err() {
            return Err("Refactored code does not parse — aborting".to_string());
        }

        Ok((output, report))
    }

    /// Cascade refactor: if fn A becomes Result, update all callers too
    pub fn cascade_refactor(code: &str) -> Result<(String, RefactorReport), String> {
        let ast = parse_file(code).map_err(|e| format!("Parse: {}", e))?;

        let mut functions: Vec<FnInfo> = Vec::new();
        let mut call_graph: Vec<(String, String)> = Vec::new();

        for item in &ast.items {
            if let Item::Fn(func) = item {
                let name = func.sig.ident.to_string();
                let mut counter = UnwrapCounter::default();
                syn::visit::visit_item_fn(&mut counter, func);

                let return_str = match &func.sig.output {
                    ReturnType::Type(_, ty) => quote!(#ty).to_string().replace(" ", ""),
                    ReturnType::Default => "()".to_string(),
                };

                let body_tokens = quote!(#func).to_string();
                for other_item in &ast.items {
                    if let Item::Fn(other_func) = other_item {
                        let other_name = other_func.sig.ident.to_string();
                        if other_name != name && body_tokens.contains(&other_name) {
                            call_graph.push((name.clone(), other_name));
                        }
                    }
                }

                functions.push(FnInfo {
                    name,
                    original_return: return_str.clone(),
                    unwraps: counter.count,
                    has_result: return_str.contains("Result"),
                });
            }
        }

        let mut needs_result: HashSet<String> = functions.iter()
            .filter(|f| f.unwraps > 0 && !f.has_result && f.name != "main" && !f.name.starts_with("test_"))
            .map(|f| f.name.clone())
            .collect();

        // Propagate: if callee needs Result, caller does too
        let mut changed = true;
        while changed {
            changed = false;
            let current = needs_result.clone();
            for (caller, callee) in &call_graph {
                if current.contains(callee) && !needs_result.contains(caller)
                    && *caller != "main" && !caller.starts_with("test_")
                {
                    if let Some(ci) = functions.iter().find(|f| f.name == *caller) {
                        if !ci.has_result {
                            needs_result.insert(caller.clone());
                            changed = true;
                        }
                    }
                }
            }
        }

        if needs_result.is_empty() {
            return Ok((code.to_string(), RefactorReport {
                functions_found: functions.len(),
                functions_refactored: 0,
                unwraps_converted: 0,
            }));
        }

        // Apply transformations
        let mut result_lines: Vec<String> = code.lines().map(|l| l.to_string()).collect();
        let mut total_refactored = 0;
        let mut total_unwraps = 0;

        for fn_name in &needs_result {
            let fn_info = match functions.iter().find(|f| &f.name == fn_name) {
                Some(f) => f.clone(),
                None => continue,
            };

            let mut in_fn = false;
            let mut fn_start = 0;
            let mut fn_end = 0;
            let mut depth = 0i32;

            for (i, line) in result_lines.iter().enumerate() {
                let trimmed = line.trim();
                if !in_fn {
                    if (trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ")
                        || trimmed.starts_with("pub async fn ") || trimmed.starts_with("async fn "))
                        && trimmed.contains(&format!("{}(", fn_name))
                    {
                        in_fn = true;
                        fn_start = i;
                        depth = 0;
                    }
                }
                if in_fn {
                    depth += trimmed.matches('{').count() as i32;
                    depth -= trimmed.matches('}').count() as i32;
                    if depth == 0 && i > fn_start {
                        fn_end = i;
                        break;
                    }
                }
            }

            if fn_end == 0 { continue; }

            let sig = &result_lines[fn_start];
            if sig.contains("->") && !sig.contains("Result") {
                result_lines[fn_start] = sig.replace(
                    &format!("-> {}", fn_info.original_return),
                    &format!("-> Result<{}, Box<dyn std::error::Error>>", fn_info.original_return)
                );
                total_refactored += 1;
            } else if !sig.contains("->") && sig.contains("{") {
                result_lines[fn_start] = sig.replace(
                    "{", "-> Result<(), Box<dyn std::error::Error>> {"
                );
                total_refactored += 1;
            }

            for i in (fn_start + 1)..fn_end {
                let current = result_lines[i].clone();
                let trimmed = current.trim();
                if trimmed.starts_with("//") || trimmed.contains("format!(")
                    || trimmed.contains("println!(") { continue; }

                if trimmed.contains(".unwrap()") {
                    result_lines[i] = current.replace(".unwrap()", "?");
                    total_unwraps += 1;
                }
            }

            if fn_end > fn_start + 1 {
                let last_idx = fn_end - 1;
                let last = result_lines[last_idx].trim().to_string();
                if !last.starts_with("Ok(") && !last.starts_with("Err(")
                    && !last.starts_with("}") && !last.is_empty()
                    && !last.starts_with("//") && !last.starts_with("return")
                {
                    let indent = result_lines[last_idx].len() - result_lines[last_idx].trim_start().len();
                    let pad = " ".repeat(indent);
                    if last.ends_with(';') {
                        result_lines.insert(fn_end, format!("{}Ok(())", pad));
                    } else {
                        result_lines[last_idx] = format!("{}Ok({})", pad, last);
                    }
                }
            }
        }

        let output = result_lines.join("\n");
        if parse_file(&output).is_err() {
            return Ok((code.to_string(), RefactorReport {
                functions_found: functions.len(),
                functions_refactored: 0,
                unwraps_converted: 0,
            }));
        }

        Ok((output, RefactorReport {
            functions_found: functions.len(),
            functions_refactored: total_refactored,
            unwraps_converted: total_unwraps,
        }))
    }
}

#[derive(Debug, Clone)]
struct FnTarget {
    name: String,
    original_return: String,
    unwrap_count: usize,
}

#[derive(Debug, Clone)]
struct FnInfo {
    name: String,
    original_return: String,
    unwraps: usize,
    has_result: bool,
}

#[derive(Default)]
struct UnwrapCounter {
    count: usize,
}

impl<'ast> syn::visit::Visit<'ast> for UnwrapCounter {
    fn visit_expr_method_call(&mut self, i: &'ast syn::ExprMethodCall) {
        if i.method == "unwrap" && i.args.is_empty() {
            self.count += 1;
        }
        syn::visit::visit_expr_method_call(self, i);
    }
}
