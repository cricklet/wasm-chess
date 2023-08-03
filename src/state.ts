import { atom } from "jotai";
import { Board, boardFromFen,  } from "./helpers";
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

export const validStartSquaresAtom = atom<Set<string>>(get => {
    let possibleInputs = get(possibleInputsAtom)
    return new Set(possibleInputs.map(input => input.slice(0, 2)))
})

export const startIsCompleteAtom = atom<boolean>(get => {
    let input = get(inputAtom)
    let validStartSquares = get(validStartSquaresAtom)

    return validStartSquares.size === 1 && input.length >= 2
})

export const validEndSquaresAtom = atom<Set<string>>(get => {
    let possibleInputs = get(possibleInputsAtom)
    return new Set(possibleInputs.map(input => input.slice(2, 4)))
})

export const endIsCompleteAtom = atom<boolean>(get => {
    let input = get(inputAtom)
    let validEndSquares = get(validEndSquaresAtom)

    return validEndSquares.size === 1 && input.length >= 4
})

export const inputIsCompleteAtom = atom<boolean>(get => {
    let input = get(inputAtom)
    let inputs = get(possibleInputsAtom)
    return inputs.filter(possibleInput => possibleInput === input).length > 0
})

export const logAtom = atom<string[]>([])