
export type Piece = 'R' | 'N' | 'B' | 'K' | 'Q' | 'P' | 'r' | 'n' | 'b' | 'k' | 'q' | 'p' | ' ';
export type Row = [Piece, Piece, Piece, Piece, Piece, Piece, Piece, Piece];
export type Board = [Row, Row, Row, Row, Row, Row, Row, Row];

export function rankStr(rank: number): string {
    return (rank + 1).toString();
}

export function fileStr(file: number): string {
    return 'abcdefgh'[file];
}

export function locationStr(file: number, rank: number): string {
    return fileStr(file) + rankStr(rank);
}

export function boardFromFen(fen: string): Board {
    if (fen == 'startpos') {
        fen = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - - 0 1';
    }

    if (fen.startsWith('fen ')) {
        fen = fen.split('fen ')[1];
    }

    let board: Board = [
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    ];

    fen.split(' ')[0].split('/').map((row, rank) => {
        let file = 0;
        row.split('').map(piece => {
            let pieceAsInt = parseInt(piece);
            if (isNaN(pieceAsInt)) {
                board[rank][file] = piece as Piece;
                file++
            } else {
                file += pieceAsInt;
            }
        })
    });

    return board
}

export function boardString(board: Board): string {
    return [...board].map(row => row.join('')).join('\n');
}

export function resolveable<T>(): [promise: Promise<T>, resolve: (t: T) => void] {
    let resolve: (t: T) => void;
    let promise = new Promise<T>((r) => {
        resolve = r;
    });
    return [promise, resolve!];
}

export function indent_string(lines: string, indent: number): string {
    let s = ' '.repeat(indent);
    return lines.split('\n').map(line => s + line).join('\n');
}

export function prettyJson(json: any, indent?: number): string {
    if (!indent) {
        indent = 0;
    }

    // array
    if (Array.isArray(json)) {
        let result = "";
        for (let value of json) {
            result += "\n  " + value;
        }
        return result;
    }

    let result = "{" + "\n";

    for (let key in json) {
        let value = json[key];
        if (typeof value == 'string') {
            if (value.indexOf("\n") != -1) {
                result += "  " + key + ": " + "\n";
                result += indent_string(value, 4) + "\n";
            } else {
                result += "  " + key + ": " + value + "\n";
            }
        } else {
            result += "  " + key + ": " + value + "\n";
        }
    }

    return result;
}

export function omit<T, K extends keyof T>(obj: T, ...keys: K[]): Omit<T, K> {
    let result: any = {};
    for (let key in obj) {
        if (!keys.includes(key as any)) {
            result[key] = obj[key];
        }
    }
    return result;
}
