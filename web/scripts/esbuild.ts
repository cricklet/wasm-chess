
import * as esbuild from 'esbuild'
import svgr from 'esbuild-plugin-svgr'
import { wasmLoader } from 'esbuild-plugin-wasm'
import { execSync } from 'child_process'
import { argv } from 'process'

function getOutputOfCommand(command: string) {
  let output: Buffer | undefined = undefined
  try {
    output = execSync(command)
    return output.toString()
  } catch (error: any) {
    console.log(error)
    console.log("sdtout", error.stderr.toString())
    console.log("sdterr", error.stderr.toString())
    process.exit(1)
  }
}

function getArgs(): Set<string> {
  let args = argv.slice(2)
  let argsSet: Set<string> = new Set(args)

  let key: string | undefined = undefined

  for (let i = 0; i < args.length; i++) {
    let arg = args[i]
    if (arg.indexOf('--') === 0) {
      key = arg.slice(2)
      argsSet.add(key)
    } else {
      if (key === undefined) {
        console.error('args must be specified with --key1 value1 value2 --key2 value3')
        console.error('invalid args: ' + args.join(' '))
        process.exit(1)
      }
    }
  }

  return argsSet
}

let i = 0;

async function main() {
  let argsSet = getArgs()
  
  try {
    let shouldClear = false;
    let endMessage: string | undefined = undefined;

    let clearPlugin = {
      name: 'clear',
      setup: (build: any) => {
        build.onStart(() => {
          if (shouldClear) {
            console.clear()
          }
        })
        build.onEnd(() => {
          if (endMessage) {
            console.log(endMessage, '(' + i++ + ')')
          }
        })
      },
    }

    let ctx = await esbuild.context({
      entryPoints: [
        'src/entry.tsx',
        'src/entry.test.ts',
        'src/worker/js-worker-for-testing.ts',
        'src/worker/wasm-worker-for-testing.ts',
      ],
      bundle: true,
      sourcemap: true,
      platform: 'browser',
      format: 'esm',
      outdir: 'public/build/',
      plugins: [
        svgr({
          plugins: ['@svgr/plugin-jsx'],
        }),
        wasmLoader(),
        clearPlugin
      ],
    })

    let shouldDispose = true
    
    if (argsSet.has('watch')) {
      await ctx.watch()
      shouldDispose = false
    }

    if (argsSet.has('clear')) {
      shouldClear = true;
    }

    if (argsSet.has('serve')) {
      let result = await ctx.serve({
        servedir: 'public',
      })
      endMessage = `serving on ${result.host}:${result.port}`
      shouldDispose = false
    }

    if (shouldDispose) {
      await ctx.dispose()
    }

  } catch (e) {
    console.error(e)
    process.exit(1)
  }
}

main()