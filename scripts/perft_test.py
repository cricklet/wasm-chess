
import subprocess, itertools
from typing import Iterable

class UciRunner:
    def __init__(self, command):
        self.subprocess = subprocess.Popen(
            command.split(" "),
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
        )

    def send(self, command):
        if self.subprocess.stdin:
            self.subprocess.stdin.write(command.encode() + b'\n')
            self.subprocess.stdin.flush()

    def read(self, until) -> Iterable[str]:
        while self.subprocess.stdout:
            line = self.subprocess.stdout.readline().strip().decode()
            yield line

            for s in until:
                if s in line:
                    return

    def kill(self):
        try:
            self.subprocess.terminate()
        except OSError:
            pass
        self.subprocess.wait()


def print_columns(*columns):
    max_lengths = [max(map(len, col)) for col in columns]
    zipped = itertools.zip_longest(*columns, fillvalue="")
    for row in zipped:
        for (i, col) in enumerate(row):
            padding = max_lengths[i] - len(col) + 1
            print(col + ' ' * padding, end='')
        print()

def main(stockfish, crab, position, moves, depth):
    stockfish_perft_overall = 0
    stockfish_perft_per_move = []

    stockfish.send('position {} moves {}'.format(position, moves))
    stockfish.send('go perft {}'.format(depth))
    for line in stockfish.read(["Nodes searched:", "error"]):
        if "Nodes searched:" in line:
            stockfish_perft_overall = line
        elif ":" in line:
            stockfish_perft_per_move.append(line.strip())


    crab_perft_overall = 0
    crab_perft_per_move = []

    crab.send('position {} moves {}'.format(position, moves))
    crab.send('go perft {}'.format(depth))
    for line in crab.read(["Nodes searched:", "error"]):
        if "Nodes searched:" in line:
            crab_perft_overall = line
        elif ":" in line:
            crab_perft_per_move.append(line.strip())

    stockfish_perft_per_move = sorted(stockfish_perft_per_move)
    crab_perft_per_move = sorted(crab_perft_per_move)

    print_columns(
        sorted(stockfish_perft_per_move),
        list(map(
            lambda x: "!=" if x[0] != x[1] else "==",
            zip(
                sorted(stockfish_perft_per_move),
                sorted(crab_perft_per_move))
        )),
        sorted(crab_perft_per_move),
    )

    print_columns(
        [stockfish_perft_overall],[crab_perft_overall])

if __name__ == '__main__':
    stockfish = UciRunner('stockfish')
    crab = UciRunner('cargo run main')

    try:
        main(
            stockfish,
            crab,
            "startpos",
            "e2e4",
            3
        )
    finally:
        stockfish.kill()
        crab.kill()