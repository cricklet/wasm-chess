import { atom } from "jotai";
import { Board, boardFromFen, fileFromStr } from "./helpers";
import * as wasm from "./wasm-bindings";

interface GameState {
    start: string,
    moves: string[],
}

export const gameAtom = atom<GameState>({
    start: 'startpos',
    moves: ['e2e4'],
})

export const boardAtom = atom<Board>(get => {
    let state = get(gameAtom)
    wasm.setPosition(state.start, state.moves)
    let currentFen = wasm.currentFen()

    return boardFromFen(currentFen)
})

export const inputAtom = atom<string>('')

export const allMovesAtom = atom<string[]>(get => {
    get(boardAtom)
    return wasm.possibleMoves()
})

export const possibleInputsAtom = atom<string[]>(get => {
    let input = get(inputAtom)
    let allMoves = get(allMovesAtom)
    return allMoves.filter(move => move.startsWith(input))
})

export const inputIsValidAtom = atom<boolean>(get => {
    let input = get(inputAtom)
    let possibleInputs = get(possibleInputsAtom)
    return possibleInputs
        .filter(possibleInput => possibleInput.startsWith(input)).length > 0
})

export const validInputSubstrAtom = atom<string>(get => {
    let input = get(inputAtom)
    let allMoves = get(allMovesAtom)

    let validInputSubstr = ''
    for (let i = 0; i < input.length; i++) {
        let substr = input.slice(0, i + 1)
        let possibleInputs = allMoves.filter(move => move.startsWith(substr))
        if (possibleInputs.length > 0) {
            validInputSubstr = substr
        }
    }

    return validInputSubstr
})


export const inputIsCompleteAtom = atom<boolean>(get => {
    let input = get(inputAtom)
    let inputs = get(possibleInputsAtom)
    return inputs.filter(possibleInput => possibleInput === input).length > 0
})

export const logAtom = atom<string[]>([])