# Leaf Render

A Rust library for rendering Minijinja templates with Leaf context.

## Development

```bash
pnpm install
pnpm build # build the wasm module and compile the typescript bindings
cargo test # run rust unit tests
pnpm test # run wasm integration tests
```

## Usage

Entrypoint `LeafRenderer` exposes `compileTemplates` and `renderTemplate` functions.

`compileTemplates` takes an array of JSON representations of Entities, like:

```ts
export type Entity<T> = {
  [key: ComponentId]: T
}
```

It expects each entity to have a `template:01JVK339CW6Q67VAMXCA7XAK7D` key, mapping to a `TemplateSource` object that looks like:

```ts
export interface TemplateSource {
  name: string
  source: string
  components: ComponentId[]
}
```

and returns a `CompileResult`.

```ts
export type CompileResult =
  | { type: "Success" }
  | { type: "Error"; error: CompileError }
```

`renderTemplate` takes a template name, a context `any`, and returns a `RenderResult`.

```ts
export type RenderResult =
  | { type: "Success"; result: string }
  | { type: "Error"; error: RenderError }
```

## Registering Components

Components are registered with the `register_component` function.

```ts
register_component(json.as_ptr(), json.len());
```

The JSON is a tuple of `(component_id, component_json)` where the JSON is JSON Schema describing the component, and the `component_id` is a Leaf `ComponentId`. Eg:

```json
["button", {
  "type": "object",
  "properties": {
    "label": {"type": "string"},
    "url": {"type": "string"}
  },
  "required": ["label"]
}]
```

The current approach is to assume that all component schemas contain a top-level Object. The valid namespace for the context is then the union of the properties of all the component Objects listed for that template. The alternative would be for users to write out the full component ID in the template, or to add in some aliasing feature.
