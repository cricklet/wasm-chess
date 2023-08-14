
async function init() {
    self.onmessage = async event => {
        let response = `message received by worker: ${event.data}`
        self.postMessage(response)
    }
}

init()