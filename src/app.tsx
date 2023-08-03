

import './app.css'

import RookSvg from './assets/rook.svg'
import KnightSvg from './assets/knight.svg'
import BishopSvg from './assets/bishop.svg'
import KingSvg from './assets/king.svg'
import QueenSvg from './assets/queen.svg'
import PawnSvg from './assets/pawn.svg'
import { Board, Piece, Row, locationStr, rankStr, } from './helpers'
import { atom, useAtom, useAtomValue, useSetAtom } from 'jotai'
import { boardAtom, logAtom, inputAtom, allMovesAtom, possibleInputsAtom, validInputSubstrAtom, inputIsCompleteAtom, validStartSquaresAtom, startIsCompleteAtom, validEndSquaresAtom, endIsCompleteAtom } from './state'
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

function FileRankHint(props: { location: string }) {
  let validStartSquares = useAtomValue(validStartSquaresAtom)
  let startIsComplete = useAtomValue(startIsCompleteAtom)

  let validEndSquares = useAtomValue(validEndSquaresAtom)
  let endIsComplete = useAtomValue(endIsCompleteAtom)

  let isStart = false
  let isEnd = false

  if (startIsComplete) {
    if (validStartSquares.has(props.location)) {
      isStart = true
    } else if (validEndSquares.has(props.location)) {
      isEnd = true
    }
  } else {
    if (validStartSquares.has(props.location)) {
      isStart = true
    }
  }

  if (!isStart && !isEnd) {
    return <></>
  }

  let className = ''
  if (isStart && startIsComplete) {
    className = 'is-start'
  }
  if (isEnd && endIsComplete) {
    className = 'is-end'
  }

  return (
    <div className={`square square-hint ${className}`}>
      <span>
        {props.location}
      </span>
    </div>
  )
}

function Square(props: { piece: Piece, file: number, rank: number }) {
  let colorClass = (props.file + props.rank) % 2 === 0 ? 'light' : 'dark'
  let location = locationStr(props.file, props.rank)

  let { piece } = props

  let pieceEl = <></>
  if (piece !== ' ') {
    pieceEl = <PieceComponent piece={piece} />
  }

  return (
    <div className={`square ${colorClass}`}>
      {pieceEl}
      <FileRankHint location={location} />
    </div>
  )
}


function BoardComponent(props: { board: Board }) {
  return (
    <div className="board">
      {props.board.map((row, inverseRank) => {
        let rank = 7 - inverseRank
        return <>
          <div className="row">
            {row.map((piece, file) => {
              return <Square piece={piece} file={file} rank={rank} key={file} />
            })}
            {/* <div className="square rank-label">{rankStr(rank)}</div> */}
          </div>
        </>
      })}

      {/* <div className="row file-labels">
        {['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'].map((file) => {
          return <div className="square file-label" key={file}>{file}</div>
        })}
        <div className="square rank-label"></div>
      </div> */}
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
