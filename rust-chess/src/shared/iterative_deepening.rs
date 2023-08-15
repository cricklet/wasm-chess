/*
    To implement iterative deepening, we need a few things:
        * SearchStack::search needs to return the PV
        * To do that, as we do a regular search, we need to keep track of
          the best line at each frame.
        * Then, we need some way to sort the moves to prioritize PV moves
*/

pub struct IterativeSearch {
  
}
