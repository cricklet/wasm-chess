

import './App.css';

import RookSvg from './assets/rook.svg';
import KnightSvg from './assets/knight.svg';
import BishopSvg from './assets/bishop.svg';
import KingSvg from './assets/king.svg';
import QueenSvg from './assets/queen.svg';
import PawnSvg from './assets/pawn.svg';

import * as wasm from 'crab-chess'
wasm.greet()
wasm.process('d\n');
wasm.process('go perft 1\n');

type Piece = 'R' | 'N' | 'B' | 'K' | 'Q' | 'P' | 'r' | 'n' | 'b' | 'k' | 'q' | 'p' | ' ';
type Row = [Piece, Piece, Piece, Piece, Piece, Piece, Piece, Piece];
type Board = [Row, Row, Row, Row, Row, Row, Row, Row];

type File = number;
type Rank = number;
type FileRank = [File, Rank];

function invert(rank: Rank): Rank {
  return 7 - rank;
}

function PieceComponent(props: { piece: Piece }) {
  let { piece } = props;

  let pieceEl = <></>;

  let svgProps = {
    className: 'piece',
    fill: piece === piece.toLowerCase() ? 'var(--dark-piece)' : 'var(--light-piece)',
  };

  switch (piece.toLowerCase()) {
    case 'r':
      pieceEl = <RookSvg {...svgProps} />;
      break;
    case 'n':
      pieceEl = <KnightSvg {...svgProps} />;
      break;
    case 'b':
      pieceEl = <BishopSvg {...svgProps} />;
      break;
    case 'k':
      pieceEl = <KingSvg {...svgProps} />;
      break;
    case 'q':
      pieceEl = <QueenSvg {...svgProps} />;
      break;
    case 'p':
      pieceEl = <PawnSvg {...svgProps} />;
      break;

    default:
      throw new Error(`unknown piece: ${piece}`);
  }

  return pieceEl;
}

function Square(props: { piece: Piece, fileRank: FileRank }) {
  let colorClass = (props.fileRank[0] + props.fileRank[1]) % 2 === 0 ? 'light' : 'dark';

  let { piece } = props;

  let pieceEl = <></>;
  if (piece !== ' ') {
    pieceEl = <PieceComponent piece={piece} />;
  }
  return (
    <div className={`square ${colorClass}`}>
      {pieceEl}
    </div>
  );
}

function RowComponent(props: { row: Row, rank: Rank }) {
  return (
    <div className="row">
      {props.row.map((piece, file) => {
        return <Square piece={piece} fileRank={[file, props.rank]} key={file} />
      })}
    </div>
  );
}

function BoardComponent(props: { board: Board }) {
  return (
    <div className="board">
      {props.board.map((row, inverseRank) => {
        let rank = invert(inverseRank);
        return <RowComponent row={row} rank={rank} key={rank} />
      })}
    </div>
  );
}

function App() {
  let board: Board = [
    ['r', 'n', 'b', 'k', 'q', 'b', 'n', 'r'],
    ['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p'],
    [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    ['P', 'P', 'P', 'P', 'P', 'P', 'P', 'P'],
    ['R', 'N', 'B', 'K', 'Q', 'B', 'N', 'R'],
  ];

  return (
    <div className="App">
      <BoardComponent board={board} />
    </div>
  );
}

export default App;
