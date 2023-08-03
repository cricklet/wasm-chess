import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './app'
import * as helpers from './helpers'
import * as wasm from './wasm-bindings'

import { create } from 'zustand'

interface UciState {
  fen: string
}

const useChessStore = create<UciState>((set) => ({
  fen: 'startpos',
  setFen: (fen: string) => set((state) => ({ fen })),
}))

new EventSource('/esbuild').addEventListener('change', () => location.reload())

const rootEl = document.getElementById('root') as HTMLElement
rootEl.style.filter = `hue-rotate(${Math.random() * 20}deg)`

const root = ReactDOM.createRoot(rootEl);

root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);

wasm.greet()