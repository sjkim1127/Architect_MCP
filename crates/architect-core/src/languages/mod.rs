use std::collections::HashMap;
use std::sync::Arc;
use tree_sitter::Language;

pub trait LanguageProvider: Send + Sync {
    fn language(&self) -> Language;
    fn fn_query(&self) -> &str;
    fn import_query(&self) -> &str;
    fn security_query(&self) -> &str;
    fn api_query(&self) -> &str;
    fn outbound_query(&self) -> &str;
    fn error_query(&self) -> &str;
    fn is_complexity_node(&self, kind: &str) -> bool;
}

pub struct LanguageRegistry {
    providers: HashMap<&'static str, Arc<dyn LanguageProvider>>,
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
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
    fn language(&self) -> Language {
        tree_sitter_rust::language()
    }
    fn fn_query(&self) -> &str {
        "(function_item name: (identifier) @fn_name)"
    }
    fn import_query(&self) -> &str {
        "(use_declaration) @import"
    }
    fn security_query(&self) -> &str {
        "(call_expression function: (identifier) @name (#match? @name \"eval|exec|system|panic|unsafe\")) @security"
    }
    fn api_query(&self) -> &str {
        "(attribute (path) @attr_path (#match? @path \"get|post|put|delete|patch|route\")) @api"
    }
    fn outbound_query(&self) -> &str {
        "(call_expression function: (field_expression field: (field_identifier) @name (#match? @name \"get|post|put|delete|request|send\"))) @outbound"
    }
    fn error_query(&self) -> &str {
        "(call_expression function: (field_expression field: (field_identifier) @name (#match? @name \"unwrap|expect\"))) @error"
    }
    fn is_complexity_node(&self, kind: &str) -> bool {
        matches!(
            kind,
            "if_expression" | "for_expression" | "while_expression" | "match_arm" | "&&" | "||"
        )
    }
}

struct PythonProvider;
impl LanguageProvider for PythonProvider {
    fn language(&self) -> Language {
        tree_sitter_python::language()
    }
    fn fn_query(&self) -> &str {
        "(function_definition name: (identifier) @fn_name)"
    }
    fn import_query(&self) -> &str {
        "(import_statement) @import (import_from_statement) @import"
    }
    fn security_query(&self) -> &str {
        "(call function: (identifier) @name (#match? @name \"eval|exec|os\\\\.system|subprocess|pickle\")) @security"
    }
    fn api_query(&self) -> &str {
        "(decorator (call function: (attribute object: (identifier) @obj attribute: (identifier) @name (#match? @name \"get|post|put|delete|route\")))) @api"
    }
    fn outbound_query(&self) -> &str {
        "(call function: (attribute attribute: (identifier) @name (#match? @name \"get|post|put|delete|request|fetch\"))) @outbound"
    }
    fn error_query(&self) -> &str {
        "(except_clause (block (pass_statement))) @error_swallowed"
    }
    fn is_complexity_node(&self, kind: &str) -> bool {
        matches!(
            kind,
            "if_statement" | "for_statement" | "while_statement" | "case_statement" | "and" | "or"
        )
    }
}

struct JsTsProvider;
impl LanguageProvider for JsTsProvider {
    fn language(&self) -> Language {
        tree_sitter_typescript::language_typescript()
    }
    fn fn_query(&self) -> &str {
        "(function_declaration name: (identifier) @fn_name) (method_definition name: (property_identifier) @fn_name)"
    }
    fn import_query(&self) -> &str {
        "(import_statement) @import (call_expression function: (identifier) @require_call (#eq? @require_call \"require\")) @import"
    }
    fn security_query(&self) -> &str {
        "(call_expression function: (identifier) @name (#match? @name \"eval|exec|child_process|fs\\\\.readSync\")) @security"
    }
    fn api_query(&self) -> &str {
        "(call_expression function: (member_expression property: (property_identifier) @name (#match? @name \"get|post|put|delete|use|route\"))) @api"
    }
    fn outbound_query(&self) -> &str {
        "(call_expression function: (identifier) @name (#match? @name \"fetch|axios\")) @outbound"
    }
    fn error_query(&self) -> &str {
        "(catch_clause (statement_block) @block (#match? @block \"^\\\\{\\\\s*\\\\}\")) @error_swallowed"
    }
    fn is_complexity_node(&self, kind: &str) -> bool {
        matches!(
            kind,
            "if_statement"
                | "for_statement"
                | "while_statement"
                | "switch_case"
                | "&&"
                | "||"
                | "conditional_expression"
        )
    }
}

struct GoProvider;
impl LanguageProvider for GoProvider {
    fn language(&self) -> Language {
        tree_sitter_go::language()
    }
    fn fn_query(&self) -> &str {
        "(function_declaration name: (identifier) @fn_name) (method_declaration name: (field_identifier) @fn_name)"
    }
    fn import_query(&self) -> &str {
        "(import_declaration) @import"
    }
    fn security_query(&self) -> &str {
        "(call_expression function: (selector_expression field: (field_identifier) @name (#match? @name \"Command|UnsafePointer|Panic\"))) @security"
    }
    fn api_query(&self) -> &str {
        "(call_expression function: (selector_expression field: (field_identifier) @name (#match? @name \"Get|Post|Put|Delete|Handle|HandleFunc\"))) @api"
    }
    fn outbound_query(&self) -> &str {
        "(call_expression function: (selector_expression field: (field_identifier) @name (#match? @name \"Get|Post|Do|NewRequest\"))) @outbound"
    }
    fn error_query(&self) -> &str {
        "(if_statement condition: (binary_expression left: (identifier) @err (#eq? @err \"err\") right: (nil) @nil (#eq? @nil \"nil\")) consequence: (block)) @error_unchecked"
    }
    fn is_complexity_node(&self, kind: &str) -> bool {
        matches!(
            kind,
            "if_statement" | "for_statement" | "while_statement" | "case_statement" | "&&" | "||"
        )
    }
}

struct CCppProvider;
impl LanguageProvider for CCppProvider {
    fn language(&self) -> Language {
        tree_sitter_cpp::language()
    } // C++ 파서로 통합 사용 가능
    fn fn_query(&self) -> &str {
        "(function_definition declarator: (function_declarator declarator: (identifier) @fn_name))"
    }
    fn import_query(&self) -> &str {
        "(preproc_include) @import"
    }
    fn security_query(&self) -> &str {
        "(call_expression function: (identifier) @name (#match? @name \"system|exec|gets|strcpy|sprintf|scanf\")) @security"
    }
    fn api_query(&self) -> &str {
        "(attribute (identifier) @name (#match? @name \"get|post|put|delete|route\")) @api"
    }
    fn outbound_query(&self) -> &str {
        "(call_expression function: (identifier) @name (#match? @name \"connect|send|sendto|write|curl_easy_perform\")) @outbound"
    }
    fn error_query(&self) -> &str {
        "(if_statement condition: (parenthesized_expression (binary_expression left: (identifier) @res right: (number_literal) @val (#eq? @val \"-1\")))) @error_unchecked"
    }
    fn is_complexity_node(&self, kind: &str) -> bool {
        matches!(
            kind,
            "if_statement"
                | "for_statement"
                | "while_statement"
                | "case_statement"
                | "&&"
                | "||"
                | "conditional_expression"
        )
    }
}

struct JavaProvider;
impl LanguageProvider for JavaProvider {
    fn language(&self) -> Language {
        tree_sitter_java::language()
    }
    fn fn_query(&self) -> &str {
        "(method_declaration name: (identifier) @fn_name) (constructor_declaration name: (identifier) @fn_name)"
    }
    fn import_query(&self) -> &str {
        "(import_declaration) @import"
    }
    fn security_query(&self) -> &str {
        "(method_invocation name: (identifier) @name (#match? @name \"exec|eval|System\\\\.exit|getRuntime\")) @security"
    }
    fn api_query(&self) -> &str {
        "(annotation name: (identifier) @name (#match? @name \"GetMapping|PostMapping|PutMapping|DeleteMapping|RequestMapping\")) @api"
    }
    fn outbound_query(&self) -> &str {
        "(method_invocation name: (identifier) @name (#match? @name \"send|execute|exchange|getForObject\")) @outbound"
    }
    fn error_query(&self) -> &str {
        "(catch_clause (block) @block (#match? @block \"^\\\\{\\\\s*\\\\}\")) @error_swallowed"
    }
    fn is_complexity_node(&self, kind: &str) -> bool {
        matches!(
            kind,
            "if_statement"
                | "for_statement"
                | "while_statement"
                | "switch_label"
                | "&&"
                | "||"
                | "ternary_expression"
        )
    }
}

struct RubyProvider;
impl LanguageProvider for RubyProvider {
    fn language(&self) -> Language {
        tree_sitter_ruby::language()
    }
    fn fn_query(&self) -> &str {
        "(method name: (identifier) @fn_name) (singleton_method name: (identifier) @fn_name)"
    }
    fn import_query(&self) -> &str {
        "(call method: (identifier) @require_call (#match? @require_call \"require|load\")) @import"
    }
    fn security_query(&self) -> &str {
        "(call method: (identifier) @name (#match? @name \"eval|system|exec|exit\")) @security"
    }
    fn api_query(&self) -> &str {
        "(call method: (identifier) @name (#match? @name \"get|post|put|delete|patch|resource\")) @api"
    }
    fn outbound_query(&self) -> &str {
        "(call method: (identifier) @name (#match? @name \"get|post|request|get_response\")) @outbound"
    }
    fn error_query(&self) -> &str {
        "(rescue (then (block (pass_statement)))) @error_swallowed"
    }
    fn is_complexity_node(&self, kind: &str) -> bool {
        matches!(
            kind,
            "if" | "unless" | "for" | "while" | "until" | "when" | "&&" | "||" | "and" | "or"
        )
    }
}

struct PhpProvider;
impl LanguageProvider for PhpProvider {
    fn language(&self) -> Language {
        tree_sitter_php::language_php()
    }
    fn fn_query(&self) -> &str {
        "(function_definition name: (name) @fn_name) (method_declaration name: (name) @fn_name)"
    }
    fn import_query(&self) -> &str {
        "(include_expression) @import (require_expression) @import (namespace_use_declaration) @import"
    }
    fn security_query(&self) -> &str {
        "(function_call_expression function: (name) @name (#match? @name \"eval|exec|system|passthru|shell_exec\")) @security"
    }
    fn api_query(&self) -> &str {
        "(attribute (attribute_group_clause_list (attribute name: (name) @name (#match? @name \"Get|Post|Put|Delete|Route\")))) @api"
    }
    fn outbound_query(&self) -> &str {
        "(function_call_expression function: (name) @name (#match? @name \"curl_init|file_get_contents|get_headers\")) @outbound"
    }
    fn error_query(&self) -> &str {
        "(catch_clause (compound_statement) @block (#match? @block \"^\\\\{\\\\s*\\\\}\")) @error_swallowed"
    }
    fn is_complexity_node(&self, kind: &str) -> bool {
        matches!(
            kind,
            "if_statement"
                | "for_statement"
                | "foreach_statement"
                | "while_statement"
                | "case_statement"
                | "&&"
                | "||"
                | "and"
                | "or"
                | "conditional_expression"
        )
    }
}

struct KotlinProvider;
impl LanguageProvider for KotlinProvider {
    fn language(&self) -> Language {
        tree_sitter_kotlin::language()
    }
    fn fn_query(&self) -> &str {
        "(function_declaration name: (simple_identifier) @fn_name)"
    }
    fn import_query(&self) -> &str {
        "(import_header) @import"
    }
    fn security_query(&self) -> &str {
        "(call_expression (user_type (reference_expression (simple_identifier) @name (#match? @name \"eval|exec|panic|System\\\\.exit\")))) @security"
    }
    fn api_query(&self) -> &str {
        "(annotation (user_type (reference_expression (simple_identifier) @name (#match? @name \"GetMapping|PostMapping|PutMapping|DeleteMapping|Path\")))) @api"
    }
    fn outbound_query(&self) -> &str {
        "(call_expression (user_type (reference_expression (simple_identifier) @name (#match? @name \"send|execute|exchange|getForObject\")))) @outbound"
    }
    fn error_query(&self) -> &str {
        "(catch_clause (block) @block (#match? @block \"^\\\\{\\\\s*\\\\}\")) @error_swallowed"
    }
    fn is_complexity_node(&self, kind: &str) -> bool {
        matches!(
            kind,
            "if_expression" | "for_statement" | "while_statement" | "when_entry" | "&&" | "||"
        )
    }
}
