#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::c_void;

    fn setup_test_templates() -> Vec<u8> {
        let templates = vec![
            TemplateSource {
                name: "test1".to_string(),
                source: "Hello {{ name }}!".to_string(),
            },
            TemplateSource {
                name: "test2".to_string(),
                source: "{% if condition %}True{% else %}False{% endif %}".to_string(),
            },
        ];
        serde_json::to_vec(&templates).unwrap()
    }

    #[test]
    fn test_compile_templates() {
        let templates = setup_test_templates();
        let result = unsafe { compile_templates(templates.as_ptr(), templates.len()) };
        assert_eq!(result, 0);
    }

    #[test]
    fn test_compile_invalid_templates() {
        let invalid_json = b"invalid json";
        let result = unsafe { compile_templates(invalid_json.as_ptr(), invalid_json.len()) };
        assert_eq!(result, 1);
    }

    #[test]
    fn test_render_template() {
        // First compile the template
        let templates = setup_test_templates();
        unsafe {
            compile_templates(templates.as_ptr(), templates.len());
        }

        // Test rendering template1
        let name = "test1";
        let context = json!({"name": "World"});
        let context_bytes = serde_json::to_vec(&context).unwrap();

        let mut output = vec![0u8; 100];
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

        let rendered = String::from_utf8_lossy(&output[..result]);
        assert_eq!(rendered.trim(), "Hello World!");
    }

    #[test]
    fn test_render_template_with_condition() {
        // First compile the template
        let templates = setup_test_templates();
        unsafe {
            compile_templates(templates.as_ptr(), templates.len());
        }

        // Test rendering template2 with true condition
        let name = "test2";
        let context = json!({"condition": true});
        let context_bytes = serde_json::to_vec(&context).unwrap();

        let mut output = vec![0u8; 100];
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

        let rendered = String::from_utf8_lossy(&output[..result]);
        assert_eq!(rendered.trim(), "True");

        // Test with false condition
        let context = json!({"condition": false});
        let context_bytes = serde_json::to_vec(&context).unwrap();

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

        let rendered = String::from_utf8_lossy(&output[..result]);
        assert_eq!(rendered.trim(), "False");
    }

    #[test]
    fn test_render_nonexistent_template() {
        let name = "nonexistent";
        let context = json!({});
        let context_bytes = serde_json::to_vec(&context).unwrap();

        let mut output = vec![0u8; 100];
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

        assert_eq!(result, 0);
    }

    #[test]
    fn test_render_with_invalid_context() {
        let name = "test1";
        let invalid_context = b"invalid json";

        let mut output = vec![0u8; 100];
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

        assert_eq!(result, 0);
    }
}
