/-
CTA.Core.Util
=============
Shared helpers.
-/

import CTA.Core.Prelude

namespace CTA.Core

/-- `inRange lo hi x` is `lo ≤ x ∧ x < hi`. -/
def inRange (lo hi x : Nat) : Prop := lo ≤ x ∧ x < hi

end CTA.Core
