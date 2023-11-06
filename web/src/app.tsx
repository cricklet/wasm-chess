

import './app.css'

import { atom, useAtom, useAtomValue, useSetAtom } from 'jotai'
import { isValidElement, useEffect, useState } from 'react'
import BishopSvg from './assets/bishop.svg'
import KingSvg from './assets/king.svg'
import KnightSvg from './assets/knight.svg'
import PawnSvg from './assets/pawn.svg'
import QueenSvg from './assets/queen.svg'
import RookSvg from './assets/rook.svg'
import { Board, Piece, Row, locationStr, rankStr, } from './helpers'
import { GameState, atomBoard, atomCompleteMovesMatchingInput, atomEndFromInput, atomEngineControlsBlack, atomEngineControlsWhite, atomFen, atomGame, atomInput, atomInputIsLegal as atomInputIsLegalMove, atomLastMove, atomLegalMoves, atomLegalStarts, atomStartFromInput, atomValidEndsForInput as atomValidEndsMatchingInput, atomValidPortionOfInput, atomValidStartsForInput as atomValidStartsMatchingInput, atomWhiteToMove, finalizeMove, logAtom, moveContainsLocation, performMove } from './state'
import * as wasm from './wasm-bindings'
import { useNavigate, useParams, useSearchParams } from 'react-router-dom'


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

enum SquarePotential {
  None,
  Start,
  End,
}

enum SquareHighlight {
  None,
  StartAvailable,
  StartComplete,
  EndAvailable,
  EndComplete,
  LastMove,
}

function computeHighlightAndPotential(location: string): [SquarePotential, SquareHighlight] {
  let allStarts = useAtomValue(atomLegalStarts)

  let matchingStarts = useAtomValue(atomValidStartsMatchingInput)
  let matchingEnds = useAtomValue(atomValidEndsMatchingInput)

  let startFromInput = useAtomValue(atomStartFromInput)
  let endFromInput = useAtomValue(atomEndFromInput)

  let lastMove = useAtomValue(atomLastMove)

  let isStart = allStarts.has(location)
  let isEnd = startFromInput !== undefined && matchingEnds.has(location)

  let squareHighlight = SquareHighlight.None
  if (isEnd) {
    if (endFromInput === location) {
      squareHighlight = SquareHighlight.EndComplete
    } else {
      squareHighlight = SquareHighlight.EndAvailable
    }
  } else if (isStart) {
    if (startFromInput === location) {
      squareHighlight = SquareHighlight.StartComplete
    } else if (matchingStarts.has(location)) {
      squareHighlight = SquareHighlight.StartAvailable
    }
  } else if (moveContainsLocation(lastMove, location)) {
    squareHighlight = SquareHighlight.LastMove
  }

  let squarePotential = SquarePotential.None
  if (isStart) {
    squarePotential = SquarePotential.Start
  } else if (isEnd) {
    squarePotential = SquarePotential.End
  }

  return [squarePotential, squareHighlight]
}

function SquareHint(props: { highlight: SquareHighlight, location: string }) {
  const { highlight, location } = props

  if (highlight === SquareHighlight.None) {
    return <></>
  }

  let className = ''
  if (highlight === SquareHighlight.StartComplete) {
    className = 'is-start'
  } else if (highlight === SquareHighlight.EndComplete) {
    className = 'is-end'
  } else if (highlight === SquareHighlight.LastMove) {
    className = 'is-last-move'
  }

  return <div className={`square square-hint ${className}`}>
    <span>
      {location}
    </span>
  </div>
}

function Square(props: { piece: Piece, file: number, rank: number }) {
  let colorClass = (props.file + props.rank) % 2 === 0 ? 'light' : 'dark'
  let location = locationStr(props.file, props.rank)

  let { piece } = props
  let [potential, highlight] = computeHighlightAndPotential(location)

  let pieceEl = <></>
  if (piece !== ' ') {
    pieceEl = <PieceComponent piece={piece} />
  }

  let allMoves = useAtomValue(atomLegalMoves)
  let [input, setInput] = useAtom(atomInput)
  let setGame = useSetAtom(atomGame);

  const onClick = () => {
    if (potential === SquarePotential.None) {
      setInput("")
    } else if (potential === SquarePotential.Start) {
      setInput(location)
    } else if (potential === SquarePotential.End) {
      let move = finalizeMove(input.slice(0, 2) + location, allMoves)
      if (move === undefined) {
        throw new Error(`move ${input.slice(0, 2) + location} is not legal`)
      } else {
        setGame((game) => performMove(move as string, game))
        setInput("")
      }
    }
  }

  const clickableClass = potential !== SquarePotential.None ? '' : 'clickable'

  return (
    <div
      className={`square ${colorClass} ${clickableClass}`}
      onClick={onClick}>
      {pieceEl}
      <SquareHint highlight={highlight} location={location} />
    </div>
  )
}


function BoardComponent(props: { board: Board }) {
  return (
    <div className="board">
      {props.board.map((row, inverseRank) => {
        let rank = 7 - inverseRank
        return (
          <div className="row" key={rank}>
            {row.map((piece, file) => {
              return <Square piece={piece} file={file} rank={rank} key={file} />
            })}
            {/* <div className="square rank-label">{rankStr(rank)}</div> */}
          </div>
        )
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

function EngineOptions() {
  let [engineControlsWhite, setEngineControlsWhite] = useAtom(atomEngineControlsWhite)
  let [engineControlsBlack, setEngineControlsBlack] = useAtom(atomEngineControlsBlack)

  let value = "-"
  if (engineControlsWhite && !engineControlsBlack) {
    value = "w"
  } else if (!engineControlsWhite && engineControlsBlack) {
    value = "b"
  } else if (engineControlsWhite && engineControlsBlack) {
    value = "wb"
  }

  return (
    <select value={value} onChange={(e) => {
      let value = e.target.value
      if (value === "w") {
        setEngineControlsWhite(true)
        setEngineControlsBlack(false)
      } else if (value === "b") {
        setEngineControlsWhite(false)
        setEngineControlsBlack(true)
      } else if (value === "wb") {
        setEngineControlsWhite(true)
        setEngineControlsBlack(true)
      } else if (value === "-") {
        setEngineControlsWhite(false)
        setEngineControlsBlack(false)
      }
    }}>
      <option value="w">Engine plays white</option>
      <option value="b">Engine plays black</option>
      <option value="wb">Engine vs engine</option>
      <option value="-">Player vs player</option>
    </select>
  )
}

function InputComponent() {
  let input = useAtomValue(atomInput)
  let isLegal = useAtomValue(atomInputIsLegalMove)

  let inputToValidSubstr = useAtomValue(atomValidPortionOfInput)
  let invalidInputSuffix = input.slice(inputToValidSubstr.length)

  return (
    <div className="input">
      {isLegal ? (
        <span className="complete-input">{input}</span>
      ) : (
        <>
          <span className="valid-input">{inputToValidSubstr}</span>
          <span className="invalid-input">{invalidInputSuffix}</span>
        </>
      )}
    </div>
  )
}

let worker = await wasm.searchWorker()

function App() {
  let board = useAtomValue(atomBoard)
  let [game, setGame] = useAtom(atomGame)
  let [fen] = useAtom(atomFen)
  let setLog = useSetAtom(logAtom)
  let [input, setInput] = useAtom(atomInput)
  let allMoves = useAtomValue(atomLegalMoves)

  let [engineControlsWhite, setEngineControlsWhite] = useAtom(atomEngineControlsWhite)
  let [engineControlsBlack, setEngineControlsBlack] = useAtom(atomEngineControlsBlack)

  let whiteToMove = useAtomValue(atomWhiteToMove)

  let [params, setParams] = useSearchParams()
  let [initialized, setInitialized] = useState(false)
  useEffect(() => {
    if (initialized) {
      return
    }
    setInitialized(true)

    let start = params.get('fen')
    if (start !== null) {
      setGame({ start, moves: [] })
    }

    let white = params.get('white')
    if (white === 'engine') {
      setEngineControlsWhite(true)
    } else if (white === 'player') {
      setEngineControlsWhite(false)
    }

    let black = params.get('black')
    if (black === 'engine') {
      setEngineControlsBlack(true)
    } else if (black === 'player') {
      setEngineControlsBlack(false)
    }
  }, [params])

  useEffect(() => {
    if (!initialized) {
      return
    }

    let newParams = new URLSearchParams()
    if (game.start !== undefined) {
      newParams.set('fen', fen)
    }
    if (engineControlsWhite) {
      newParams.set('white', 'engine')
    } else {
      newParams.set('white', 'player')
    }
    if (engineControlsBlack) {
      newParams.set('black', 'engine')
    } else {
      newParams.set('black', 'player')
    }
    setParams(newParams)
  }, [game, engineControlsWhite, engineControlsBlack])

  useEffect(() => {
    let cleanup = wasm.listen((line: string) => {
      setLog((log) => [...log, line])
    })
    return cleanup
  }, [])

  useEffect(() => {
    async function think() {
      let start = game.start
      let moves = [...game.moves]

      if (whiteToMove && !engineControlsWhite) {
        return
      }

      if (!whiteToMove && !engineControlsBlack) {
        return
      }

      let bestMove = await worker.search(start, moves)
      if (bestMove.indexOf("none") !== -1) {
        return
      }

      setGame((_) => performMove(bestMove, { start, moves }))
    }
    think()
  }, [game, engineControlsBlack, engineControlsWhite])

  useEffect(() => {
    document.onkeydown = (event) => {
      if (event.key === 'Enter') {
        let move = finalizeMove(input, allMoves);
        if (move !== undefined) {
          setGame((game) => performMove(move as string, game))
          setInput('')
          return
        }
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
      } else if (event.key === 'Enter') {
        let move = finalizeMove(input, allMoves);
        if (move !== undefined) {

        }
      } else if (event.key.length > 1) {
        return
      }

      let newInput = input + event.key
      setInput(newInput)
    }
    return () => {
      document.onkeydown = null
    }
  }, [input, setInput])

  return (
    <div className="app">
      <BoardComponent board={board} />
      <InputComponent />
      <div className="log">
      </div>
      <EngineOptions />
    </div>
  )
}

export default App
