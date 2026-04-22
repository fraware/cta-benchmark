/-
CTA.Core.Prelude
================
Shared imports and notation used by every benchmark scaffold.
Keep this file minimal and dependency-free.
-/

namespace CTA.Core

/-- A deterministic, total comparison over a type `α` induced by `Ord`. -/
abbrev Cmp (α : Type u) := α → α → Ordering

end CTA.Core
