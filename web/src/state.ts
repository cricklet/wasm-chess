import { SetStateAction, atom } from "jotai";
import { Board, boardFromFen, } from "./helpers";
import * as wasm from "./wasm-bindings";

interface GameState {
    start: string,
    moves: string[],
}

let _wasmUci: ReturnType<typeof wasm.syncWasmUci> | undefined = undefined
function wasmUci(): ReturnType<typeof wasm.syncWasmUci> {
    if (!_wasmUci) {
        _wasmUci = wasm.syncWasmUci()
    }
    return _wasmUci
}

export const atomGame = atom<GameState>({
    start: 'startpos',
    moves: [],
})

export function performMove(move: string, game: GameState): GameState {
    return {
        ...game,
        moves: [...game.moves, move as string],
    }
}

export const atomBoard = atom<Board>(get => {
    let state = get(atomGame)
    wasmUci().setPosition(state.start, state.moves)
    let currentFen = wasmUci().currentFen()

    return boardFromFen(currentFen)
})

export const atomInput = atom<string>('')

export const atomLegalMoves = atom<string[]>(get => {
    get(atomBoard)
    return wasmUci().possibleMoves()
})

export const atomLegalStarts = atom<Set<string>>(get => {
    let allMoves = get(atomLegalMoves)
    return new Set(allMoves.map(move => move.slice(0, 2)))
})

export const atomCompleteMovesMatchingInput = atom<string[]>(get => {
    let input = get(atomInput)
    let allMoves = get(atomLegalMoves)
    return allMoves.filter(move => move.startsWith(input))
})

export const atomValidPortionOfInput = atom<string>(get => {
    let input = get(atomInput)
    let allMoves = get(atomLegalMoves)

    let inputToValidSubstr = ''
    for (let i = 0; i < input.length; i++) {
        let substr = input.slice(0, i + 1)
        let possibleInputs = allMoves.filter(move => move.startsWith(substr))
        if (possibleInputs.length > 0) {
            inputToValidSubstr = substr
        }
    }

    return inputToValidSubstr
})

export const atomValidStartsForInput = atom<Set<string>>(get => {
    let moves = get(atomCompleteMovesMatchingInput)
    return new Set(moves.map(input => input.slice(0, 2)))
})

export const atomStartFromInput = atom<string | undefined>(get => {
    let input = get(atomInput)
    let starts = get(atomValidStartsForInput)

    return starts.size === 1 && input.length >= 2 ? input.slice(0, 2) : undefined
})

export const atomValidEndsForInput = atom<Set<string>>(get => {
    let moves = get(atomCompleteMovesMatchingInput)
    return new Set(moves.map(input => input.slice(2, 4)))
})

export const atomEndFromInput = atom<string | undefined>(get => {
    let input = get(atomInput)
    let ends = get(atomValidEndsForInput)

    return ends.size === 1 && input.length >= 4 ? input.slice(2, 4) : undefined
})

export function finalizeMove(input: string, allMoves: string[]): string | undefined {
    if (input.length < 4) {
        return undefined
    }

    if (allMoves.includes(input)) {
        return input
    }

    if (allMoves.includes(input + 'q')) {
        return input + 'q'
    }

    return undefined
}

export const atomInputIsLegal = atom<boolean>(get => {
    let input = get(atomInput)
    let allMoves = get(atomLegalMoves)

    return finalizeMove(input, allMoves) !== undefined
})

export const logAtom = atom<string[]>([])