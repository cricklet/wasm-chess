
async function init() {
    console.log('initializing test worker')

    self.onmessage = async event => {
        let response = `message received by worker: ${event.data}`
        console.log(response)
        self.postMessage(response)
    }
}

init()