/-
Scaffold for instance `greedy_coin_change_canonical_005`.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Benchmark.Greedy.CoinChangeCanonicalTheory

namespace CTA.Benchmark.Greedy.CoinChangeCanonical005

open CTA.Core
open CTA.Benchmark.Greedy.CoinChangeCanonicalTheory

abbrev Denoms := CoinChangeCanonicalTheory.Denoms
abbrev Counts := CoinChangeCanonicalTheory.Counts
abbrev Canonical := CoinChangeCanonicalTheory.Canonical
abbrev Decomposes := CoinChangeCanonicalTheory.Decomposes
abbrev coinChangeCanonical := CoinChangeCanonicalTheory.coinChangeCanonical

end CTA.Benchmark.Greedy.CoinChangeCanonical005
