
console.log('loading wasm tests')

import * as wasm from './wasm-bindings'

describe('wasm-bindings.test.ts', function () {
    it('works', function () {
        wasm.loadWasm()
        expect(1).toEqual(1)
    })
})
