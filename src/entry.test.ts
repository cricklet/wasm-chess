
describe('entry.test.ts', function () {
    it('works', function () {
        expect(1).toEqual(1)
    })
})

import './wasm-bindings.test'
import './helpers.test'

console.log('loaded tests')

const env = jasmine.getEnv();
env.execute();

new EventSource('/esbuild').addEventListener('change', () => location.reload())
