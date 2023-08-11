import { WebToWorker, WorkerToWeb, decodeWorkerToWeb } from "./worker-types"

export async function createWorker(url: string) {
    let worker = new Worker(url)

    let listeners: Array<(e: WorkerToWeb) => void> = [
        (e: WorkerToWeb) => {
            if (e.kind === 'log') {
                console.log('> log:', ...e.msg)
            } else if (e.kind === 'error') {
                console.error('> err:', e.msg)
            } else {
                let {kind, ...rest} = e
                console.log('> ' + kind + ': ' + JSON.stringify(rest))
            }
        },
    ]

    worker.onmessage = (e) => {
        let data = decodeWorkerToWeb(e.data)
        listeners.forEach((l) => l(data))
    }

    async function waitFor<T>(f: (e: WorkerToWeb) => T | undefined): Promise<T> {
        return new Promise((resolve) => {
            let callback = (e: WorkerToWeb) => {
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
        return e.kind === 'ready'
    })

    return {
        send: (data: WebToWorker) => {
            worker.postMessage(JSON.stringify(data))
        },
        wait: waitFor,
        terminate: () => {
            worker.terminate()
        }
    }

}