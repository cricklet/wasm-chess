import { describe, it } from "mocha";
import { boardFromFen, boardString } from "./helpers";
import { assert } from "chai";

describe("helpers boardFromFen", () => {
    it("should return the starting position", () => {
        let board = boardFromFen('startpos')
        assert.strictEqual(boardString(board),
            'rnbqkbnr\n' +
            'pppppppp\n' +
            '        \n' +
            '        \n' +
            '        \n' +
            '        \n' +
            'PPPPPPPP\n' +
            'RNBQKBNR'
        );
    });
});