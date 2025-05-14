import { Minijinja, TemplateSource } from "./index"
import * as fs from "fs"
import * as path from "path"
import { describe, expect, it, beforeAll, beforeEach } from "@jest/globals"

describe("Minijinja", () => {
  let minijinja: Minijinja
  let wasmModule: WebAssembly.Module

  beforeAll(async () => {
    // Load the WASM module
    const wasmPath = path.resolve(
      __dirname,
      "../target/wasm32-unknown-unknown/release/leaf_render.wasm"
    )
    const wasmBuffer = fs.readFileSync(wasmPath)
    wasmModule = await WebAssembly.compile(wasmBuffer)
    minijinja = new Minijinja(wasmModule)
  })

  describe("compileTemplates", () => {
    it("should successfully compile valid templates", () => {
      const templates: TemplateSource[] = [
        {
          name: "test1",
          source: "Hello {{ name }}!",
        },
        {
          name: "test2",
          source: "{% if condition %}True{% else %}False{% endif %}",
        },
      ]

      const result = minijinja.compileTemplates(templates)
      expect(result).toEqual({ type: "Success" })
    })

    it("should handle invalid templates", () => {
      const templates: TemplateSource[] = [
        {
          name: "invalid",
          source: "{{ invalid syntax }}",
        },
      ]

      const result = minijinja.compileTemplates(templates)
      expect(result.type).toBe("Error")
      if (result.type === "Error") {
        expect(result.error.error_type).toBe("CompileError")
      }
    })

    it("should handle missing dependencies", () => {
      const templates: TemplateSource[] = [
        {
          name: "parent",
          source: "{% include 'child' %}",
        },
      ]

      const result = minijinja.compileTemplates(templates)
      expect(result.type).toBe("Error")
      if (result.type === "Error") {
        expect(result.error.error_type).toBe("MissingDependency")
        expect(result.error.missing_dependencies).toContain("child")
      }
    })
  })

  describe("renderTemplate", () => {
    beforeEach(() => {
      // Compile test templates before each test
      const templates: TemplateSource[] = [
        {
          name: "test1",
          source: "Hello {{ name }}!",
        },
        {
          name: "test2",
          source: "{% if condition %}True{% else %}False{% endif %}",
        },
      ]
      minijinja.compileTemplates(templates)
    })

    it("should render template with simple variable", () => {
      const result = minijinja.renderTemplate("test1", { name: "World" })
      expect(result).toBe("Hello World!")
    })

    it("should render template with condition (true)", () => {
      const result = minijinja.renderTemplate("test2", { condition: true })
      expect(result).toBe("True")
    })

    it("should render template with condition (false)", () => {
      const result = minijinja.renderTemplate("test2", { condition: false })
      expect(result).toBe("False")
    })

    it("should handle non-existent template", () => {
      expect(() => {
        minijinja.renderTemplate("nonexistent", {})
      }).toThrow()
    })
  })
})
