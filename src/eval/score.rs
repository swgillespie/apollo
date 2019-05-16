// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::cmp::Ordering;
use std::fmt;
use std::ops::Neg;

/// Score is the output of a board evaluation function. Board evaluators can return one of three
/// variants, depending on the board position.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Score {
    /// The board position is a guaranteed win for the maximizing player in the given number of moves.
    Win(u32),

    /// The board position is a guaranteed loss for the maximizing player in the given number of moves.
    Loss(u32),

    /// The board position is not a guaranteed win or loss and the board evaluator has assigned the
    /// given score to this position, relative to the maximizing player.
    Evaluated(f32),
}

impl Score {
    pub fn step(self) -> Score {
        match self {
            Score::Win(score) => Score::Win(score + 1),
            Score::Loss(score) => Score::Loss(score + 1),
            s => s,
        }
    }

    pub fn step_if(self, cond: bool) -> Score {
        if cond {
            self.step()
        } else {
            self
        }
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Score::Win(in_moves) => write!(f, "#{}", in_moves),
            Score::Loss(in_moves) => write!(f, "#-{}", in_moves),
            Score::Evaluated(score) => write!(f, "{}", score),
        }
    }
}

impl Eq for Score {}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Score) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Score) -> Ordering {
        // The Ord implementation totally orders scores based on "badness":
        //   1. A winning score is better than another winning score if it wins in less moves
        //      than the other.
        //   2. A losing score is better than another losing score if it it loses in more moves
        //      than the other.
        //   3. A winning score is better than all non-winning scores.
        //   4. A losing score is worse than all non-losing scores.
        //   5. Two evaluated scores are comparable like any other number.
        match (self, other) {
            // Rules 1 and 2
            (Score::Win(self_win), Score::Win(other_win)) => other_win.cmp(self_win),
            (Score::Loss(self_loss), Score::Loss(other_loss)) => self_loss.cmp(other_loss),

            // Rules 3 and 4
            (Score::Win(_), _) => Ordering::Greater,
            (_, Score::Win(_)) => Ordering::Less,
            (Score::Loss(_), _) => Ordering::Less,
            (_, Score::Loss(_)) => Ordering::Greater,

            // Rule 5
            (Score::Evaluated(self_score), Score::Evaluated(other_score)) => self_score
                .partial_cmp(other_score)
                .expect("NaN in score comparison"),
        }
    }
}

impl Neg for Score {
    type Output = Score;

    fn neg(self) -> Score {
        match self {
            Score::Win(moves) => Score::Loss(moves),
            Score::Loss(moves) => Score::Win(moves),
            Score::Evaluated(score) => Score::Evaluated(-score),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Score;

    #[test]
    fn win_cmp() {
        assert!(Score::Win(2) > Score::Win(3));
        assert!(Score::Win(4) < Score::Win(3));
        assert!(Score::Win(3) == Score::Win(3));
        assert!(Score::Win(4) > Score::Evaluated(9999999f32));
        assert!(Score::Win(4) > Score::Loss(1));
    }

    #[test]
    fn loss_cmp() {
        assert!(Score::Loss(1) < Score::Loss(2));
        assert!(Score::Loss(2) == Score::Loss(2));
        assert!(Score::Loss(3) > Score::Loss(2));
        assert!(Score::Evaluated(42f32) > Score::Loss(999));
        assert!(Score::Loss(99) < Score::Win(1));
    }

    #[test]
    fn eval_cmp() {
        assert!(Score::Evaluated(1f32) < Score::Evaluated(2f32));
        assert!(Score::Evaluated(3f32) > Score::Evaluated(2f32));
    }

    #[test]
    fn neg() {
        assert_eq!(-Score::Win(1), Score::Loss(1));
        assert_eq!(-Score::Loss(1), Score::Win(1));
        assert_eq!(-Score::Evaluated(1f32), Score::Evaluated(-1f32));
    }

    #[test]
    fn up_one_ply() {
        // white wins in one.
        let score = Score::Win(1);
        let prev_score = -score.step();

        // from the previous ply, black loses in 2.
        assert_eq!(Score::Loss(2), prev_score);
    }
}
