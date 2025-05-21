// A bunch of imports expected because valico is a bit of a pain

const decoder = new TextDecoder()
let memory: WebAssembly.Memory
export const wasmBindgenImports = {
  __wbindgen_placeholder__: {
    __wbindgen_throw: function (ptr: number, len: number) {
      throw new Error(decoder.decode(new Uint8Array(memory.buffer, ptr, len)))
    },
    __wbg_getRandomValues_3d90134a348e46b3: function () {
      // Return a no-op function since we don't need actual random values
      return function () {}
    },
    __wbindgen_object_drop_ref: function () {
      // No-op since we're not actually managing object references
    },
    __wbindgen_describe: function () {
      // No-op since we don't need type information
    },
  },
  __wbindgen_externref_xform__: {
    __wbindgen_externref_table_grow: function () {
      return 0
    },
    __wbindgen_externref_table_set_null: function () {},
    __wbindgen_externref_table_set: function () {
      return 0
    },
    __wbindgen_externref_table_get: function () {
      return null
    },
  },
}
