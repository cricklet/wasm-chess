

import './app.css'

import RookSvg from './assets/rook.svg'
import KnightSvg from './assets/knight.svg'
import BishopSvg from './assets/bishop.svg'
import KingSvg from './assets/king.svg'
import QueenSvg from './assets/queen.svg'
import PawnSvg from './assets/pawn.svg'
import { Board, FileRank, Piece, Rank, Row, fileStr, invert, rankStr } from './helpers'
import { atom, useAtom, useAtomValue, useSetAtom } from 'jotai'
import { boardAtom, logAtom, inputAtom, allMovesAtom, possibleInputsAtom, validInputSubstrAtom, inputIsCompleteAtom } from './state'
import { isValidElement, useEffect } from 'react'
import * as wasm from './wasm-bindings'

function PieceComponent(props: { piece: Piece }) {
  let { piece } = props

  let pieceEl = <></>

  let svgProps = {
    className: 'piece',
    fill: piece === piece.toLowerCase() ? 'var(--dark-piece)' : 'var(--light-piece)',
  }

  switch (piece.toLowerCase()) {
    case 'r':
      pieceEl = <RookSvg {...svgProps} />
      break
    case 'n':
      pieceEl = <KnightSvg {...svgProps} />
      break
    case 'b':
      pieceEl = <BishopSvg {...svgProps} />
      break
    case 'k':
      pieceEl = <KingSvg {...svgProps} />
      break
    case 'q':
      pieceEl = <QueenSvg {...svgProps} />
      break
    case 'p':
      pieceEl = <PawnSvg {...svgProps} />
      break

    default:
      throw new Error(`unknown piece: ${piece}`)
  }

  return pieceEl
}

function Square(props: { piece: Piece, fileRank: FileRank }) {
  let colorClass = (props.fileRank[0] + props.fileRank[1]) % 2 === 0 ? 'light' : 'dark'

  let { piece } = props

  let pieceEl = <></>
  if (piece !== ' ') {
    pieceEl = <PieceComponent piece={piece} />
  }
  return (
    <div className={`square ${colorClass}`}>
      <div className="square-hint">
        {fileStr(props.fileRank[0])}
        {rankStr(props.fileRank[1])}
      </div>

      {pieceEl}
    </div>
  )
}

function RowComponent(props: { row: Row, rank: Rank }) {
  return (
    <div className="row">
      {props.row.map((piece, file) => {
        return <Square piece={piece} fileRank={[file, props.rank]} key={file} />
      })}
    </div>
  )
}

function BoardComponent(props: { board: Board }) {
  return (
    <div className="board">
      {props.board.map((row, inverseRank) => {
        let rank = invert(inverseRank)
        return <RowComponent row={row} rank={rank} key={rank} />
      })}
    </div>
  )
}

function InputComponent() {
  let input = useAtomValue(inputAtom)
  let isComplete = useAtomValue(inputIsCompleteAtom)

  let validInputSubstr = useAtomValue(validInputSubstrAtom)
  let invalidInputSuffix = input.slice(validInputSubstr.length)

  return (
    <div className="input">
      {isComplete ? (
        <span className="complete-input">{input}</span>
      ) : (
        <>
          <span className="valid-input">{validInputSubstr}</span>
          <span className="invalid-input">{invalidInputSuffix}</span>
        </>
      )}
    </div>
  )
}

function App() {
  let board = useAtomValue(boardAtom)
  let setLog = useSetAtom(logAtom)
  let possibleInputs = useAtomValue(possibleInputsAtom)
  let [input, setInput] = useAtom(inputAtom)

  useEffect(() => {
    let cleanup = wasm.listen((line: string) => {
      setLog((log) => [...log, line])
    })
    return cleanup
  }, [])

  useEffect(() => {
    document.onkeydown = (event) => {
      if (event.key === 'Enter') {
        return
      } else if (event.key === 'Backspace') {
        if (event.getModifierState('Control')) {
          setInput('')
        } else {
          setInput((input) => input.slice(0, -1))
        }
        return
      } else if (event.key === 'Escape') {
        setInput('')
        return
      } else if (event.key.length > 1) {
        return
      }

      let newInput = input + event.key
      setInput(newInput)
    }
    return () => {
      document.onkeydown = null
    }
  }, [input, possibleInputs, setInput])

  return (
    <div className="app">
      <BoardComponent board={board} />
      <InputComponent />
      <div className="log">
      </div>
    </div>
  )
}

export default App
