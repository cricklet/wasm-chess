import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './app'
import { loadWasmBindgen, searchWorker } from './wasm-bindings'
import { BrowserRouter, Router, createBrowserRouter } from 'react-router-dom'


new EventSource('/esbuild').addEventListener('change', () => location.reload())

const rootEl = document.getElementById('root') as HTMLElement
rootEl.style.filter = `hue-rotate(${Math.random() * 20}deg)`

const root = ReactDOM.createRoot(rootEl);

(async () => {
  await loadWasmBindgen()
  await searchWorker()

  root.render(
    <React.StrictMode>
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </React.StrictMode>
  );
})()
