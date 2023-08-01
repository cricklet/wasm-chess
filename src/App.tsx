

import './App.css';

import { ReactComponent as RookSvg } from './assets/chess-rook-solid.svg';
import { ReactComponent as KnightSvg } from './assets/chess-knight-solid.svg';
import { ReactComponent as BishopSvg } from './assets/chess-bishop-solid.svg';
import { ReactComponent as KingSvg } from './assets/chess-king-solid.svg';
import { ReactComponent as QueenSvg } from './assets/chess-queen-solid.svg';
import { ReactComponent as PawnSvg } from './assets/chess-pawn-solid.svg';

// import wasmModule from '%PUBLIC_URL%/stockfish.wasm?module';
// import wasmModule from './stockfish.wasm';

// import * as wasm from 'crab_chess'
import * as wasm from 'hello-wasm-pack'
wasm.greet();

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
