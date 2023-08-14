
import { indent_string, omit, prettyJson } from "../helpers"
import { SendToWorker, ReceiveFromWorkerResponse, ReceiveFromWorkerMessage, decodeReceiveFromWorker, ReceiveFromWorker, responseMatchesRequest, SendToWorkerWithResponse, isEmpty } from "./worker-types"

export async function createWorker(url: string) {
    let worker = new Worker(url)

    let listeners: Array<(e: ReceiveFromWorker) => void> = [
        (e: ReceiveFromWorker) => {
            if (isEmpty(e)) {
                return
            }

            if (e.name === 'log') {
                console.log('log (wasm worker) =>', prettyJson(e.msg))
            } else if (e.name === 'error') {
                console.error('error (wasm worker) =>', prettyJson(e.msg))
            } else if (e.name === 'ready') {
                console.log('ready (wasm worker)')
            } else if (e.name === 'uci') {
                if (e.output.length > 0) {
                    console.log('uci sync response (wasm worker)', indent_string('\n' + e.output, 2))
                }
            } else if (e.name === 'uci-flush-output') {
                if (e.output.length > 0) {
                    console.log('uci async response (wasm worker)', e.output)
                }
            } else {
                console.log(e.kind, '(wasm worker) =>', prettyJson(omit(e, 'name', 'kind', 'id')))
            }
        },
    ]

    worker.onmessage = (e) => {
        let data = decodeReceiveFromWorker(e.data)
        listeners.forEach((l) => l(data))
    }

    async function waitFor<T>(f: (e: ReceiveFromWorker) => T | undefined): Promise<T> {
        return new Promise((resolve) => {
            let callback = (e: ReceiveFromWorker) => {
                let t = f(e)
                if (t != null && t !== false) {
                    listeners = listeners.filter((l) => l !== callback)
                    resolve(t)
                }
            }

            listeners.push(callback)
        })
    }

    await waitFor(e => {
        return e.name === 'ready'
    })

    let responseId = 0;

    return {
        send: (data: SendToWorker) => {
            worker.postMessage(JSON.stringify(data))
        },
        sendWithResponse: async<
            S extends Omit<SendToWorkerWithResponse, "id">,
        >(data: S): Promise<ReceiveFromWorkerResponse & Pick<S, "name">> => {
            let id = responseId++
            let sent = { ...data, id } as SendToWorkerWithResponse

            worker.postMessage(JSON.stringify(sent))
            return await waitFor((e: ReceiveFromWorker): ReceiveFromWorkerResponse & Pick<S, "name"> | undefined => {
                if (responseMatchesRequest(sent, e)) {
                    return e
                }
            })
        },
        wait: waitFor,
        terminate: () => {
            worker.terminate()
        }
    }

}
