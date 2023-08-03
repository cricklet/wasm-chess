
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

function getArgs() {
  let args = argv.slice(2)
  let argsMap: { [key: string]: Array<string> } = {}

  let key: string | undefined = undefined

  for (let i = 0; i < args.length; i++) {
    let arg = args[i]
    if (arg.indexOf('--') === 0) {
      key = arg.slice(2)
    } else {
      if (key === undefined) {
        console.error('args must be specified with --key1 value1 value2 --key2 value3')
        console.error('invalid args: ' + args.join(' '))
        process.exit(1)
      }

      let val = args[i]
      if (key in argsMap) {
        argsMap[key].push(val)
      } else {
        argsMap[key] = [val]
      }
    }

  }

  return argsMap
}

let args = getArgs()
let linkedWasmPackages = args['link'] || []
for (let linkedWasmPackage of linkedWasmPackages) {
  let isLinked = getOutputOfCommand('ls node_modules')
    .split('\n')
    .filter((line) => line.indexOf(linkedWasmPackage) > -1)
    .length > 0

  if (!isLinked) {
    console.log(`package ${linkedWasmPackage} is not linked`)
    console.log(`either install via 'yarn install' or link via 'yarn link'`)
    process.exit(1)
  }
}

async function main() {
  try {
    let host = '?'
    let port: number | string = '?'

    let clearPlugin = {
      name: 'clear',
      setup: (build: any) => {
        build.onStart(() => {
          console.clear()
        })
        build.onEnd(() => {
          console.log(`Serving on http://${host}:${port}`)
        })
      },
    }

    let ctx = await esbuild.context({
      entryPoints: ['src/entry.tsx', 'src/entry.test.ts'],
      bundle: true,
      sourcemap: true,
      platform: 'browser',
      format: 'esm',
      outdir: 'public/build/',
      // define: {
      //   'process.stdout': '{}',
      //   'process.env.NODE_ENV': '"development"',
      //   'process.env.NODE_DEBUG': '"development"',
      // },
      plugins: [
        svgr({
          plugins: ['@svgr/plugin-jsx'],
        }),
        wasmLoader(),
        clearPlugin
      ],
    })
    await ctx.watch()

    let result = await ctx.serve({
      servedir: 'public',
    })

    host = result.host
    port = result.port

  } catch (e) {
    console.error(e)
    process.exit(1)
  }
}

main()