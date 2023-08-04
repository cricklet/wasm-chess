
import subprocess, itertools, sys, datetime, select, threading, time
from typing import Iterable

from pprint import pprint

debug = False

class SafeList:
    def __init__(self):
        self.lock = threading.RLock()
        self.buffer = []

    def add(self, line):
        with self.lock:
            self.buffer.append(line)

    def pop(self) -> str | None:
        with self.lock:
            if len(self.buffer) == 0:
                return None
            return self.buffer.pop(0)

    def flush(self) -> list[str]:
        with self.lock:
            result = self.buffer
            self.buffer = []
            return result

    def __len__(self):
        with self.lock:
            return len(self.buffer)

def watch_stdout(subprocess, thread_list: SafeList):
    while subprocess.stdout and subprocess.poll() is None:
        line = subprocess.stdout.readline().strip().decode()
        thread_list.add(line)

class UciRunner:
    def __init__(self, command):
        self.subprocess = subprocess.Popen(
            command.split(" "),
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
        )
        self.stdout_buffer = SafeList()
        self.stdout_watcher = threading.Thread(target=watch_stdout, args=(self.subprocess, self.stdout_buffer))
        self.stdout_watcher.start()

    def send(self, command):
        if self.subprocess.stdin:
            print("sending command: '{}'".format(command))
            self.subprocess.stdin.write(command.encode() + b'\n')
            self.subprocess.stdin.flush()

    def read(self, until: str | None = None) -> list[str]:
        result = []
        if until is None:
            output = self.stdout_buffer.flush()
            for line in output:
                if debug:
                    print("read: {}".format(line))
                result.append(line)
        else:
            while True:
                output = self.stdout_buffer.flush()
                if len(output) == 0:
                    time.sleep(0.05)
                    continue

                for line in output:
                    if debug:
                        print("read: {}".format(line))
                    result.append(line)

                    if until in line:
                        return result

        return result

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
    for line in stockfish.read("Nodes searched:"):
        if "Nodes searched:" in line:
            stockfish_perft_overall = line
        elif ":" in line:
            stockfish_perft_per_move.append(line.strip())


    crab_perft_overall = 0
    crab_perft_per_move = []

    crab.send("isready")
    crab.read("readyok")

    crab.send('position {} moves {}'.format(position, moves))
    crab.read()

    crab.send('d')
    crab.send("isready")
    for line in crab.read("readyok")[:-1]:
        print(line)

    crab.send('go perft {}'.format(depth))
    for line in crab.read("Nodes searched:"):
        if "Nodes searched:" in line:
            crab_perft_overall = line
        elif ":" in line:
            crab_perft_per_move.append(line.strip())

    stockfish_perft_per_move = sorted(stockfish_perft_per_move)
    crab_perft_per_move = sorted(crab_perft_per_move)

    eq_str = lambda x: "!=" if x[0] != x[1] else "=="

    print_columns(
        ['stock'] + sorted(stockfish_perft_per_move),
        [''] + list(map(
            eq_str,
            zip(
                sorted(stockfish_perft_per_move),
                sorted(crab_perft_per_move))
        )),
        ['crab'] + sorted(crab_perft_per_move),
    )

    print_columns(
        [stockfish_perft_overall], [
            eq_str((stockfish_perft_overall, crab_perft_overall))
        ], [crab_perft_overall])

if __name__ == '__main__':
    stockfish = UciRunner('stockfish')
    crab = UciRunner('cargo run main')

    position = "position startpos"
    moves = ""
    depth = 3

    args = sys.argv[1:]

    if len(args) > 0 and args[0] == '-d':
        debug = True
        args.pop(0)

    if len(args) > 0:
        position = args.pop(0)
    if len(args) > 0:
        moves = args.pop(0)
    if len(args) > 0:
        depth = int(args.pop(0))

    print("computing for position: '{}' moves: '{}' depth: '{}'".format(position, moves, depth))

    try:
        main(
            stockfish,
            crab,
            position,
            moves,
            depth
        )
    finally:
        stockfish.kill()
        crab.kill()