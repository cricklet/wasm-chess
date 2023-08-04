import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './app'


new EventSource('/esbuild').addEventListener('change', () => location.reload())

const rootEl = document.getElementById('root') as HTMLElement
rootEl.style.filter = `hue-rotate(${Math.random() * 20}deg)`

const root = ReactDOM.createRoot(rootEl);

root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
