use lingo_ast::BinaryOp;
use lingo_ir::*;
use std::collections::HashMap;

pub struct LlvmGenerator {
    target_triple: String,
    string_constants: Vec<String>,
    next_id: usize,
    var_allocas: HashMap<String, String>,
    extern_funcs: HashMap<String, IrExternFunction>,
}

impl LlvmGenerator {
    pub fn new() -> Self {
        let triple = if cfg!(windows) {
            "x86_64-pc-windows-msvc".to_string()
        } else {
            "x86_64-unknown-linux-gnu".to_string()
        };

        Self {
            target_triple: triple,
            string_constants: Vec::new(),
            next_id: 0,
            var_allocas: HashMap::new(),
            extern_funcs: HashMap::new(),
        }
    }

    fn new_temp_alloca(&mut self, out_allocas: &mut String) -> String {
        self.next_id += 1;
        let reg = format!("%temp_{}", self.next_id);
        out_allocas.push_str(&format!("  {reg} = alloca %struct.LingoValue, align 8\n"));
        reg
    }

    fn new_label(&mut self, prefix: &str) -> String {
        self.next_id += 1;
        format!("{}_{}", prefix, self.next_id)
    }

    fn add_string_constant(&mut self, s: &str) -> (String, usize) {
        let idx = self.string_constants.len();
        self.string_constants.push(s.to_string());
        (format!("@.str.{idx}"), s.len() + 1)
    }

    pub fn generate_module(&mut self, module: &IrModule) -> String {
        for ext in &module.extern_functions {
            self.extern_funcs.insert(ext.name.clone(), ext.clone());
        }

        let mut body = String::new();

        for func in &module.functions {
            body.push_str(&self.generate_function(func));
            body.push('\n');
        }

        let mut header = String::new();
        header.push_str("; ModuleID = 'lingo'\n");
        header.push_str(&format!("target triple = \"{}\"\n\n", self.target_triple));

        header.push_str("%struct.LingoValue = type { i64, i64 }\n\n");

        header.push_str("declare void @lingo_val_nil(ptr)\n");
        header.push_str("declare void @lingo_val_bool(ptr, i64)\n");
        header.push_str("declare void @lingo_val_int(ptr, i64)\n");
        header.push_str("declare void @lingo_val_float(ptr, double)\n");
        header.push_str("declare void @lingo_val_str(ptr, ptr, i64)\n");
        header.push_str("declare i64 @lingo_val_is_truthy(ptr)\n");
        header.push_str("declare void @lingo_val_add(ptr, ptr, ptr)\n");
        header.push_str("declare void @lingo_val_sub(ptr, ptr, ptr)\n");
        header.push_str("declare void @lingo_val_mul(ptr, ptr, ptr)\n");
        header.push_str("declare void @lingo_val_div(ptr, ptr, ptr)\n");
        header.push_str("declare void @lingo_val_mod(ptr, ptr, ptr)\n");
        header.push_str("declare void @lingo_val_and(ptr, ptr, ptr)\n");
        header.push_str("declare void @lingo_val_or(ptr, ptr, ptr)\n");
        header.push_str("declare i64 @lingo_val_eq(ptr, ptr)\n");
        header.push_str("declare i64 @lingo_val_neq(ptr, ptr)\n");
        header.push_str("declare i64 @lingo_val_lt(ptr, ptr)\n");
        header.push_str("declare i64 @lingo_val_gt(ptr, ptr)\n");
        header.push_str("declare i64 @lingo_val_lte(ptr, ptr)\n");
        header.push_str("declare i64 @lingo_val_gte(ptr, ptr)\n");
        header.push_str("declare void @lingo_list_new(ptr)\n");
        header.push_str("declare void @lingo_list_push(ptr, ptr)\n");
        header.push_str("declare void @lingo_list_get(ptr, ptr, i64)\n");
        header.push_str("declare void @lingo_map_new(ptr)\n");
        header.push_str("declare void @lingo_map_insert(ptr, ptr, ptr)\n");
        header.push_str("declare void @lingo_val_get(ptr, ptr, ptr)\n");
        header.push_str("declare void @lingo_val_set(ptr, ptr, ptr)\n");
        header.push_str("declare void @lingo_val_len(ptr, ptr)\n");
        header.push_str("declare void @lingo_channel_new(ptr)\n");
        header.push_str("declare void @lingo_channel_send(ptr, ptr)\n");
        header.push_str("declare void @lingo_channel_recv(ptr, ptr)\n");
        header.push_str("declare void @lingo_file_read(ptr, ptr)\n");
        header.push_str("declare i64 @lingo_file_write(ptr, ptr)\n");
        header.push_str("declare i64 @lingo_file_exists(ptr)\n");
        header.push_str("declare void @lingo_env_get(ptr, ptr)\n");
        header.push_str("declare void @lingo_math_sqrt(ptr, ptr)\n");
        header.push_str("declare void @lingo_math_sin(ptr, ptr)\n");
        header.push_str("declare void @lingo_math_cos(ptr, ptr)\n");
        header.push_str("declare void @lingo_math_floor(ptr, ptr)\n");
        header.push_str("declare void @lingo_math_ceil(ptr, ptr)\n");
        header.push_str("declare void @lingo_math_abs(ptr, ptr)\n");
        header.push_str("declare void @lingo_sleep_ms(i64)\n");
        header.push_str("declare i64 @lingo_time_now_ms()\n");
        header.push_str("declare void @lingo_println_val(ptr)\n");
        header.push_str("declare void @lingo_print_val(ptr)\n\n");

        for ext in &module.extern_functions {
            if ext.name == "lingo_time_now_ms" || ext.name == "lingo_sleep_ms" {
                continue;
            }
            let ret_ty = match ext.ret_ty {
                lingo_types::Type::Int => "i64",
                lingo_types::Type::Float => "double",
                lingo_types::Type::Str => "ptr",
                lingo_types::Type::Void => "void",
                _ => "i64",
            };
            let mut sig = format!("declare {ret_ty} @{}(", ext.name);
            for (idx, (_, p_ty)) in ext.params.iter().enumerate() {
                if idx > 0 {
                    sig.push_str(", ");
                }
                let param_ty = match p_ty {
                    lingo_types::Type::Int => "i64",
                    lingo_types::Type::Float => "double",
                    lingo_types::Type::Str => "ptr",
                    _ => "i64",
                };
                sig.push_str(param_ty);
            }
            sig.push_str(")\n");
            header.push_str(&sig);
        }

        for (idx, s) in self.string_constants.iter().enumerate() {
            let escaped: String = s
                .chars()
                .flat_map(|c| match c {
                    '\n' => "\\0A".chars().collect(),
                    '\t' => "\\09".chars().collect(),
                    '\r' => "\\0D".chars().collect(),
                    '\\' => "\\\\".chars().collect(),
                    '"' => "\\22".chars().collect(),
                    _ => vec![c],
                })
                .collect();
            let len = s.len() + 1;
            header.push_str(&format!(
                "@.str.{idx} = private unnamed_addr constant [{len} x i8] c\"{escaped}\\00\", align 1\n"
            ));
        }
        header.push('\n');

        header.push_str(&body);
        header
    }

    fn generate_function(&mut self, func: &IrFunction) -> String {
        self.var_allocas.clear();
        let is_main = func.name == "main";

        let mut out_allocas = String::new();
        let mut out_body = String::new();

        let mut func_sig = if is_main {
            "define i32 @main() {\nentry:\n".to_string()
        } else {
            let mut s = format!("define void @{}(ptr %out", func.name);
            for (name, _) in &func.params {
                s.push_str(&format!(", ptr %arg_{name}"));
            }
            s.push_str(") {\nentry:\n");
            s
        };

        for (name, _) in &func.params {
            let alloca_reg = format!("%var_{name}");
            out_allocas.push_str(&format!("  {alloca_reg} = alloca %struct.LingoValue, align 8\n"));
            self.next_id += 1;
            let temp_val = format!("%param_copy_{}", self.next_id);
            out_body.push_str(&format!("  {temp_val} = load %struct.LingoValue, ptr %arg_{name}, align 8\n"));
            out_body.push_str(&format!("  store %struct.LingoValue {temp_val}, ptr {alloca_reg}, align 8\n"));
            self.var_allocas.insert(name.clone(), alloca_reg);
        }

        for inst in &func.instructions {
            self.collect_allocas(inst, &mut out_allocas);
        }

        for inst in &func.instructions {
            self.generate_instruction(inst, &mut out_allocas, &mut out_body);
        }

        if is_main {
            out_body.push_str("  ret i32 0\n");
        } else {
            out_body.push_str("  call void @lingo_val_nil(ptr %out)\n");
            out_body.push_str("  ret void\n");
        }

        func_sig.push_str(&out_allocas);
        func_sig.push_str(&out_body);
        func_sig.push_str("}\n");
        func_sig
    }

    fn collect_allocas(&mut self, inst: &IrInstruction, out_allocas: &mut String) {
        match inst {
            IrInstruction::Assign { target, .. } => {
                if !self.var_allocas.contains_key(target) {
                    let alloca_reg = format!("%var_{target}");
                    out_allocas.push_str(&format!("  {alloca_reg} = alloca %struct.LingoValue, align 8\n"));
                    self.var_allocas.insert(target.clone(), alloca_reg);
                }
            }
            IrInstruction::If { then_branch, else_branch, .. } => {
                for i in then_branch {
                    self.collect_allocas(i, out_allocas);
                }
                if let Some(eb) = else_branch {
                    for i in eb {
                        self.collect_allocas(i, out_allocas);
                    }
                }
            }
            IrInstruction::While { cond_prep, body, .. } => {
                for i in cond_prep {
                    self.collect_allocas(i, out_allocas);
                }
                for i in body {
                    self.collect_allocas(i, out_allocas);
                }
            }
            _ => {}
        }
    }

    fn generate_instruction(&mut self, inst: &IrInstruction, out_allocas: &mut String, out_body: &mut String) {
        match inst {
            IrInstruction::Assign { target, value } => {
                let target_reg = self.var_allocas.get(target).cloned().unwrap();
                self.generate_expr_into(value, &target_reg, out_allocas, out_body);
            }
            IrInstruction::Call { target, callee, args } => {
                let mut arg_ptrs = Vec::new();
                for arg in args {
                    arg_ptrs.push(self.load_value_ptr(arg, out_allocas, out_body));
                }

                if callee == "println" || callee == "print" {
                    let fn_name = if callee == "println" { "lingo_println_val" } else { "lingo_print_val" };
                    if let Some(first_ptr) = arg_ptrs.first() {
                        out_body.push_str(&format!("  call void @{fn_name}(ptr {first_ptr})\n"));
                    }
                } else if callee == "Channel" || callee == "channel_new" {
                    let target_reg = if let Some(tgt) = target {
                        if let Some(alloca) = self.var_allocas.get(tgt) {
                            alloca.clone()
                        } else {
                            let alloca = self.new_temp_alloca(out_allocas);
                            self.var_allocas.insert(tgt.clone(), alloca.clone());
                            alloca
                        }
                    } else {
                        self.new_temp_alloca(out_allocas)
                    };
                    out_body.push_str(&format!("  call void @lingo_channel_new(ptr {target_reg})\n"));
                } else if callee == "channel_send" {
                    if arg_ptrs.len() >= 2 {
                        out_body.push_str(&format!("  call void @lingo_channel_send(ptr {}, ptr {})\n", arg_ptrs[0], arg_ptrs[1]));
                    }
                } else if callee == "channel_recv" {
                    let target_reg = if let Some(tgt) = target {
                        if let Some(alloca) = self.var_allocas.get(tgt) {
                            alloca.clone()
                        } else {
                            let alloca = self.new_temp_alloca(out_allocas);
                            self.var_allocas.insert(tgt.clone(), alloca.clone());
                            alloca
                        }
                    } else {
                        self.new_temp_alloca(out_allocas)
                    };
                    if let Some(ch_ptr) = arg_ptrs.first() {
                        out_body.push_str(&format!("  call void @lingo_channel_recv(ptr {target_reg}, ptr {ch_ptr})\n"));
                    }
                } else if callee == "sleep_ms" {
                    if let Some(ms_ptr) = arg_ptrs.first() {
                        self.next_id += 1;
                        let tmp = format!("%sleep_val_{}", self.next_id);
                        out_body.push_str(&format!("  {tmp} = load %struct.LingoValue, ptr {ms_ptr}, align 8\n"));
                        self.next_id += 1;
                        let raw = format!("%sleep_raw_{}", self.next_id);
                        out_body.push_str(&format!("  {raw} = extractvalue %struct.LingoValue {tmp}, 1\n"));
                        out_body.push_str(&format!("  call void @lingo_sleep_ms(i64 {raw})\n"));
                    }
                } else if callee == "time_now_ms" {
                    let target_reg = if let Some(tgt) = target {
                        if let Some(alloca) = self.var_allocas.get(tgt) {
                            alloca.clone()
                        } else {
                            let alloca = self.new_temp_alloca(out_allocas);
                            self.var_allocas.insert(tgt.clone(), alloca.clone());
                            alloca
                        }
                    } else {
                        self.new_temp_alloca(out_allocas)
                    };
                    self.next_id += 1;
                    let time_raw = format!("%time_raw_{}", self.next_id);
                    out_body.push_str(&format!("  {time_raw} = call i64 @lingo_time_now_ms()\n"));
                    out_body.push_str(&format!("  call void @lingo_val_int(ptr {target_reg}, i64 {time_raw})\n"));
                } else if callee == "file_read" {
                    let target_reg = if let Some(tgt) = target {
                        if let Some(alloca) = self.var_allocas.get(tgt) {
                            alloca.clone()
                        } else {
                            let alloca = self.new_temp_alloca(out_allocas);
                            self.var_allocas.insert(tgt.clone(), alloca.clone());
                            alloca
                        }
                    } else {
                        self.new_temp_alloca(out_allocas)
                    };
                    if let Some(p_ptr) = arg_ptrs.first() {
                        out_body.push_str(&format!("  call void @lingo_file_read(ptr {target_reg}, ptr {p_ptr})\n"));
                    }
                } else if callee == "file_write" {
                    let target_reg = if let Some(tgt) = target {
                        if let Some(alloca) = self.var_allocas.get(tgt) {
                            alloca.clone()
                        } else {
                            let alloca = self.new_temp_alloca(out_allocas);
                            self.var_allocas.insert(tgt.clone(), alloca.clone());
                            alloca
                        }
                    } else {
                        self.new_temp_alloca(out_allocas)
                    };
                    if arg_ptrs.len() >= 2 {
                        self.next_id += 1;
                        let ok_raw = format!("%write_ok_{}", self.next_id);
                        out_body.push_str(&format!("  {ok_raw} = call i64 @lingo_file_write(ptr {}, ptr {})\n", arg_ptrs[0], arg_ptrs[1]));
                        out_body.push_str(&format!("  call void @lingo_val_bool(ptr {target_reg}, i64 {ok_raw})\n"));
                    }
                } else if callee == "file_exists" {
                    let target_reg = if let Some(tgt) = target {
                        if let Some(alloca) = self.var_allocas.get(tgt) {
                            alloca.clone()
                        } else {
                            let alloca = self.new_temp_alloca(out_allocas);
                            self.var_allocas.insert(tgt.clone(), alloca.clone());
                            alloca
                        }
                    } else {
                        self.new_temp_alloca(out_allocas)
                    };
                    if let Some(p_ptr) = arg_ptrs.first() {
                        self.next_id += 1;
                        let ex_raw = format!("%exists_raw_{}", self.next_id);
                        out_body.push_str(&format!("  {ex_raw} = call i64 @lingo_file_exists(ptr {p_ptr})\n"));
                        out_body.push_str(&format!("  call void @lingo_val_bool(ptr {target_reg}, i64 {ex_raw})\n"));
                    }
                } else if callee == "env_get" {
                    let target_reg = if let Some(tgt) = target {
                        if let Some(alloca) = self.var_allocas.get(tgt) {
                            alloca.clone()
                        } else {
                            let alloca = self.new_temp_alloca(out_allocas);
                            self.var_allocas.insert(tgt.clone(), alloca.clone());
                            alloca
                        }
                    } else {
                        self.new_temp_alloca(out_allocas)
                    };
                    if let Some(k_ptr) = arg_ptrs.first() {
                        out_body.push_str(&format!("  call void @lingo_env_get(ptr {target_reg}, ptr {k_ptr})\n"));
                    }
                } else if callee == "math_sqrt" {
                    let target_reg = if let Some(tgt) = target {
                        if let Some(alloca) = self.var_allocas.get(tgt) {
                            alloca.clone()
                        } else {
                            let alloca = self.new_temp_alloca(out_allocas);
                            self.var_allocas.insert(tgt.clone(), alloca.clone());
                            alloca
                        }
                    } else {
                        self.new_temp_alloca(out_allocas)
                    };
                    if let Some(x_ptr) = arg_ptrs.first() {
                        out_body.push_str(&format!("  call void @lingo_math_sqrt(ptr {target_reg}, ptr {x_ptr})\n"));
                    }
                } else if callee == "len" {
                    let target_reg = if let Some(tgt) = target {
                        if let Some(alloca) = self.var_allocas.get(tgt) {
                            alloca.clone()
                        } else {
                            let alloca = self.new_temp_alloca(out_allocas);
                            self.var_allocas.insert(tgt.clone(), alloca.clone());
                            alloca
                        }
                    } else {
                        self.new_temp_alloca(out_allocas)
                    };
                    if let Some(x_ptr) = arg_ptrs.first() {
                        out_body.push_str(&format!("  call void @lingo_val_len(ptr {target_reg}, ptr {x_ptr})\n"));
                    }
                } else if callee == "math_sin" || callee == "math_cos" || callee == "math_floor" || callee == "math_ceil" || callee == "math_abs" {
                    let target_reg = if let Some(tgt) = target {
                        if let Some(alloca) = self.var_allocas.get(tgt) {
                            alloca.clone()
                        } else {
                            let alloca = self.new_temp_alloca(out_allocas);
                            self.var_allocas.insert(tgt.clone(), alloca.clone());
                            alloca
                        }
                    } else {
                        self.new_temp_alloca(out_allocas)
                    };
                    if let Some(x_ptr) = arg_ptrs.first() {
                        let fn_name = format!("lingo_{callee}");
                        out_body.push_str(&format!("  call void @{fn_name}(ptr {target_reg}, ptr {x_ptr})\n"));
                    }
                } else if let Some(ext) = self.extern_funcs.get(callee).cloned() {
                    let target_reg = if let Some(tgt) = target {
                        if let Some(alloca) = self.var_allocas.get(tgt) {
                            alloca.clone()
                        } else {
                            let alloca = self.new_temp_alloca(out_allocas);
                            self.var_allocas.insert(tgt.clone(), alloca.clone());
                            alloca
                        }
                    } else {
                        self.new_temp_alloca(out_allocas)
                    };

                    let mut call_args = Vec::new();
                    for (i, arg_ptr) in arg_ptrs.iter().enumerate() {
                        self.next_id += 1;
                        let tmp_val = format!("%ext_tmp_{}", self.next_id);
                        out_body.push_str(&format!("  {tmp_val} = load %struct.LingoValue, ptr {arg_ptr}, align 8\n"));
                        self.next_id += 1;
                        let raw_val = format!("%ext_raw_{}", self.next_id);
                        out_body.push_str(&format!("  {raw_val} = extractvalue %struct.LingoValue {tmp_val}, 1\n"));
                        let p_ty = ext.params.get(i).map(|(_, t)| t).unwrap_or(&lingo_types::Type::Int);
                        let p_llvm_ty = match p_ty {
                            lingo_types::Type::Int => "i64",
                            lingo_types::Type::Float => "double",
                            lingo_types::Type::Str => "ptr",
                            _ => "i64",
                        };
                        call_args.push(format!("{p_llvm_ty} {raw_val}"));
                    }

                    let ret_llvm_ty = match ext.ret_ty {
                        lingo_types::Type::Int => "i64",
                        lingo_types::Type::Float => "double",
                        lingo_types::Type::Str => "ptr",
                        lingo_types::Type::Void => "void",
                        _ => "i64",
                    };

                    self.next_id += 1;
                    let ret_raw = format!("%ext_res_{}", self.next_id);
                    if ret_llvm_ty == "void" {
                        out_body.push_str(&format!("  call void @{callee}({})\n", call_args.join(", ")));
                        out_body.push_str(&format!("  call void @lingo_val_nil(ptr {target_reg})\n"));
                    } else {
                        out_body.push_str(&format!("  {ret_raw} = call {ret_llvm_ty} @{callee}({})\n", call_args.join(", ")));
                        if ret_llvm_ty == "i64" {
                            out_body.push_str(&format!("  call void @lingo_val_int(ptr {target_reg}, i64 {ret_raw})\n"));
                        } else if ret_llvm_ty == "double" {
                            out_body.push_str(&format!("  call void @lingo_val_float(ptr {target_reg}, double {ret_raw})\n"));
                        } else {
                            out_body.push_str(&format!("  call void @lingo_val_int(ptr {target_reg}, i64 {ret_raw})\n"));
                        }
                    }
                } else {
                    let target_reg = if let Some(tgt) = target {
                        if let Some(alloca) = self.var_allocas.get(tgt) {
                            alloca.clone()
                        } else {
                            let alloca = self.new_temp_alloca(out_allocas);
                            self.var_allocas.insert(tgt.clone(), alloca.clone());
                            alloca
                        }
                    } else {
                        self.new_temp_alloca(out_allocas)
                    };

                    let mut call_str = format!("  call void @{callee}(ptr {target_reg}");
                    for arg in arg_ptrs {
                        call_str.push_str(&format!(", ptr {arg}"));
                    }
                    call_str.push_str(")\n");
                    out_body.push_str(&call_str);
                }
            }
            IrInstruction::Return(opt_val) => {
                if let Some(val) = opt_val {
                    let val_ptr = self.load_value_ptr(val, out_allocas, out_body);
                    self.next_id += 1;
                    let tmp = format!("%ret_tmp_{}", self.next_id);
                    out_body.push_str(&format!("  {tmp} = load %struct.LingoValue, ptr {val_ptr}, align 8\n"));
                    out_body.push_str(&format!("  store %struct.LingoValue {tmp}, ptr %out, align 8\n"));
                } else {
                    out_body.push_str("  call void @lingo_val_nil(ptr %out)\n");
                }
                out_body.push_str("  ret void\n");
            }
            IrInstruction::If { condition, then_branch, else_branch } => {
                let cond_ptr = self.load_value_ptr(condition, out_allocas, out_body);
                self.next_id += 1;
                let is_truthy = format!("%truthy_{}", self.next_id);
                out_body.push_str(&format!("  {is_truthy} = call i64 @lingo_val_is_truthy(ptr {cond_ptr})\n"));
                self.next_id += 1;
                let cmp_reg = format!("%cmp_{}", self.next_id);
                out_body.push_str(&format!("  {cmp_reg} = icmp ne i64 {is_truthy}, 0\n"));

                let then_lbl = self.new_label("then");
                let else_lbl = self.new_label("else");
                let merge_lbl = self.new_label("merge");

                let has_else = else_branch.is_some();
                let false_target = if has_else { &else_lbl } else { &merge_lbl };

                out_body.push_str(&format!("  br i1 {cmp_reg}, label %{then_lbl}, label %{false_target}\n\n"));

                out_body.push_str(&format!("{then_lbl}:\n"));
                for i in then_branch {
                    self.generate_instruction(i, out_allocas, out_body);
                }
                out_body.push_str(&format!("  br label %{merge_lbl}\n\n"));

                if let Some(eb) = else_branch {
                    out_body.push_str(&format!("{else_lbl}:\n"));
                    for i in eb {
                        self.generate_instruction(i, out_allocas, out_body);
                    }
                    out_body.push_str(&format!("  br label %{merge_lbl}\n\n"));
                }

                out_body.push_str(&format!("{merge_lbl}:\n"));
            }
            IrInstruction::While { cond_prep, condition, body } => {
                let loop_hdr = self.new_label("loop_hdr");
                let loop_body = self.new_label("loop_body");
                let loop_exit = self.new_label("loop_exit");

                out_body.push_str(&format!("  br label %{loop_hdr}\n\n"));
                out_body.push_str(&format!("{loop_hdr}:\n"));

                for inst in cond_prep {
                    self.generate_instruction(inst, out_allocas, out_body);
                }

                let cond_ptr = self.load_value_ptr(condition, out_allocas, out_body);
                self.next_id += 1;
                let is_truthy = format!("%truthy_{}", self.next_id);
                out_body.push_str(&format!("  {is_truthy} = call i64 @lingo_val_is_truthy(ptr {cond_ptr})\n"));
                self.next_id += 1;
                let cmp_reg = format!("%cmp_{}", self.next_id);
                out_body.push_str(&format!("  {cmp_reg} = icmp ne i64 {is_truthy}, 0\n"));
                out_body.push_str(&format!("  br i1 {cmp_reg}, label %{loop_body}, label %{loop_exit}\n\n"));

                out_body.push_str(&format!("{loop_body}:\n"));
                for i in body {
                    self.generate_instruction(i, out_allocas, out_body);
                }
                out_body.push_str(&format!("  br label %{loop_hdr}\n\n"));

                out_body.push_str(&format!("{loop_exit}:\n"));
            }
            IrInstruction::IndexAssign { target, index, value } => {
                let t_ptr = self.load_value_ptr(target, out_allocas, out_body);
                let i_ptr = self.load_value_ptr(index, out_allocas, out_body);
                let v_ptr = self.load_value_ptr(value, out_allocas, out_body);
                out_body.push_str(&format!("  call void @lingo_val_set(ptr {t_ptr}, ptr {i_ptr}, ptr {v_ptr})\n"));
            }
        }
    }

    fn generate_expr_into(&mut self, expr: &IrExpr, target_ptr: &str, out_allocas: &mut String, out_body: &mut String) {
        match expr {
            IrExpr::Val(val) => {
                self.emit_value_into(val, target_ptr, out_allocas, out_body);
            }
            IrExpr::Binary { op, left, right } => {
                let l_ptr = self.load_value_ptr(left, out_allocas, out_body);
                let r_ptr = self.load_value_ptr(right, out_allocas, out_body);

                match op {
                    BinaryOp::Add => {
                        out_body.push_str(&format!("  call void @lingo_val_add(ptr {target_ptr}, ptr {l_ptr}, ptr {r_ptr})\n"));
                    }
                    BinaryOp::Sub => {
                        out_body.push_str(&format!("  call void @lingo_val_sub(ptr {target_ptr}, ptr {l_ptr}, ptr {r_ptr})\n"));
                    }
                    BinaryOp::Mul => {
                        out_body.push_str(&format!("  call void @lingo_val_mul(ptr {target_ptr}, ptr {l_ptr}, ptr {r_ptr})\n"));
                    }
                    BinaryOp::Div => {
                        out_body.push_str(&format!("  call void @lingo_val_div(ptr {target_ptr}, ptr {l_ptr}, ptr {r_ptr})\n"));
                    }
                    BinaryOp::Mod => {
                        out_body.push_str(&format!("  call void @lingo_val_mod(ptr {target_ptr}, ptr {l_ptr}, ptr {r_ptr})\n"));
                    }
                    BinaryOp::And => {
                        out_body.push_str(&format!("  call void @lingo_val_and(ptr {target_ptr}, ptr {l_ptr}, ptr {r_ptr})\n"));
                    }
                    BinaryOp::Or => {
                        out_body.push_str(&format!("  call void @lingo_val_or(ptr {target_ptr}, ptr {l_ptr}, ptr {r_ptr})\n"));
                    }
                    BinaryOp::Eq => {
                        self.next_id += 1;
                        let bool_raw = format!("%bool_eq_{}", self.next_id);
                        out_body.push_str(&format!("  {bool_raw} = call i64 @lingo_val_eq(ptr {l_ptr}, ptr {r_ptr})\n"));
                        out_body.push_str(&format!("  call void @lingo_val_bool(ptr {target_ptr}, i64 {bool_raw})\n"));
                    }
                    BinaryOp::NotEq => {
                        self.next_id += 1;
                        let bool_raw = format!("%bool_neq_{}", self.next_id);
                        out_body.push_str(&format!("  {bool_raw} = call i64 @lingo_val_neq(ptr {l_ptr}, ptr {r_ptr})\n"));
                        out_body.push_str(&format!("  call void @lingo_val_bool(ptr {target_ptr}, i64 {bool_raw})\n"));
                    }
                    BinaryOp::Lt => {
                        self.next_id += 1;
                        let bool_raw = format!("%bool_lt_{}", self.next_id);
                        out_body.push_str(&format!("  {bool_raw} = call i64 @lingo_val_lt(ptr {l_ptr}, ptr {r_ptr})\n"));
                        out_body.push_str(&format!("  call void @lingo_val_bool(ptr {target_ptr}, i64 {bool_raw})\n"));
                    }
                    BinaryOp::Gt => {
                        self.next_id += 1;
                        let bool_raw = format!("%bool_gt_{}", self.next_id);
                        out_body.push_str(&format!("  {bool_raw} = call i64 @lingo_val_gt(ptr {l_ptr}, ptr {r_ptr})\n"));
                        out_body.push_str(&format!("  call void @lingo_val_bool(ptr {target_ptr}, i64 {bool_raw})\n"));
                    }
                    BinaryOp::GtEq => {
                        self.next_id += 1;
                        let bool_raw = format!("%bool_gte_{}", self.next_id);
                        out_body.push_str(&format!("  {bool_raw} = call i64 @lingo_val_gte(ptr {l_ptr}, ptr {r_ptr})\n"));
                        out_body.push_str(&format!("  call void @lingo_val_bool(ptr {target_ptr}, i64 {bool_raw})\n"));
                    }
                    BinaryOp::LtEq => {
                        self.next_id += 1;
                        let bool_raw = format!("%bool_lte_{}", self.next_id);
                        out_body.push_str(&format!("  {bool_raw} = call i64 @lingo_val_lte(ptr {l_ptr}, ptr {r_ptr})\n"));
                        out_body.push_str(&format!("  call void @lingo_val_bool(ptr {target_ptr}, i64 {bool_raw})\n"));
                    }
                }
            }
            IrExpr::Unary { expr, .. } => {
                let zero_alloca = self.new_temp_alloca(out_allocas);
                out_body.push_str(&format!("  call void @lingo_val_int(ptr {zero_alloca}, i64 0)\n"));
                let val_ptr = self.load_value_ptr(expr, out_allocas, out_body);
                out_body.push_str(&format!("  call void @lingo_val_sub(ptr {target_ptr}, ptr {zero_alloca}, ptr {val_ptr})\n"));
            }
            IrExpr::List(items) => {
                out_body.push_str(&format!("  call void @lingo_list_new(ptr {target_ptr})\n"));
                for item in items {
                    let item_ptr = self.load_value_ptr(item, out_allocas, out_body);
                    out_body.push_str(&format!("  call void @lingo_list_push(ptr {target_ptr}, ptr {item_ptr})\n"));
                }
            }
            IrExpr::Map(entries) => {
                out_body.push_str(&format!("  call void @lingo_map_new(ptr {target_ptr})\n"));
                for (k, v) in entries {
                    let k_ptr = self.load_value_ptr(k, out_allocas, out_body);
                    let v_ptr = self.load_value_ptr(v, out_allocas, out_body);
                    out_body.push_str(&format!("  call void @lingo_map_insert(ptr {target_ptr}, ptr {k_ptr}, ptr {v_ptr})\n"));
                }
            }
            IrExpr::Index { target, index } => {
                let t_ptr = self.load_value_ptr(target, out_allocas, out_body);
                let i_ptr = self.load_value_ptr(index, out_allocas, out_body);
                out_body.push_str(&format!("  call void @lingo_val_get(ptr {target_ptr}, ptr {t_ptr}, ptr {i_ptr})\n"));
            }
        }
    }

    fn emit_value_into(&mut self, val: &IrValue, target_ptr: &str, _out_allocas: &mut String, out_body: &mut String) {
        match val {
            IrValue::Nil => {
                out_body.push_str(&format!("  call void @lingo_val_nil(ptr {target_ptr})\n"));
            }
            IrValue::Bool(b) => {
                let int_b = if *b { 1 } else { 0 };
                out_body.push_str(&format!("  call void @lingo_val_bool(ptr {target_ptr}, i64 {int_b})\n"));
            }
            IrValue::Int(i) => {
                out_body.push_str(&format!("  call void @lingo_val_int(ptr {target_ptr}, i64 {i})\n"));
            }
            IrValue::Float(f) => {
                out_body.push_str(&format!("  call void @lingo_val_float(ptr {target_ptr}, double {f})\n"));
            }
            IrValue::Str(s) => {
                let (sym, _) = self.add_string_constant(s);
                out_body.push_str(&format!("  call void @lingo_val_str(ptr {target_ptr}, ptr {sym}, i64 {})\n", s.len()));
            }
            IrValue::Var(name) => {
                if let Some(src_alloca) = self.var_allocas.get(name) {
                    self.next_id += 1;
                    let tmp = format!("%copy_tmp_{}", self.next_id);
                    out_body.push_str(&format!("  {tmp} = load %struct.LingoValue, ptr {src_alloca}, align 8\n"));
                    out_body.push_str(&format!("  store %struct.LingoValue {tmp}, ptr {target_ptr}, align 8\n"));
                } else {
                    out_body.push_str(&format!("  call void @lingo_val_nil(ptr {target_ptr})\n"));
                }
            }
        }
    }

    fn load_value_ptr(&mut self, val: &IrValue, out_allocas: &mut String, out_body: &mut String) -> String {
        if let IrValue::Var(name) = val {
            if let Some(alloca_reg) = self.var_allocas.get(name) {
                return alloca_reg.clone();
            }
        }
        let temp_ptr = self.new_temp_alloca(out_allocas);
        self.emit_value_into(val, &temp_ptr, out_allocas, out_body);
        temp_ptr
    }
}
