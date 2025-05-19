use minijinja::{Environment, UndefinedBehavior};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use serde_json::Value;
use std::slice;
use std::str;
use std::sync::Mutex;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static ENV: Lazy<Mutex<Environment<'static>>> = Lazy::new(|| Mutex::new(Environment::new()));

const TEMPLATE_KEY: &str = "template:01JVK339CW6Q67VAMXCA7XAK7D";

#[link(wasm_import_module = "console")]
extern "C" {
    fn log(ptr: *const u8, len: usize);
    fn error(ptr: *const u8, len: usize);
}

pub fn js_log(s: &str) {
    unsafe {
        log(s.as_ptr(), s.len());
    }
}

pub fn js_error(msg: &str) {
    unsafe {
        error(msg.as_ptr(), msg.len());
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TemplateSource {
    name: String,
    source: String,
    components: Vec<String>,
}

type Entity = Map<String, Value>;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum CompileErrorType {
    ParseError,
    MissingDependency,
    CompileError,
}

#[derive(Serialize, Deserialize)]
struct CompileError {
    error_type: CompileErrorType,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    missing_dependencies: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum RenderErrorType {
    ParseError,
    RenderError,
}

#[derive(Serialize, Deserialize)]
struct RenderError {
    error_type: RenderErrorType,
    message: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum CompileResult {
    Success,
    Error { error: CompileError },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum RenderResult {
    Success { result: String },
    Error { error: RenderError },
}

fn write_to_memory(ptr: *mut u8, data: &[u8], max_len: usize) -> usize {
    let len = data.len().min(max_len);
    unsafe {
        std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, len);
    }
    len
}

#[no_mangle]
pub extern "C" fn compile_templates(
    ptr: *const u8,
    len: usize,
    out_ptr: *mut u8,
    out_len: usize,
) -> usize {
    let json_bytes = unsafe { slice::from_raw_parts(ptr, len) };
    let entities: Vec<Entity> = match serde_json::from_slice(json_bytes) {
        Ok(t) => t,
        Err(e) => {
            let error = CompileError {
                error_type: CompileErrorType::ParseError,
                message: e.to_string(),
                missing_dependencies: None,
            };
            let result = CompileResult::Error { error };
            let result_json = serde_json::to_string(&result).unwrap();
            return write_to_memory(out_ptr, result_json.as_bytes(), out_len);
        }
    };

    let templates: Vec<TemplateSource> = entities
        .iter()
        .filter_map(|e| {
            js_log(&format!("template entity: {:#?}", e));
            let template = e.get(TEMPLATE_KEY).and_then(|v| v.as_object())?;
            let name = template.get("name").and_then(|v| v.as_str())?;
            let source = template.get("source").and_then(|v| v.as_str())?;
            let components = template.get("components").and_then(|v| v.as_array())?;
            let components = components
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
            Some(TemplateSource {
                name: name.to_string(),
                source: source.to_string(),
                components,
            })
        })
        .collect();

    js_log(&format!("templates: {:#?}", templates));

    let mut env = ENV.lock().unwrap();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    for t in &templates {
        match env.add_template_owned(t.name.clone(), t.source.clone()) {
            Ok(_) => continue,
            Err(e) => {
                let deps = if e.to_string().contains("not found") {
                    Some(
                        e.to_string()
                            .split("not found")
                            .filter_map(|s| {
                                let s = s.trim();
                                if s.is_empty() {
                                    None
                                } else {
                                    Some(s.to_string())
                                }
                            })
                            .collect(),
                    )
                } else {
                    None
                };

                let error = CompileError {
                    error_type: if deps.is_some() {
                        CompileErrorType::MissingDependency
                    } else {
                        CompileErrorType::CompileError
                    },
                    message: e.to_string(),
                    missing_dependencies: deps,
                };

                let result = CompileResult::Error { error };
                let result_json = serde_json::to_string(&result).unwrap();
                return write_to_memory(out_ptr, result_json.as_bytes(), out_len);
            }
        }
    }

    let result = CompileResult::Success;
    let result_json = serde_json::to_string(&result).unwrap();
    write_to_memory(out_ptr, result_json.as_bytes(), out_len)
}

#[no_mangle]
pub extern "C" fn render_template(
    name_ptr: *const u8,
    name_len: usize,
    ctx_ptr: *const u8,
    ctx_len: usize,
    out_ptr: *mut u8,
    out_len: usize,
) -> usize {
    let name_bytes = unsafe { slice::from_raw_parts(name_ptr, name_len) };
    let name = match str::from_utf8(name_bytes) {
        Ok(n) => n,
        Err(_) => {
            let result = RenderResult::Error {
                error: RenderError {
                    error_type: RenderErrorType::ParseError,
                    message: "Invalid template name".to_string(),
                },
            };
            let result_json = serde_json::to_string(&result).unwrap();
            return write_to_memory(out_ptr, result_json.as_bytes(), out_len);
        }
    };
    let ctx_bytes = unsafe { slice::from_raw_parts(ctx_ptr, ctx_len) };
    let ctx: Value = match serde_json::from_slice(ctx_bytes) {
        Ok(c) => c,
        Err(_) => {
            let result = RenderResult::Error {
                error: RenderError {
                    error_type: RenderErrorType::ParseError,
                    message: "Invalid context".to_string(),
                },
            };
            let result_json = serde_json::to_string(&result).unwrap();
            return write_to_memory(out_ptr, result_json.as_bytes(), out_len);
        }
    };

    let env = ENV.lock().unwrap();
    let tmpl = match env.get_template(name) {
        Ok(t) => t,
        Err(_) => {
            let result = RenderResult::Error {
                error: RenderError {
                    error_type: RenderErrorType::ParseError,
                    message: "Invalid template name".to_string(),
                },
            };
            let result_json = serde_json::to_string(&result).unwrap();
            return write_to_memory(out_ptr, result_json.as_bytes(), out_len);
        }
    };
    let rendered = match tmpl.render(ctx) {
        Ok(s) => s,
        Err(_) => {
            let result = RenderResult::Error {
                error: RenderError {
                    error_type: RenderErrorType::RenderError,
                    message: "Failed to render template".to_string(),
                },
            };
            let result_json = serde_json::to_string(&result).unwrap();
            return write_to_memory(out_ptr, result_json.as_bytes(), out_len);
        }
    };
    let result = RenderResult::Success { result: rendered };
    let result_json = serde_json::to_string(&result).unwrap();
    write_to_memory(out_ptr, result_json.as_bytes(), out_len)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use std::ffi::c_void;

    fn setup_test_templates() -> Vec<u8> {
        let templates = vec![
            TemplateSource {
                name: "test1".to_string(),
                source: "Hello {{ name }}!".to_string(),
                components: vec![],
            },
            TemplateSource {
                name: "test2".to_string(),
                source: "{% if condition %}True{% else %}False{% endif %}".to_string(),
                components: vec![],
            },
        ];
        serde_json::to_vec(&templates).unwrap()
    }

    fn setup_template_with_dependency() -> Vec<u8> {
        let templates = vec![TemplateSource {
            name: "parent".to_string(),
            source: "{% include 'child' %}".to_string(),
            components: vec![],
        }];
        serde_json::to_vec(&templates).unwrap()
    }

    #[test]
    fn test_compile_templates() {
        let templates = setup_test_templates();
        let mut output = vec![0u8; 1024];
        let result = unsafe {
            compile_templates(
                templates.as_ptr(),
                templates.len(),
                output.as_mut_ptr(),
                output.len(),
            )
        };

        let result_str = String::from_utf8_lossy(&output[..result]);
        let result: CompileResult = serde_json::from_str(&result_str).unwrap();
        assert!(matches!(result, CompileResult::Success));
    }

    #[test]
    fn test_compile_invalid_templates() {
        let invalid_json = b"invalid json";
        let mut output = vec![0u8; 1024];
        let result = unsafe {
            compile_templates(
                invalid_json.as_ptr(),
                invalid_json.len(),
                output.as_mut_ptr(),
                output.len(),
            )
        };

        let result_str = String::from_utf8_lossy(&output[..result]);
        let result: CompileResult = serde_json::from_str(&result_str).unwrap();
        match result {
            CompileResult::Error { error } => {
                assert_eq!(error.error_type, CompileErrorType::ParseError);
                assert!(error.message.contains("expected value"));
                assert!(error.missing_dependencies.is_none());
            }
            _ => panic!("Expected error result"),
        }
    }

    // note: we cannot currently inspect template dependencies from the template source
    // #[test]
    // fn test_compile_template_with_missing_dependency() {
    //     let templates = setup_template_with_dependency();
    //     let mut output = vec![0u8; 1024];
    //     let result = unsafe {
    //         compile_templates(
    //             templates.as_ptr(),
    //             templates.len(),
    //             output.as_mut_ptr(),
    //             output.len(),
    //         )
    //     };

    //     let result_str = String::from_utf8_lossy(&output[..result]);
    //     println!("Result string: {}", result_str); // Add debug output
    //     let result: CompileResult = serde_json::from_str(&result_str).unwrap();
    //     match result {
    //         CompileResult::Error(error) => {
    //             assert_eq!(error.error_type, CompileErrorType::MissingDependency);
    //             assert!(error.message.contains("not found"));
    //             assert!(error.missing_dependencies.is_some());
    //             let deps = error.missing_dependencies.unwrap();
    //             assert!(deps.contains(&"child".to_string()));
    //         }
    //         _ => panic!("Expected error result"),
    //     }
    // }

    #[test]
    fn test_render_template() {
        // First compile the template
        let templates = setup_test_templates();
        let mut output = vec![0u8; 1024];
        unsafe {
            compile_templates(
                templates.as_ptr(),
                templates.len(),
                output.as_mut_ptr(),
                output.len(),
            );
        }

        // Test rendering template1
        let name = "test1";
        let context = json!({"name": "World"});
        let context_bytes = serde_json::to_vec(&context).unwrap();

        let mut output = vec![0u8; 1024];
        let result = unsafe {
            render_template(
                name.as_ptr(),
                name.len(),
                context_bytes.as_ptr(),
                context_bytes.len(),
                output.as_mut_ptr(),
                output.len(),
            )
        };

        let result_str = String::from_utf8_lossy(&output[..result]);
        let result: RenderResult = serde_json::from_str(&result_str).unwrap();
        match result {
            RenderResult::Success { result } => {
                assert_eq!(result, "Hello World!");
            }
            RenderResult::Error { error } => {
                panic!("Expected success result, got error: {}", error.message);
            }
        }
    }

    #[test]
    fn test_render_template_with_condition() {
        // First compile the template
        let templates = setup_test_templates();
        let mut output = vec![0u8; 1024];
        unsafe {
            compile_templates(
                templates.as_ptr(),
                templates.len(),
                output.as_mut_ptr(),
                output.len(),
            );
        }

        // Test rendering template2 with true condition
        let name = "test2";
        let context = json!({"condition": true});
        let context_bytes = serde_json::to_vec(&context).unwrap();

        let mut output = vec![0u8; 1024];
        let result = unsafe {
            render_template(
                name.as_ptr(),
                name.len(),
                context_bytes.as_ptr(),
                context_bytes.len(),
                output.as_mut_ptr(),
                output.len(),
            )
        };

        let result_str = String::from_utf8_lossy(&output[..result]);
        let result: RenderResult = serde_json::from_str(&result_str).unwrap();
        match result {
            RenderResult::Success { result } => {
                assert_eq!(result, "True");
            }
            RenderResult::Error { error } => {
                panic!("Expected success result, got error: {}", error.message);
            }
        }
    }

    #[test]
    fn test_render_template_with_false_condition() {
        // First compile the template
        let templates = setup_test_templates();
        let mut output = vec![0u8; 1024];
        unsafe {
            compile_templates(
                templates.as_ptr(),
                templates.len(),
                output.as_mut_ptr(),
                output.len(),
            );
        }
        let context = json!({"condition": false});
        let context_bytes = serde_json::to_vec(&context).unwrap();

        let name = "test2";

        let mut output = vec![0u8; 1024];
        let result = unsafe {
            render_template(
                name.as_ptr(),
                name.len(),
                context_bytes.as_ptr(),
                context_bytes.len(),
                output.as_mut_ptr(),
                output.len(),
            )
        };

        let result_str = String::from_utf8_lossy(&output[..result]);
        let result: RenderResult = serde_json::from_str(&result_str).unwrap();
        match result {
            RenderResult::Success { result } => {
                assert_eq!(result, "False");
            }
            RenderResult::Error { error } => {
                panic!("Expected success result, got error: {}", error.message);
            }
        }
    }

    #[test]
    fn test_render_nonexistent_template() {
        let name = "nonexistent";
        let context = json!({});
        let context_bytes = serde_json::to_vec(&context).unwrap();

        let mut output = vec![0u8; 1024];
        let result = unsafe {
            render_template(
                name.as_ptr(),
                name.len(),
                context_bytes.as_ptr(),
                context_bytes.len(),
                output.as_mut_ptr(),
                output.len(),
            )
        };

        let result_str = String::from_utf8_lossy(&output[..result]);
        let result: RenderResult = serde_json::from_str(&result_str).unwrap();
        match result {
            RenderResult::Error { error } => {
                assert_eq!(error.error_type, RenderErrorType::ParseError);
                assert!(error.message.contains("Invalid template name"));
            }
            RenderResult::Success { result } => {
                panic!("Expected error result, got success: {}", result);
            }
        }
    }

    #[test]
    fn test_render_with_invalid_context() {
        let name = "test1";
        let invalid_context = b"invalid json";

        let mut output = vec![0u8; 1024];
        let result = unsafe {
            render_template(
                name.as_ptr(),
                name.len(),
                invalid_context.as_ptr(),
                invalid_context.len(),
                output.as_mut_ptr(),
                output.len(),
            )
        };

        println!("Result: {}", result);

        let result_str = String::from_utf8_lossy(&output[..result]);
        let result: RenderResult = serde_json::from_str(&result_str).unwrap();
        match result {
            RenderResult::Error { error } => {
                assert_eq!(error.error_type, RenderErrorType::ParseError);
                assert!(error.message.contains("Invalid context"));
            }
            RenderResult::Success { result } => {
                panic!("Expected error result, got success: {}", result);
            }
        }
    }
}
