//! Canonical enums used across the benchmark and metrics stack.
//!
//! Every enum serializes as a stable lowercase snake_case string.

use serde::{Deserialize, Serialize};

/// Algorithmic domain an instance belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    /// Array and slice problems (search, partitioning, windowing).
    Arrays,
    /// Sorting algorithms and their invariants.
    Sorting,
    /// Graph algorithms (shortest path, connectivity, traversal).
    Graph,
    /// Greedy algorithms.
    Greedy,
    /// Dynamic programming.
    Dp,
    /// Tree / forest problems.
    Trees,
}

impl Domain {
    /// All variants in declaration order.
    pub const ALL: &'static [Domain] = &[
        Domain::Arrays,
        Domain::Sorting,
        Domain::Graph,
        Domain::Greedy,
        Domain::Dp,
        Domain::Trees,
    ];

    /// Canonical snake_case string form.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Domain::Arrays => "arrays",
            Domain::Sorting => "sorting",
            Domain::Graph => "graph",
            Domain::Greedy => "greedy",
            Domain::Dp => "dp",
            Domain::Trees => "trees",
        }
    }
}

/// Difficulty level for calibration / split analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    /// Short, well-known problems.
    Easy,
    /// Non-trivial implementation.
    Medium,
    /// Conceptually subtle or adversarial.
    Hard,
}

/// Kind of Lean obligation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationKind {
    /// Input precondition.
    Precondition,
    /// Output postcondition.
    Postcondition,
    /// Loop or recursion invariant.
    Invariant,
    /// Termination argument.
    Termination,
    /// Bounds or complexity statement.
    Bounds,
    /// Uniqueness of a returned value.
    Uniqueness,
    /// Monotonicity property.
    Monotonicity,
    /// Optimality (greedy / DP).
    Optimality,
    /// Structural well-formedness of a data structure.
    Structural,
    /// Auxiliary helper lemma.
    Auxiliary,
    /// Fallback when the generator did not classify.
    Unknown,
}

/// Annotator label for semantic faithfulness of a single obligation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaithfulnessLabel {
    /// Captures the intended meaning fully.
    Faithful,
    /// Captures part of the intended meaning but is incomplete.
    Partial,
    /// Does not reflect the intended meaning.
    Unfaithful,
    /// Could not be decided by the annotator (for adjudication).
    Ambiguous,
}

/// Annotator label for consistency with the reference Rust implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsistencyLabel {
    /// Agrees with reference behavior.
    Consistent,
    /// Contradicted by reference behavior.
    Inconsistent,
    /// Not decidable from behavior alone.
    NotApplicable,
}

/// Importance ranking of a reference obligation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Importance {
    /// Must be captured for a faithful obligation set.
    Critical,
    /// Strengthens the set but not required.
    Supporting,
    /// Nice to have.
    Optional,
}

/// Relevance of an obligation to a machine-checkable proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofRelevance {
    /// Directly proves part of the main theorem.
    Direct,
    /// Used as an auxiliary lemma.
    Auxiliary,
    /// Not used in any proof obligation.
    None,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn domain_serializes_snake_case() {
        let j = serde_json::to_string(&Domain::Graph).unwrap();
        assert_eq!(j, "\"graph\"");
        let back: Domain = serde_json::from_str("\"dp\"").unwrap();
        assert_eq!(back, Domain::Dp);
    }

    #[test]
    fn obligation_kind_roundtrips() {
        for k in [
            ObligationKind::Precondition,
            ObligationKind::Postcondition,
            ObligationKind::Invariant,
            ObligationKind::Termination,
            ObligationKind::Bounds,
            ObligationKind::Uniqueness,
            ObligationKind::Monotonicity,
            ObligationKind::Optimality,
            ObligationKind::Structural,
            ObligationKind::Auxiliary,
            ObligationKind::Unknown,
        ] {
            let j = serde_json::to_string(&k).unwrap();
            let back: ObligationKind = serde_json::from_str(&j).unwrap();
            assert_eq!(k, back);
        }
    }

    #[test]
    fn domain_all_matches_variants() {
        assert_eq!(Domain::ALL.len(), 6);
    }
}
