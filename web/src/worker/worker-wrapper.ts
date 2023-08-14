
import { omit, prettyJson } from "../helpers"
import { SendToWorker, ReceiveFromWorkerResponse, ReceiveFromWorkerMessage, decodeReceiveFromWorker, ReceiveFromWorker, responseMatchesRequest, SendToWorkerWithResponse, isEmpty } from "./worker-types"

export async function createWorker(url: string) {
    let worker = new Worker(url)

    let listeners: Array<(e: ReceiveFromWorker) => void> = [
        (e: ReceiveFromWorker) => {
            if (isEmpty(e)) {
                return
            }

            let { name, kind, ...rest } = e
            if ('id' in rest) {
                rest = omit(rest, 'id')
            }
            if (name === 'log') {
                if ('msg' in rest) {
                    console.log('log (worker-wrapper.ts) =>', prettyJson(rest.msg))
                } else {
                    throw new Error('log message missing')
                }
            } else if (name === 'error') {
                console.error('error (worker-wrapper.ts) =>', prettyJson(rest))
            } else if (name === 'ready') {
                console.log('ready (worker-wrapper.ts, worker)')
            } else {
                console.log(kind, '(worker-wrapper.ts) =>', prettyJson(rest))
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