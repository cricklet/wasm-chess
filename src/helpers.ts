
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
