use tree_sitter::Language;
use std::collections::HashMap;
use std::sync::Arc;

pub trait LanguageProvider: Send + Sync {
    fn language(&self) -> Language;
    fn fn_query(&self) -> &str;
    fn import_query(&self) -> &str;
    fn is_complexity_node(&self, kind: &str) -> bool;
}

pub struct LanguageRegistry {
    providers: HashMap<&'static str, Arc<dyn LanguageProvider>>,
}

impl LanguageRegistry {
    pub fn new() -> Self {
        let mut providers: HashMap<&'static str, Arc<dyn LanguageProvider>> = HashMap::new();
        
        let rust = Arc::new(RustProvider);
        providers.insert("rs", rust);

        let python = Arc::new(PythonProvider);
        providers.insert("py", python);

        let js_ts = Arc::new(JsTsProvider);
        providers.insert("js", js_ts.clone());
        providers.insert("jsx", js_ts.clone());
        providers.insert("ts", js_ts.clone());
        providers.insert("tsx", js_ts.clone());

        let go = Arc::new(GoProvider);
        providers.insert("go", go);

        let java = Arc::new(JavaProvider);
        providers.insert("java", java);

        let ruby = Arc::new(RubyProvider);
        providers.insert("rb", ruby);

        let php = Arc::new(PhpProvider);
        providers.insert("php", php);

        let kotlin = Arc::new(KotlinProvider);
        providers.insert("kt", kotlin.clone());
        providers.insert("kts", kotlin);

        let c_cpp = Arc::new(CCppProvider);
        providers.insert("c", c_cpp.clone());
        providers.insert("h", c_cpp.clone());
        providers.insert("cpp", c_cpp.clone());
        providers.insert("hpp", c_cpp.clone());
        providers.insert("cc", c_cpp.clone());
        providers.insert("cxx", c_cpp.clone());

        Self { providers }
    }

    pub fn get_provider(&self, ext: &str) -> Option<Arc<dyn LanguageProvider>> {
        self.providers.get(ext).cloned()
    }
}

// 각 언어별 구현체 (기존 로직 이관)

struct RustProvider;
impl LanguageProvider for RustProvider {
    fn language(&self) -> Language { tree_sitter_rust::language().into() }
    fn fn_query(&self) -> &str { "(function_item name: (identifier) @fn_name)" }
    fn import_query(&self) -> &str { "(use_declaration) @import" }
    fn is_complexity_node(&self, kind: &str) -> bool {
        matches!(kind, "if_expression" | "for_expression" | "while_expression" | "match_arm" | "&&" | "||")
    }
}

struct PythonProvider;
impl LanguageProvider for PythonProvider {
    fn language(&self) -> Language { tree_sitter_python::language().into() }
    fn fn_query(&self) -> &str { "(function_definition name: (identifier) @fn_name)" }
    fn import_query(&self) -> &str { "(import_statement) @import (import_from_statement) @import" }
    fn is_complexity_node(&self, kind: &str) -> bool {
        matches!(kind, "if_statement" | "for_statement" | "while_statement" | "case_statement" | "and" | "or")
    }
}

struct JsTsProvider;
impl LanguageProvider for JsTsProvider {
    fn language(&self) -> Language { tree_sitter_typescript::language_typescript().into() }
    fn fn_query(&self) -> &str { "(function_declaration name: (identifier) @fn_name) (method_definition name: (property_identifier) @fn_name)" }
    fn import_query(&self) -> &str { "(import_statement) @import (call_expression function: (identifier) @require_call (#eq? @require_call \"require\")) @import" }
    fn is_complexity_node(&self, kind: &str) -> bool {
        matches!(kind, "if_statement" | "for_statement" | "while_statement" | "switch_case" | "&&" | "||" | "conditional_expression")
    }
}

struct GoProvider;
impl LanguageProvider for GoProvider {
    fn language(&self) -> Language { tree_sitter_go::language().into() }
    fn fn_query(&self) -> &str { "(function_declaration name: (identifier) @fn_name) (method_declaration name: (field_identifier) @fn_name)" }
    fn import_query(&self) -> &str { "(import_declaration) @import" }
    fn is_complexity_node(&self, kind: &str) -> bool {
        matches!(kind, "if_statement" | "for_statement" | "while_statement" | "case_statement" | "&&" | "||")
    }
}

struct CCppProvider;
impl LanguageProvider for CCppProvider {
    fn language(&self) -> Language { tree_sitter_cpp::language().into() } // C++ 파서로 통합 사용 가능
    fn fn_query(&self) -> &str { "(function_definition declarator: (function_declarator declarator: (identifier) @fn_name))" }
    fn import_query(&self) -> &str { "(preproc_include) @import" }
    fn is_complexity_node(&self, kind: &str) -> bool {
        matches!(kind, "if_statement" | "for_statement" | "while_statement" | "case_statement" | "&&" | "||" | "conditional_expression")
    }
}

struct JavaProvider;
impl LanguageProvider for JavaProvider {
    fn language(&self) -> Language { tree_sitter_java::language().into() }
    fn fn_query(&self) -> &str { "(method_declaration name: (identifier) @fn_name) (constructor_declaration name: (identifier) @fn_name)" }
    fn import_query(&self) -> &str { "(import_declaration) @import" }
    fn is_complexity_node(&self, kind: &str) -> bool {
        matches!(kind, "if_statement" | "for_statement" | "while_statement" | "switch_label" | "&&" | "||" | "ternary_expression")
    }
}

struct RubyProvider;
impl LanguageProvider for RubyProvider {
    fn language(&self) -> Language { tree_sitter_ruby::language().into() }
    fn fn_query(&self) -> &str { "(method name: (identifier) @fn_name) (singleton_method name: (identifier) @fn_name)" }
    fn import_query(&self) -> &str { "(call method: (identifier) @require_call (#match? @require_call \"require|load\")) @import" }
    fn is_complexity_node(&self, kind: &str) -> bool {
        matches!(kind, "if" | "unless" | "for" | "while" | "until" | "when" | "&&" | "||" | "and" | "or")
    }
}

struct PhpProvider;
impl LanguageProvider for PhpProvider {
    fn language(&self) -> Language { tree_sitter_php::language_php().into() }
    fn fn_query(&self) -> &str { "(function_definition name: (name) @fn_name) (method_declaration name: (name) @fn_name)" }
    fn import_query(&self) -> &str { "(include_expression) @import (require_expression) @import (namespace_use_declaration) @import" }
    fn is_complexity_node(&self, kind: &str) -> bool {
        matches!(kind, "if_statement" | "for_statement" | "foreach_statement" | "while_statement" | "case_statement" | "&&" | "||" | "and" | "or" | "conditional_expression")
    }
}

struct KotlinProvider;
impl LanguageProvider for KotlinProvider {
    fn language(&self) -> Language { tree_sitter_kotlin::language().into() }
    fn fn_query(&self) -> &str { "(function_declaration name: (simple_identifier) @fn_name)" }
    fn import_query(&self) -> &str { "(import_header) @import" }
    fn is_complexity_node(&self, kind: &str) -> bool {
        matches!(kind, "if_expression" | "for_statement" | "while_statement" | "when_entry" | "&&" | "||")
    }
}
