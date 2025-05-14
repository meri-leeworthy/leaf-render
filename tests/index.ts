export interface TemplateSource {
  name: string;
  source: string;
  components?: string[];
}

export interface CompileError {
  error_type: string;
  message: string;
  missing_dependencies?: string[];
}

export type CompileResult =
  | { type: "Success" }
  | { type: "Error"; error: CompileError };

export class Minijinja {
  private wasm: WebAssembly.Instance;
  private memory: WebAssembly.Memory;

  constructor(wasmModule: WebAssembly.Module) {
    this.memory = new WebAssembly.Memory({ initial: 10 });
    this.wasm = new WebAssembly.Instance(wasmModule, {
      env: {
        memory: this.memory,
      },
    });
  }

  compileTemplates(templates: TemplateSource[]): CompileResult {
    const input = JSON.stringify(templates);
    const inputPtr = this.allocateString(input);
    const outputPtr = this.allocateMemory(1024);

    try {
      const resultSize = (this.wasm.exports.compile_templates as Function)(
        inputPtr,
        input.length,
        outputPtr,
        1024
      );

      const result = this.readString(outputPtr, resultSize);
      return JSON.parse(result);
    } finally {
      this.freeMemory(inputPtr);
      this.freeMemory(outputPtr);
    }
  }

  renderTemplate(name: string, context: any): string {
    const namePtr = this.allocateString(name);
    const contextStr = JSON.stringify(context);
    const contextPtr = this.allocateString(contextStr);
    const outputPtr = this.allocateMemory(1024);

    try {
      const resultSize = (this.wasm.exports.render_template as Function)(
        namePtr,
        name.length,
        contextPtr,
        contextStr.length,
        outputPtr,
        1024
      );

      return this.readString(outputPtr, resultSize);
    } finally {
      this.freeMemory(namePtr);
      this.freeMemory(contextPtr);
      this.freeMemory(outputPtr);
    }
  }

  private allocateString(str: string): number {
    const encoder = new TextEncoder();
    const bytes = encoder.encode(str);
    const ptr = this.allocateMemory(bytes.length + 1);
    const view = new Uint8Array(this.memory.buffer);
    view.set(bytes, ptr);
    view[ptr + bytes.length] = 0;
    return ptr;
  }

  private readString(ptr: number, length: number): string {
    const view = new Uint8Array(this.memory.buffer);
    const bytes = view.slice(ptr, ptr + length);
    return new TextDecoder().decode(bytes);
  }

  private allocateMemory(size: number): number {
    return (this.wasm.exports.malloc as Function)(size);
  }

  private freeMemory(ptr: number): void {
    (this.wasm.exports.free as Function)(ptr);
  }
}
