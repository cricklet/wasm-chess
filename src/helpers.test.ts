
import { boardFromFen, boardString } from "./helpers";

describe('helpers.test.ts', function () {
    it('boardFromFen: should return the starting position', function () {
        let board = boardFromFen('startpos')
        expect(boardString(board)).toEqual(
            'rnbqkbnr\n' +
            'pppppppp\n' +
            '        \n' +
            '        \n' +
            '        \n' +
            '        \n' +
            'PPPPPPPP\n' +
            'RNBQKBNR'
        );
    })
})
