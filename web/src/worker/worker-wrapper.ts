import {  SendToWorker, ReceiveFromWorkerResponse, SendToWorkerWithResponse, ReceiveFromWorkerMessage, decodeReceiveFromWorker, ReceiveFromWorker, responseMatchesRequest } from "./worker-types"

export async function createWorker(url: string) {
    let worker = new Worker(url)

    let listeners: Array<(e: ReceiveFromWorker) => void> = [
        (e: ReceiveFromWorker) => {
            if (e.name === 'log') {
                console.log('> log:', ...e.msg)
            } else if (e.name === 'error') {
                console.error('> err:', e.msg)
            } else {
                let {name: kind, ...rest} = e
                console.log('> ' + kind + ': ' + JSON.stringify(rest))
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
        > (data: S): Promise<ReceiveFromWorkerResponse & Pick<S, "name">> => {
            let id = responseId++
            let sent: SendToWorkerWithResponse = { ...data, id }

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