export interface TemplateSource {
  name: string
  source: string
  components?: string[]
}

export interface CompileError {
  error_type: string
  message: string
  missing_dependencies?: string[]
}

export type CompileResult =
  | { type: "Success" }
  | { type: "Error"; error: CompileError }

export class LeafRenderer {
  private wasm: WebAssembly.Instance
  private memory: WebAssembly.Memory
  private heapOffset = 1024

  constructor(wasmModule: WebAssembly.Module) {
    this.wasm = new WebAssembly.Instance(wasmModule, {
      env: {}, // no need to pass memory
    }) as any
    this.memory = this.wasm.exports.memory as WebAssembly.Memory
  }

  private get memoryBuffer(): Uint8Array {
    return new Uint8Array(this.memory.buffer)
  }

  private alloc(size: number): number {
    const ptr = this.heapOffset
    this.heapOffset += size
    this.ensureMemory(ptr + size)
    return ptr
  }

  private ensureMemory(byteLength: number) {
    const pagesNeeded = Math.ceil(byteLength / 65536)
    const currentPages = this.memory.buffer.byteLength / 65536
    if (pagesNeeded > currentPages) {
      this.memory.grow(pagesNeeded - currentPages)
    }
  }

  private writeStringToMemory(str: string): [number, number] {
    const encoder = new TextEncoder()
    const bytes = encoder.encode(str)
    const ptr = this.alloc(bytes.length)
    this.memoryBuffer.set(bytes, ptr)
    return [ptr, bytes.length]
  }

  private readString(ptr: number, length: number): string {
    const view = this.memoryBuffer.slice(ptr, ptr + length)
    return new TextDecoder().decode(view)
  }

  compileTemplates(templates: TemplateSource[]): CompileResult {
    const json = JSON.stringify(templates)
    const [inPtr, inLen] = this.writeStringToMemory(json)
    const outPtr = this.alloc(4096)

    const resultSize = (this.wasm.exports.compile_templates as Function)(
      inPtr,
      inLen,
      outPtr,
      4096
    )

    const result = this.readString(outPtr, resultSize)

    console.log("Result string:", result)
    console.log("Memory view:", [
      ...this.memoryBuffer.slice(outPtr, outPtr + resultSize),
    ])

    return JSON.parse(result)
  }

  renderTemplate(name: string, context: any): string {
    const [namePtr, nameLen] = this.writeStringToMemory(name)
    const contextStr = JSON.stringify(context)
    const [ctxPtr, ctxLen] = this.writeStringToMemory(contextStr)
    const outPtr = this.alloc(4096)

    const resultSize = (this.wasm.exports.render_template as Function)(
      namePtr,
      nameLen,
      ctxPtr,
      ctxLen,
      outPtr,
      4096
    )

    return this.readString(outPtr, resultSize)
  }
}
