import { Entity, LeafRenderer, TemplateSource } from "./index"
import * as fs from "fs"
import * as path from "path"
import { describe, expect, it, beforeAll, beforeEach } from "@jest/globals"

describe("LeafRenderer", () => {
  let renderer: LeafRenderer
  let wasmModule: WebAssembly.Module

  beforeAll(async () => {
    // Load the WASM module
    const wasmPath = path.resolve(
      __dirname,
      "../target/wasm32-unknown-unknown/release/leaf_render.wasm"
    )
    const wasmBuffer = fs.readFileSync(wasmPath)
    wasmModule = await WebAssembly.compile(wasmBuffer)
    renderer = new LeafRenderer(wasmModule)
  })

  describe("compileTemplates", () => {
    it("should successfully compile valid templates", () => {
      renderer.registerComponent([
        "name_component",
        {
          type: "object",
          properties: {
            name: { type: "string" },
          },
          required: ["name"],
        },
      ])
      renderer.registerComponent([
        "condition_component",
        {
          type: "object",
          properties: {
            condition: { type: "boolean" },
          },
          required: ["condition"],
        },
      ])
      const templates: Entity<TemplateSource>[] = [
        {
          "template:01JVK339CW6Q67VAMXCA7XAK7D": {
            name: "test1",
            source: "Hello {{ name }}!",
            components: ["name_component"],
          },
        },
        {
          "template:01JVK339CW6Q67VAMXCA7XAK7D": {
            name: "test2",
            source: "{% if condition %}True{% else %}False{% endif %}",
            components: ["condition_component"],
          },
        },
      ]

      const result = renderer.compileTemplates(templates)
      expect(result).toEqual({ type: "Success" })
    })

    it("should handle invalid templates", () => {
      const templates: Entity<TemplateSource>[] = [
        {
          "template:01JVK339CW6Q67VAMXCA7XAK7D": {
            name: "invalid",
            source: "{{ invalid syntax }}",
            components: [],
          },
        },
      ]

      const result = renderer.compileTemplates(templates)
      console.log("result parsed", result)
      expect(result.type).toBe("Error")
      if (result.type === "Error") {
        expect(result.error.error_type).toBe("CompileError")
      }
    })

    // note: we cannot yet inspect template dependencies from the template source
    // it("should handle missing dependencies", () => {
    //   const templates: TemplateSource[] = [
    //     {
    //       name: "parent",
    //       source: "{% include 'child' %}",
    //     },
    //   ]

    //   const result = renderer.compileTemplates(templates)
    //   expect(result.type).toBe("Error")
    //   if (result.type === "Error") {
    //     expect(result.error.error_type).toBe("MissingDependency")
    //     expect(result.error.missing_dependencies).toContain("child")
    //   }
    // })
  })

  describe("renderTemplate", () => {
    beforeEach(() => {
      renderer.registerComponent([
        "name_component",
        {
          type: "object",
          properties: {
            name: { type: "string" },
          },
          required: ["name"],
        },
      ])
      renderer.registerComponent([
        "condition_component",
        {
          type: "object",
          properties: {
            condition: { type: "boolean" },
          },
          required: ["condition"],
        },
      ])

      // Compile test templates before each test
      const templates: Entity<TemplateSource>[] = [
        {
          "template:01JVK339CW6Q67VAMXCA7XAK7D": {
            name: "test1",
            source: "Hello {{ name }}!",
            components: ["name_component"],
          },
        },
        {
          "template:01JVK339CW6Q67VAMXCA7XAK7D": {
            name: "test2",
            source: "{% if condition %}True{% else %}False{% endif %}",
            components: ["condition_component"],
          },
        },
      ]
      renderer.compileTemplates(templates)
    })

    it("should render template with simple variable", () => {
      const result = renderer.renderTemplate("test1", { name: "World" })
      expect(result.type).toBe("Success")
      if (result.type === "Success") {
        expect(result.result).toBe("Hello World!")
      }
    })

    it("should render template with condition (true)", () => {
      const result = renderer.renderTemplate("test2", { condition: true })
      expect(result.type).toBe("Success")
      if (result.type === "Success") {
        expect(result.result).toBe("True")
      }
    })

    it("should render template with condition (false)", () => {
      const result = renderer.renderTemplate("test2", { condition: false })
      expect(result.type).toBe("Success")
      if (result.type === "Success") {
        expect(result.result).toBe("False")
      }
    })

    it("should handle non-existent template", () => {
      const result = renderer.renderTemplate("nonexistent", {})
      expect(result.type).toBe("Error")
      if (result.type === "Error") {
        expect(result.error.error_type).toBe("ParseError")
      }
    })
  })
})
